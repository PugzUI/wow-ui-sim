//! Headless probes for the GUI mouse dispatch path.

use std::time::Instant;

use iced::Size;

use super::app::{INIT_DEBUG, INIT_ENV, INIT_EXEC_LUA, INIT_SAVED_VARS};
use super::frame_collect::collect_hittable_frames;
use super::hit_grid::HitGrid;
use super::state::CanvasMessage;
use super::strata_emit::build_hittable_rects;
use super::{App, DebugOptions, Message};
use crate::lua_api::WowLuaEnv;
use crate::render::texture::ui_scale;
use crate::saved_variables::SavedVariablesManager;

#[derive(Debug, Clone, Copy)]
pub struct NamedClick<'a> {
    /// Frame to click. Either a global frame name, or a dotted path whose
    /// first segment is a global frame name and each later segment is a
    /// parentKey (`children_keys`) lookup or a 1-based `#N` child index,
    /// e.g. `MountJournal.ScrollBox.ScrollTarget.#2`.
    pub frame_name: &'a str,
}

pub fn run_headless_named_click_probe(
    env: WowLuaEnv,
    saved_vars: Option<SavedVariablesManager>,
    screen_size: Size,
    setup_lua: &str,
    clicks: &[NamedClick<'_>],
    verify_lua: Option<&str>,
) -> Result<(), String> {
    install_boot_params(env, saved_vars);
    let (mut app, _) = App::boot();
    app.screen_size.set(screen_size);
    app.env
        .borrow()
        .set_screen_size(screen_size.width, screen_size.height);

    let _ = app.update(Message::ProcessTimers(Instant::now()));
    let setup_started = Instant::now();
    app.env
        .borrow()
        .exec(setup_lua)
        .map_err(|error| format!("setup Lua failed: {error}"))?;
    eprintln!(
        "[headless-click-probe] setup Lua completed in {:.2?}",
        setup_started.elapsed()
    );
    let _ = app.update(Message::ProcessTimers(Instant::now()));

    for click in clicks {
        let click_started = Instant::now();
        eprintln!("[headless-click-probe] clicking {}", click.frame_name);
        click_named_frame(&mut app, screen_size, click.frame_name)?;
        eprintln!(
            "[headless-click-probe] clicked {} in {:.2?}",
            click.frame_name,
            click_started.elapsed()
        );
    }

    if let Some(verify_lua) = verify_lua {
        let _ = app.update(Message::ProcessTimers(Instant::now()));
        app.env
            .borrow()
            .exec(verify_lua)
            .map_err(|error| format!("verify Lua failed: {error}"))?;
        eprintln!("[headless-click-probe] verify Lua passed");
    }

    Ok(())
}

fn install_boot_params(env: WowLuaEnv, saved_vars: Option<SavedVariablesManager>) {
    INIT_ENV.with(|cell| *cell.borrow_mut() = Some(env));
    INIT_DEBUG.with(|cell| *cell.borrow_mut() = Some(DebugOptions::default()));
    INIT_SAVED_VARS.with(|cell| *cell.borrow_mut() = saved_vars);
    INIT_EXEC_LUA.with(|cell| *cell.borrow_mut() = None);
}

fn click_named_frame(app: &mut App, screen_size: Size, frame_name: &str) -> Result<(), String> {
    rebuild_hittable_cache(app, screen_size);
    let point = named_frame_center(app, frame_name)?;
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseMove(point)));
    log_hovered_frame(app, frame_name, point);
    rebuild_hittable_cache(app, screen_size);
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseDown(point)));
    rebuild_hittable_cache(app, screen_size);
    let _ = app.update(Message::CanvasEvent(CanvasMessage::MouseUp(point)));
    Ok(())
}

fn log_hovered_frame(app: &App, frame_name: &str, point: iced::Point) {
    let env = app.env.borrow();
    let state = env.state().borrow();
    let hovered = state.hovered_frame;
    let label = hovered
        .and_then(|id| state.widgets.get(id))
        .map(|frame| {
            format!(
                "{} (type={:?} level={} parent_key={:?})",
                frame.name.as_deref().unwrap_or("<anon>"),
                frame.widget_type,
                frame.frame_level,
                frame.parent_key,
            )
        })
        .unwrap_or_else(|| "<none>".to_string());
    eprintln!(
        "[headless-click-probe] hover at ({:.1}, {:.1}) targeting {frame_name}: hovered={:?} {label}",
        point.x, point.y, hovered
    );
}

fn named_frame_center(app: &App, frame_name: &str) -> Result<iced::Point, String> {
    let env = app.env.borrow();
    let state = env.state().borrow();
    let frame_id = resolve_frame_path(&state, frame_name)?;
    let frame = state
        .widgets
        .get(frame_id)
        .expect("selected frame should exist");
    let rect = frame
        .layout_rect
        .ok_or_else(|| format!("frame has no layout rect: {frame_name}"))?;
    Ok(iced::Point::new(
        (rect.x + rect.width / 2.0) * ui_scale(),
        (rect.y + rect.height / 2.0) * ui_scale(),
    ))
}

fn resolve_frame_path(state: &crate::lua_api::state::SimState, path: &str) -> Result<u64, String> {
    let mut segments = path.split('.');
    let root_name = segments.next().expect("split yields at least one segment");
    let mut frame_id = find_global_frame(state, root_name)?;
    for segment in segments {
        frame_id = resolve_child_segment(state, frame_id, segment, path)?;
    }
    Ok(frame_id)
}

fn find_global_frame(
    state: &crate::lua_api::state::SimState,
    frame_name: &str,
) -> Result<u64, String> {
    state
        .widgets
        .iter_ids()
        .filter(|id| {
            state
                .widgets
                .get(*id)
                .is_some_and(|frame| frame.name.as_deref() == Some(frame_name))
        })
        .max_by_key(|id| {
            let frame = state.widgets.get(*id).expect("filtered frame should exist");
            (frame.visible, frame.layout_rect.is_some(), *id)
        })
        .ok_or_else(|| format!("frame not found: {frame_name}"))
}

fn resolve_child_segment(
    state: &crate::lua_api::state::SimState,
    parent_id: u64,
    segment: &str,
    path: &str,
) -> Result<u64, String> {
    let parent = state
        .widgets
        .get(parent_id)
        .ok_or_else(|| format!("frame path parent missing: {path}"))?;
    if let Some(index) = segment.strip_prefix('#') {
        let index: usize = index
            .parse()
            .ok()
            .filter(|index| *index >= 1)
            .ok_or_else(|| {
                format!("invalid 1-based child index '{segment}' in frame path: {path}")
            })?;
        return parent
            .children
            .get(index - 1)
            .copied()
            .ok_or_else(|| format!("child {segment} out of range in frame path: {path}"));
    }
    parent
        .children_keys
        .get(segment)
        .copied()
        .ok_or_else(|| format!("parentKey '{segment}' not found in frame path: {path}"))
}

fn rebuild_hittable_cache(app: &App, screen_size: Size) {
    let env = app.env.borrow();
    let mut state = env.state().borrow_mut();
    state.ensure_layout_rects();
    let strata_buckets = state
        .get_strata_buckets()
        .expect("visible strata buckets should exist")
        .clone();
    let collected = collect_hittable_frames(&state.widgets, &strata_buckets);
    let hittable = build_hittable_rects(&collected, &state.widgets);
    let grid = HitGrid::new(hittable, screen_size.width, screen_size.height);
    *app.cached_hittable.borrow_mut() = Some(grid);
}
