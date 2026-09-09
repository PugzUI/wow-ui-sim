mod addon_loading;
mod cache_texture;
mod exec_lua;
#[cfg(feature = "gui")]
mod gui_commands;
mod saved_var_config;
mod startup;
mod startup_trace;
use cache_texture::run_cache_texture;
use clap::{Parser, Subcommand};
use startup::init_and_load;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;
use tracing_subscriber::EnvFilter;
use wow_ui_sim::font::WowFontSystem;
use wow_ui_sim::logging;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::SavedVariablesManager;
use wow_ui_sim::screen::ScreenKind;

#[derive(Parser)]
#[command(name = "wow-sim", about = "WoW UI Simulator", version)]
struct Args {
    /// Skip loading WTF Lua SavedVariables (EditMode cache still loads)
    #[arg(long)]
    no_saved_vars: bool,

    /// Skip loading third-party addons
    #[arg(long)]
    no_addons: bool,

    /// Show debug borders and anchor points on all elements
    #[arg(long)]
    debug_elements: bool,

    /// Show red debug borders around all elements
    #[arg(long)]
    debug_borders: bool,

    /// Show green anchor points on all elements
    #[arg(long)]
    debug_anchors: bool,

    /// Delay in milliseconds after firing startup events (for dump-tree/screenshot)
    #[arg(long, value_name = "MS")]
    delay: Option<u64>,

    /// Execute Lua code after startup (runs after first frame in GUI, after events in screenshot/dump-tree).
    /// Prefix with @ to load from file (e.g., --exec-lua @/tmp/debug.lua).
    #[arg(long, value_name = "CODE")]
    exec_lua: Option<String>,

    /// Run `--exec-lua` under secureenv (fenv retargeted via `mark_secure`).
    /// Handy for inspecting/setting secure-scope globals from an ad-hoc
    /// script without staging a TOC-annotated `.lua` file.
    #[arg(long)]
    exec_lua_secure: bool,

    /// Which top-level WoW screen to load.
    #[arg(long, value_enum, default_value_t = ScreenKind::Game, value_name = "SCREEN")]
    screen: ScreenKind,

    /// Compatibility alias for `--screen character-select`.
    #[arg(long, hide = true)]
    character_select: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Load the focused addon test lane and invoke an addon slash command.
    Admin {
        #[command(subcommand)]
        command: AdminCommand,
    },

    /// Load UI and dump frame tree (no GUI needed)
    DumpTree {
        #[arg(short, long)]
        filter: Option<String>,
        #[arg(long)]
        filter_key: Option<String>,
        #[arg(long)]
        visible_only: bool,
        #[arg(short, long)]
        verbose: bool,
        #[arg(long, default_value_t = 1600)]
        width: u32,
        #[arg(long, default_value_t = 1200)]
        height: u32,
    },

    /// Render UI to an image file (no GUI needed)
    #[cfg(feature = "gui")]
    Screenshot {
        #[command(flatten)]
        screenshot: ScreenshotArgs,
    },

    /// Show unique Lua errors as JSON (suppresses other output)
    LuaErrors,

    /// Run simulator self-tests (Wowless test suite)
    SelfTest {
        #[arg(long, default_value_t = 10000)]
        max_ticks: u32,
        #[arg(long)]
        categories: Option<String>,
    },

    /// Run test Lua files from Interface/AddOns/<name>/tests/ or Interface/TestAddOns/<name>/tests/
    RunTests { addon_name: String },

    /// Dump textures used by frames to disk (for debugging atlas crops)
    #[cfg(feature = "gui")]
    DumpTexture {
        #[arg(short, long, default_value = "/tmp/claude/textures")]
        output: PathBuf,
        #[arg(short, long)]
        filter: Option<String>,
        #[arg(long)]
        frame_filter: Option<String>,
    },

    /// Hidden regression helper: boot the GUI App headlessly and click named frames.
    #[cfg(feature = "gui")]
    #[command(hide = true)]
    HeadlessClickProbe {
        #[arg(value_parser = ["achievements", "talents", "mounts", "micromenu"])]
        panel: String,
        #[arg(long, default_value_t = 1024)]
        width: u32,
        #[arg(long, default_value_t = 768)]
        height: u32,
    },

    /// Resolve a WoW texture path through the CASC pipeline and pre-cache it on disk.
    ///
    /// Skips addon loading. Useful to extract a single texture (or retry a previously
    /// failed extraction) without launching the full simulator.
    CacheTexture {
        /// WoW texture path (backslash or forward slash; extension optional)
        path: String,
        /// Delete any existing `.missing` sentinel before retrying extraction
        #[arg(long)]
        force: bool,
    },
}

#[derive(clap::Args, Clone)]
struct ScreenshotArgs {
    #[arg(short, long, default_value = "screenshot.webp")]
    output: PathBuf,
    #[arg(long, default_value_t = 1600)]
    width: u32,
    #[arg(long, default_value_t = 1200)]
    height: u32,
    /// Apply an explicit UIParent scale before layout and capture.
    #[arg(long)]
    ui_scale: Option<f32>,
    #[arg(short, long)]
    filter: Option<String>,
    #[arg(long, value_name = "WxH+X+Y")]
    crop: Option<String>,
    #[arg(long, value_name = "FILTER")]
    dump_tree: Option<Option<String>>,
    /// Write a machine-readable manifest of materialized WeakAuras frames.
    #[arg(long)]
    manifest: Option<PathBuf>,
    /// Resolve this exact WeakAuras display through WeakAuras.GetRegion for the manifest.
    #[arg(long = "manifest-id", value_name = "DISPLAY_ID")]
    requested_ids: Vec<String>,
}

#[derive(Subcommand)]
enum AdminCommand {
    /// Invoke WeakAuras' `/wa` slash command without keyboard or mouse input.
    #[command(name = "wa", alias = "weakauras")]
    WeakAuras,
    /// Preload WeakAuras and capture the Scalpel aura preview without restarting startup.
    #[cfg(feature = "gui")]
    #[command(name = "wa-screenshot")]
    WeakAurasScreenshot {
        #[command(flatten)]
        screenshot: ScreenshotArgs,
        /// Evaluate Lua after WeakAuras login and Options load, before ScalpelPreviewShow.
        /// Prefix with @ to load from a file.
        #[arg(long, value_name = "CODE_OR_@FILE")]
        preview_lua: Option<String>,
    },
}

impl Args {
    fn effective_screen(&self) -> ScreenKind {
        if self.character_select {
            ScreenKind::CharacterSelect
        } else {
            self.screen
        }
    }

    fn is_test_command(&self) -> bool {
        matches!(
            self.command,
            Some(Commands::SelfTest { .. })
                | Some(Commands::RunTests { .. })
                | Some(Commands::Admin { .. })
        )
    }

    fn is_admin_command(&self) -> bool {
        matches!(self.command, Some(Commands::Admin { .. }))
    }

    fn skip_addons(&self) -> bool {
        self.no_addons
            || std::env::var("WOW_SIM_NO_ADDONS")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
    }
}

fn main() {
    wow_ui_sim::stack::ensure_large_stack();
    startup_trace::print_process_started();
    startup_trace::apply_resource_limits();
    set_cwd_to_exe_dir_for_gui_launch();
    if let Err(error) = run_main() {
        report_fatal_error(error.as_ref());
        std::process::exit(1);
    }
}

fn set_cwd_to_exe_dir_for_gui_launch() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(parent) = exe.parent() else {
        return;
    };
    let _ = std::env::set_current_dir(parent);
}

fn run_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if let Some(Commands::CacheTexture { ref path, force }) = args.command {
        return run_cache_texture(path, force);
    }
    if args.is_admin_command() {
        // The admin lane is for focused addon probes, not a full Blizzard UI boot.
        // Force this before any threads or Lua execution are started so an
        // inherited `WOW_SIM_SKIP_BLIZZARD_UI=0` cannot reintroduce Blizzard
        // chrome/assets into the black-stage route.
        unsafe { std::env::set_var("WOW_SIM_SKIP_BLIZZARD_UI", "1") };
    }
    let screen = args.effective_screen();
    let saved_stdout = redirect_if_quiet(&args);
    let init = init_and_load(&args, screen)?;
    #[cfg(feature = "gui")]
    let (env, font_system, saved_vars) = init;
    #[cfg(not(feature = "gui"))]
    let (env, _font_system, _saved_vars) = init;

    let dispatch = CommandDispatch {
        command: args.command,
        env,
        #[cfg(feature = "gui")]
        font_system,
        delay: args.delay,
        exec_lua: resolve_exec_lua(&args.exec_lua),
        exec_lua_secure: args.exec_lua_secure,
        saved_stdout,
        #[cfg(feature = "gui")]
        saved_vars,
        #[cfg(feature = "gui")]
        debug_borders: args.debug_borders,
        #[cfg(feature = "gui")]
        debug_anchors: args.debug_anchors,
        #[cfg(feature = "gui")]
        debug_elements: args.debug_elements,
    };
    dispatch_command(dispatch)
}

fn redirect_if_quiet(args: &Args) -> Option<i32> {
    let quiet = matches!(
        args.command,
        Some(Commands::LuaErrors) | Some(Commands::SelfTest { .. })
    );
    if quiet {
        wow_ui_sim::lua_errors::redirect_stdout_to_stderr()
    } else {
        None
    }
}

fn apply_post_load_workarounds(env: &WowLuaEnv) {
    wow_ui_sim::logging::println_elapsed("[Startup] applying post-load workarounds");
    let post_load_started = Instant::now();
    env.apply_post_load_workarounds();
    wow_ui_sim::logging::println_elapsed(&format!(
        "[Startup] post-load workarounds complete in {:.2?}",
        post_load_started.elapsed()
    ));
}

fn restart_gc_after_bootstrap(env: &WowLuaEnv) {
    wow_ui_sim::logging::println_elapsed("[Startup] restarting GC after bootstrap");
    let gc_restart_started = Instant::now();
    env.gc_restart_after_bootstrap()
        .expect("post-bootstrap full_gc failed");
    wow_ui_sim::logging::println_elapsed(&format!(
        "[Startup] GC restart complete in {:.2?}",
        gc_restart_started.elapsed()
    ));
}

fn resolve_exec_lua(arg: &Option<String>) -> Option<String> {
    arg.as_ref().map(|s| {
        if let Some(path) = s.strip_prefix('@') {
            std::fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("[exec-lua] Failed to read {path}: {e}");
                String::new()
            })
        } else {
            s.clone()
        }
    })
}

fn resolve_preview_lua(arg: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(path) = arg.strip_prefix('@') {
        std::fs::read_to_string(path)
            .map_err(|error| format!("failed to read --preview-lua file {path}: {error}").into())
    } else {
        Ok(arg.to_owned())
    }
}

fn init_sound(env: &WowLuaEnv) {
    let skip = std::env::var("WOW_SIM_NO_SOUND")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if skip {
        logging::println_elapsed("Sound disabled");
        return;
    }
    match wow_ui_sim::sound::SoundManager::new() {
        Some(mgr) => {
            logging::println_elapsed("Sound initialized");
            env.state().borrow_mut().sound_manager = Some(mgr);
        }
        None => logging::println_elapsed("Sound: no audio device available"),
    }
}

fn init_environment(
    args: &Args,
    env: &WowLuaEnv,
    font_system: &Rc<RefCell<WowFontSystem>>,
) -> Result<(), Box<dyn std::error::Error>> {
    logging::init_process_start_time(env.state().borrow().start_time);
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    env.set_font_system(Rc::clone(font_system));
    init_sound(env);
    {
        let mut state = env.state().borrow_mut();
        let mut addon_base_paths = if skip_blizzard_ui() {
            logging::println_elapsed("Blizzard UI loading disabled");
            Vec::new()
        } else {
            vec![runtime_blizzard_ui_addons_path_with_setup().map_err(startup_blizzard_ui_help)?]
        };
        addon_base_paths.extend(wow_ui_sim::paths::default_addons_paths());
        if args.is_test_command() {
            addon_base_paths.push(PathBuf::from("./Interface/TestAddOns"));
        }
        state.addon_base_paths = addon_base_paths;
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    if skip_blizzard_ui() {
        wow_ui_sim::xml::register_mists_compat_templates();
    }
    Ok(())
}

fn skip_blizzard_ui() -> bool {
    std::env::var("WOW_SIM_SKIP_BLIZZARD_UI")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn runtime_blizzard_ui_addons_path_with_setup() -> wow_ui_sim::Result<PathBuf> {
    if let Some(path) = wow_ui_sim::blizzard_ui_sync::cached_blizzard_ui_addons_path() {
        return Ok(path);
    }

    let path = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()?;
    recover_missing_blizzard_ui_addons_path(wow_ui_sim::Error::Other(format!(
        "missing or stale Blizzard UI cache for {:?} profile at {}",
        wow_ui_sim::client_profile::ACTIVE,
        path.display()
    )))
}

fn recover_missing_blizzard_ui_addons_path(
    _missing_cache_error: wow_ui_sim::Error,
) -> wow_ui_sim::Result<PathBuf> {
    logging::println_elapsed("Blizzard UI source missing; syncing Blizzard UI from CASC");
    // Preserve the typed sync error so startup_blizzard_ui_help can pick a
    // tailored message for WowInstallNotFound / BlizzardUiPartial.
    wow_ui_sim::blizzard_ui_sync::sync_blizzard_ui()?;
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
}

fn startup_blizzard_ui_help(error: wow_ui_sim::Error) -> Box<dyn std::error::Error> {
    let message = match &error {
        wow_ui_sim::Error::WowInstallNotFound => format!(
            "Could not find a World of Warcraft installation.\n\n\
             wow-ui-sim needs a local WoW install to load Blizzard UI files via CASC.\n\n\
             Fix:\n  \
             - Install WoW via the Battle.net launcher, OR\n  \
             - Set WOW_INSTALL_PATH to your WoW directory before launching wow-ui-sim.\n\n\
             Default search paths include /Applications/World of Warcraft (macOS), \
             C:\\Program Files (x86)\\World of Warcraft (Windows), and common Wine/Lutris locations on Linux."
        ),
        wow_ui_sim::Error::BlizzardUiPartial {
            missing,
            total,
            last_error,
        } => format!(
            "Your WoW install appears incomplete: {missing} of {total} required files could not be extracted from CASC.\n\
             First missing entry: {last_error}\n\n\
             This usually means your WoW install is partially downloaded or stuck on an older build.\n\n\
             Fix:\n  \
             1. Open Battle.net\n  \
             2. Click the gear icon next to 'Play' on World of Warcraft\n  \
             3. Choose 'Scan and Repair'\n  \
             4. Re-launch wow-ui-sim after the repair completes."
        ),
        _ => format!(
            "{error}\n\nThe simulator stores Blizzard UI source in ~/.cache/wow-ui-sim/blizzard-ui/<profile>/AddOns and tries to sync it from local WoW CASC automatically. Make sure WoW is installed or set WOW_INSTALL_PATH/WOW_DATA_PATH, or run `wow-cli casc sync-blizzard-ui` after configuring CASC."
        ),
    };
    Box::<dyn std::error::Error>::from(message)
}

fn report_fatal_error(error: &dyn std::error::Error) {
    eprintln!("{error}");
    #[cfg(all(windows, feature = "gui"))]
    show_windows_error_message(&error.to_string());
    #[cfg(all(target_os = "macos", feature = "gui"))]
    show_macos_error_message(&error.to_string());
    #[cfg(all(target_os = "linux", feature = "gui"))]
    show_linux_error_message(&error.to_string());
}

#[cfg(all(target_os = "macos", feature = "gui"))]
fn show_macos_error_message(message: &str) {
    // osascript is preinstalled on every macOS. Escape backslashes and
    // double quotes so the AppleScript string literal stays well-formed.
    let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        "display alert \"wow-ui-sim startup error\" message \"{escaped}\" as critical buttons {{\"OK\"}}"
    );
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status();
}

#[cfg(all(target_os = "linux", feature = "gui"))]
fn show_linux_error_message(message: &str) {
    // Try zenity (GNOME), then kdialog (KDE), then xmessage. Any of these
    // may be missing on a minimal install; failing silently is fine because
    // stderr already carries the message.
    let title = "wow-ui-sim startup error";
    let attempts: &[(&str, &[&str])] = &[
        ("zenity", &["--error", "--title", title, "--text", message]),
        ("kdialog", &["--error", message, "--title", title]),
        ("xmessage", &["-center", message]),
    ];
    for (cmd, args) in attempts {
        if std::process::Command::new(cmd)
            .args(*args)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return;
        }
    }
}

#[cfg(all(windows, feature = "gui"))]
fn show_windows_error_message(message: &str) {
    use winapi::um::winuser::{MB_ICONERROR, MB_OK, MessageBoxW};

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let title = wide("wow-ui-sim startup error");
    let body = wide(message);
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            body.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

use wow_ui_sim::startup::{apply_delay, run_extra_update_ticks, settle_headless_startup};

struct CommandDispatch {
    command: Option<Commands>,
    env: WowLuaEnv,
    #[cfg(feature = "gui")]
    font_system: Rc<RefCell<WowFontSystem>>,
    delay: Option<u64>,
    exec_lua: Option<String>,
    exec_lua_secure: bool,
    saved_stdout: Option<i32>,
    #[cfg(feature = "gui")]
    saved_vars: Option<SavedVariablesManager>,
    #[cfg(feature = "gui")]
    debug_borders: bool,
    #[cfg(feature = "gui")]
    debug_anchors: bool,
    #[cfg(feature = "gui")]
    debug_elements: bool,
}

#[cfg(feature = "gui")]
impl CommandDispatch {
    fn debug_options(&self) -> wow_ui_sim::DebugOptions {
        wow_ui_sim::DebugOptions {
            borders: self.debug_borders || self.debug_elements,
            anchors: self.debug_anchors || self.debug_elements,
        }
    }
}

fn dispatch_command(mut dispatch: CommandDispatch) -> Result<(), Box<dyn std::error::Error>> {
    match dispatch.command.take() {
        Some(Commands::DumpTree { .. }) => dispatch_dump_tree(dispatch),
        #[cfg(feature = "gui")]
        Some(Commands::Screenshot { screenshot }) => {
            gui_commands::dispatch_screenshot(dispatch, screenshot)
        }
        Some(Commands::LuaErrors) => run_lua_errors(&dispatch),
        Some(Commands::SelfTest {
            max_ticks,
            ref categories,
        }) => run_self_test(&dispatch, max_ticks, categories.as_deref()),
        Some(Commands::RunTests { ref addon_name }) => {
            run_addon_tests(&dispatch, addon_name);
        }
        Some(Commands::Admin { ref command }) => run_admin_command(&dispatch, command)?,
        #[cfg(feature = "gui")]
        Some(Commands::DumpTexture { .. }) => gui_commands::dispatch_dump_texture(dispatch),
        #[cfg(feature = "gui")]
        Some(Commands::HeadlessClickProbe { .. }) => {
            gui_commands::dispatch_headless_click_probe(dispatch)?
        }
        Some(Commands::CacheTexture { .. }) => {
            unreachable!("CacheTexture is handled before init_and_load");
        }
        #[cfg(feature = "gui")]
        None => return gui_commands::run_gui(dispatch),
        #[cfg(not(feature = "gui"))]
        None => {
            eprintln!("GUI not available (compiled without 'gui' feature).");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn admin_wa_slash_input(command: &AdminCommand) -> Option<&'static str> {
    match command {
        AdminCommand::WeakAuras => Some("/wa"),
        #[cfg(feature = "gui")]
        AdminCommand::WeakAurasScreenshot { .. } => None,
    }
}

fn run_admin_command(
    dispatch: &CommandDispatch,
    command: &AdminCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "gui")]
    let is_wa_command = matches!(
        command,
        AdminCommand::WeakAuras | AdminCommand::WeakAurasScreenshot { .. }
    );
    #[cfg(not(feature = "gui"))]
    let is_wa_command = matches!(command, AdminCommand::WeakAuras);
    if !is_wa_command {
        settle_headless_startup(&dispatch.env);
    }
    if is_wa_command {
        const MAX_LOGIN_TICKS: usize = 8_192;
        const PROBE_SETUP: &str = r#"
            local registry = debug.getregistry()
            local key = "__wow_sim_admin_wa_coroutine_probe"
            local state = {
                previous = registry[key],
                original_resume = coroutine.resume,
                resumes = 0,
                records = {},
                max_records = 32,
                max_text = 120,
            }
            local native_unpack = unpack
            local function pack(...)
                local values = { n = select('#', ...) }
                for index = 1, values.n do
                    values[index] = select(index, ...)
                end
                return values
            end
            local function unpack_values(values, first, last)
                return native_unpack(values, first or 1, last or values.n)
            end
            local function clip(value)
                local ok, text = pcall(tostring, value)
                if not ok then
                    text = "<tostring-error>"
                end
                return string.sub(text, 1, state.max_text)
            end
            local function status(thread)
                if type(thread) ~= "thread" then
                    return "<non-thread>"
                end
                local ok, result = pcall(coroutine.status, thread)
                return ok and result or "<status-error>"
            end
            state.wrapper = function(thread, ...)
                local results = pack(state.original_resume(thread, ...))
                state.resumes = state.resumes + 1
                state.last_status = status(thread)
                state.last_ok = results[1] == true
                state.last_label = results[2] == nil and "<nil>" or clip(results[2])
                state.last_estimate = results[3] == nil and "<nil>" or clip(results[3])
                if #state.records < state.max_records then
                    state.records[#state.records + 1] = string.format(
                        "resume=%d ok=%s status=%s label=%s estimate=%s",
                        state.resumes,
                        state.last_ok and "true" or "false",
                        state.last_status,
                        state.last_label,
                        state.last_estimate
                    )
                end
                return unpack_values(results, 1, results.n)
            end
            registry[key] = state
            coroutine.resume = state.wrapper
            return true
        "#;
        const PROBE_SUMMARY: &str = r#"
            local state = debug.getregistry()["__wow_sim_admin_wa_coroutine_probe"]
            if not state then
                error("admin coroutine probe state is missing")
            end
            local lines = {
                string.format(
                    "resumes=%d last_ok=%s last_status=%s last_label=%s last_estimate=%s records=%d",
                    state.resumes,
                    state.last_ok == nil and "<none>" or (state.last_ok and "true" or "false"),
                    state.last_status or "<none>",
                    state.last_label or "<none>",
                    state.last_estimate or "<none>",
                    #state.records
                ),
            }
            for i = 1, #state.records do
                lines[#lines + 1] = state.records[i]
            end
            return string.sub(table.concat(lines, " | "), 1, 4096)
        "#;
        const PROBE_CLEANUP: &str = r#"
            local registry = debug.getregistry()
            local key = "__wow_sim_admin_wa_coroutine_probe"
            local state = registry[key]
            if not state then
                error("admin coroutine probe state is missing during cleanup")
            end
            local changed = coroutine.resume ~= state.wrapper
            coroutine.resume = state.original_resume
            registry[key] = state.previous
            if changed then
                error("coroutine.resume changed while admin probe was installed")
            end
            return true
        "#;

        let setup = dispatch
            .env
            .eval::<bool>(PROBE_SETUP)
            .map_err(|error| format!("failed to install admin coroutine probe: {error}"))?;
        if !setup {
            return Err("failed to install admin coroutine probe: Lua setup returned false".into());
        }
        // Install first so resumes made by headless startup are observable.
        settle_headless_startup(&dispatch.env);

        let probe_result: Result<Option<String>, Box<dyn std::error::Error>> = (|| {
            let login_finished = (0..MAX_LOGIN_TICKS).any(|_| {
                if dispatch
                    .env
                    .eval::<bool>("return WeakAuras and WeakAuras.IsLoginFinished()")
                    .unwrap_or(false)
                {
                    true
                } else {
                    run_extra_update_ticks(&dispatch.env, 1);
                    false
                }
            });
            if login_finished {
                Ok(None)
            } else {
                let summary = dispatch
                    .env
                    .eval::<String>(PROBE_SUMMARY)
                    .map_err(|error| format!("failed to read admin coroutine probe: {error}"))?;
                Ok(Some(summary))
            }
        })();
        let cleanup_result: Result<(), Box<dyn std::error::Error>> = (|| {
            let cleanup = dispatch
                .env
                .eval::<bool>(PROBE_CLEANUP)
                .map_err(|error| format!("failed to restore admin coroutine probe: {error}"))?;
            if cleanup {
                Ok(())
            } else {
                Err("failed to restore admin coroutine probe: Lua cleanup returned false".into())
            }
        })();
        match (probe_result, cleanup_result) {
            (Err(probe_error), Err(cleanup_error)) => {
                return Err(format!("{probe_error}; additionally, {cleanup_error}").into());
            }
            (Err(probe_error), Ok(())) => return Err(probe_error),
            (_, Err(cleanup_error)) => return Err(cleanup_error),
            (Ok(Some(summary)), Ok(())) => {
                return Err(format!(
                    "WeakAuras login did not finish before the admin command timeout: {summary}"
                )
                .into());
            }
            (Ok(None), Ok(())) => {}
        }
    }
    #[cfg(feature = "gui")]
    if let AdminCommand::WeakAurasScreenshot {
        screenshot,
        preview_lua,
    } = command
    {
        const PREVIEW_SETUP: &str = r#"
            if type(C_AddOns) ~= "table" or type(C_AddOns.LoadAddOn) ~= "function" then
                error("C_AddOns.LoadAddOn is unavailable")
            end
            C_AddOns.LoadAddOn("WeakAurasOptions")
        "#;
        dispatch
            .env
            .exec_maybe_secure(PREVIEW_SETUP, dispatch.exec_lua_secure)
            .map_err(|error| {
                format!("failed to load WeakAurasOptions for aura preview: {error}")
            })?;
        let options_loaded = dispatch
            .env
            .eval::<bool>("return C_AddOns.IsAddOnLoaded(\"WeakAurasOptions\")")
            .unwrap_or(false);
        if !options_loaded {
            return Err("WeakAurasOptions did not load for aura preview".into());
        }
        if let Some(preview_lua) = preview_lua {
            let preview_lua = resolve_preview_lua(&preview_lua)?;
            dispatch
                .env
                .exec_maybe_secure(&preview_lua, dispatch.exec_lua_secure)
                .map_err(|error| {
                    format!("failed to evaluate --preview-lua for aura preview: {error}")
                })?;
        }
        dispatch
            .env
            .exec_maybe_secure(
                r#"
                    if not WeakAuras or type(WeakAuras.ScalpelPreviewShow) ~= "function" then
                        error("WeakAuras.ScalpelPreviewShow is unavailable")
                    end
                    WeakAuras.ScalpelPreviewShow()
                "#,
                dispatch.exec_lua_secure,
            )
            .map_err(|error| format!("WeakAuras aura preview failed: {error}"))?;
        run_extra_update_ticks(&dispatch.env, 3);
        gui_commands::run_admin_screenshot(
            &dispatch.env,
            &dispatch.font_system,
            screenshot,
            dispatch.delay,
            dispatch.exec_lua.as_deref(),
            dispatch.exec_lua_secure,
        );
        return Ok(());
    }
    let input =
        admin_wa_slash_input(command).expect("only the generic admin wa uses slash dispatch");
    if dispatch.env.dispatch_slash_command(input)? {
        run_extra_update_ticks(&dispatch.env, 3);
        let options_loaded = dispatch
            .env
            .eval::<bool>("return C_AddOns.IsAddOnLoaded(\"WeakAurasOptions\")")
            .unwrap_or(false);
        if options_loaded {
            Ok(())
        } else {
            Err("/wa dispatched but WeakAurasOptions did not load".into())
        }
    } else {
        Err(format!("no slash command handler registered for {input}").into())
    }
}

fn dispatch_dump_tree(dispatch: CommandDispatch) {
    let Some(Commands::DumpTree {
        filter,
        filter_key,
        visible_only,
        verbose,
        width,
        height,
    }) = dispatch.command
    else {
        unreachable!("dispatch_dump_tree only fires for Commands::DumpTree");
    };
    run_dump_tree(
        &dispatch.env,
        DumpTreeCommand {
            filter,
            filter_key,
            visible_only,
            verbose,
            width,
            height,
            delay: dispatch.delay,
            exec_lua: dispatch.exec_lua.as_deref(),
            exec_lua_secure: dispatch.exec_lua_secure,
        },
    );
}

fn run_lua_errors(dispatch: &CommandDispatch) {
    wow_ui_sim::lua_errors::run_lua_errors(
        &dispatch.env,
        dispatch.saved_stdout,
        dispatch.exec_lua.as_deref(),
        dispatch.exec_lua_secure,
    );
}

fn run_self_test(dispatch: &CommandDispatch, max_ticks: u32, categories: Option<&str>) {
    if let Some(c) = categories {
        wow_ui_sim::self_test::inject_category_filter(&dispatch.env, c);
    }
    wow_ui_sim::self_test::run_startup(&dispatch.env);
    wow_ui_sim::self_test::run_test(
        &dispatch.env,
        max_ticks,
        dispatch.exec_lua.as_deref(),
        dispatch.exec_lua_secure,
        dispatch.saved_stdout,
    );
}

fn run_addon_tests(dispatch: &CommandDispatch, addon_name: &str) {
    settle_headless_startup(&dispatch.env);
    wow_ui_sim::addon_tests::run_addon_tests(
        &dispatch.env,
        addon_name,
        dispatch.exec_lua.as_deref(),
        dispatch.exec_lua_secure,
    );
}

struct DumpTreeCommand<'a> {
    filter: Option<String>,
    filter_key: Option<String>,
    visible_only: bool,
    verbose: bool,
    width: u32,
    height: u32,
    delay: Option<u64>,
    exec_lua: Option<&'a str>,
    exec_lua_secure: bool,
}

fn run_dump_tree(env: &WowLuaEnv, command: DumpTreeCommand<'_>) {
    settle_headless_startup(env);
    if let Some(code) = command.exec_lua {
        let code = exec_lua::wrap_headless_exec_lua(code);
        if let Err(e) = env.exec_maybe_secure(&code, command.exec_lua_secure) {
            eprintln!("[exec-lua] error: {e}");
        }
    }
    run_extra_update_ticks(env, 3);
    apply_delay(command.delay);
    update_dump_layout_rects(env);
    let state = env.state().borrow();
    let addon_names: Vec<String> = state.addons.iter().map(|a| a.folder_name.clone()).collect();
    wow_ui_sim::dump::print_frame_tree(
        &state.widgets,
        &addon_names,
        command.filter.as_deref(),
        command.filter_key.as_deref(),
        command.visible_only,
        command.verbose,
        command.width as f32,
        command.height as f32,
    );
}

#[cfg(feature = "gui")]
fn update_dump_layout_rects(env: &WowLuaEnv) {
    let mut font_system = wow_ui_sim::render::font::WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, &mut font_system);
    state.ensure_layout_rects();
}

#[cfg(not(feature = "gui"))]
fn update_dump_layout_rects(env: &WowLuaEnv) {
    env.state().borrow_mut().ensure_layout_rects();
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn legacy_character_select_flag_maps_to_character_select_screen() {
        let args = Args::try_parse_from(["wow-sim", "--character-select"])
            .expect("legacy character-select flag should parse");
        assert_eq!(args.effective_screen(), ScreenKind::CharacterSelect);
    }

    #[test]
    fn explicit_screen_still_parses_character_select() {
        let args = Args::try_parse_from(["wow-sim", "--screen", "character-select"])
            .expect("screen option should parse character-select");
        assert_eq!(args.effective_screen(), ScreenKind::CharacterSelect);
    }

    #[test]
    fn explicit_screen_parses_character_create() {
        let args = Args::try_parse_from(["wow-sim", "--screen", "character-create"])
            .expect("character-create screen should parse");
        assert_eq!(args.effective_screen(), ScreenKind::CharacterCreate);
    }

    #[cfg(feature = "gui")]
    #[test]
    fn admin_wa_screenshot_selects_preview_instead_of_slash_dispatch() {
        let wa = AdminCommand::WeakAuras;
        let screenshot = AdminCommand::WeakAurasScreenshot {
            screenshot: ScreenshotArgs {
                output: PathBuf::from("screenshot.webp"),
                width: 1600,
                height: 1200,
                ui_scale: None,
                filter: None,
                crop: None,
                dump_tree: None,
                manifest: None,
                requested_ids: Vec::new(),
            },
            preview_lua: None,
        };
        assert_eq!(admin_wa_slash_input(&wa), Some("/wa"));
        assert_eq!(admin_wa_slash_input(&screenshot), None);
    }

    #[test]
    fn admin_wa_command_parses_without_enabling_gui() {
        let args = Args::try_parse_from(["wow-sim", "admin", "wa"]).expect("admin wa should parse");
        assert!(args.is_admin_command());
        assert!(args.is_test_command());
    }

    #[cfg(feature = "gui")]
    #[test]
    fn admin_wa_screenshot_command_reuses_screenshot_options() {
        let args = Args::try_parse_from([
            "wow-sim",
            "admin",
            "wa-screenshot",
            "--output",
            "/tmp/wa.png",
            "--width",
            "2560",
            "--height",
            "1440",
            "--manifest",
            "/tmp/wa.json",
            "--manifest-id",
            "Display",
        ])
        .expect("admin wa-screenshot should parse");
        let Some(Commands::Admin {
            command:
                AdminCommand::WeakAurasScreenshot {
                    screenshot,
                    preview_lua,
                },
        }) = args.command
        else {
            panic!("expected admin wa-screenshot command");
        };
        assert_eq!(screenshot.output, PathBuf::from("/tmp/wa.png"));
        assert_eq!((screenshot.width, screenshot.height), (2560, 1440));
        assert_eq!(screenshot.manifest, Some(PathBuf::from("/tmp/wa.json")));
        assert_eq!(screenshot.requested_ids, vec!["Display"]);
        assert_eq!(preview_lua, None);
    }

    #[cfg(feature = "gui")]
    #[test]
    fn admin_wa_screenshot_parses_inline_and_file_preview_lua() {
        for value in ["return true", "@/tmp/generated-preview.lua"] {
            let args =
                Args::try_parse_from(["wow-sim", "admin", "wa-screenshot", "--preview-lua", value])
                    .expect("admin wa-screenshot should parse --preview-lua");
            let Some(Commands::Admin {
                command: AdminCommand::WeakAurasScreenshot { preview_lua, .. },
            }) = args.command
            else {
                panic!("expected admin wa-screenshot command");
            };
            assert_eq!(preview_lua.as_deref(), Some(value));
        }
    }
    #[cfg(feature = "gui")]
    #[test]
    fn admin_wa_screenshot_injects_preview_before_availability_check() {
        let source = include_str!("main.rs");
        let options_ready = source
            .find("WeakAurasOptions did not load for aura preview")
            .expect("Options readiness guard should remain");
        let preview_eval = source
            .find("failed to evaluate --preview-lua for aura preview")
            .expect("preview Lua evaluation should remain");
        let availability_check = source
            .find("WeakAuras.ScalpelPreviewShow is unavailable")
            .expect("preview availability check should remain");
        assert!(options_ready < preview_eval);
        assert!(preview_eval < availability_check);
    }
}
