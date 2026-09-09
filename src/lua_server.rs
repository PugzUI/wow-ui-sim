//! Lua execution server for wow-sim.
//!
//! Provides a Unix socket server that accepts Lua code and returns results.

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, mpsc};
use std::thread;

/// Global socket path for signal handler cleanup.
static SOCKET_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Request sent to the Lua server.
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Execute Lua code
    Exec { code: String },
    /// Ping to check if server is alive
    Ping,
    /// Dump the frame tree
    DumpTree {
        /// Filter by name (substring match)
        filter: Option<String>,
        /// Filter by name and print matching subtrees
        filter_key: Option<String>,
        /// Only show visible frames
        visible_only: bool,
        /// Show verbose texture detail lines
        verbose: bool,
    },
    /// Dump cached live GUI quads from the running client
    DumpQuads {
        /// Filter by texture path substring
        filter: Option<String>,
        /// Include per-vertex detail lines
        verbose: bool,
    },
    /// Render a screenshot to a file
    Screenshot {
        /// Output file path
        output: String,
        /// Image width in pixels
        width: u32,
        /// Image height in pixels
        height: u32,
        /// Optional UIParent scale applied before layout and capture.
        ui_scale: Option<f32>,
        /// Render only this frame subtree (name substring match)
        filter: Option<String>,
        /// Crop the output image to WxH+X+Y (e.g., 700x150+400+650)
        crop: Option<String>,
        /// Optional machine-readable native region manifest path.
        manifest: Option<String>,
        /// Exact WeakAuras display IDs to resolve through WeakAuras.GetRegion.
        #[serde(default)]
        requested_ids: Vec<String>,
    },
    /// Render the current native stage into a reduced interactive frame.
    StreamFrame {
        /// Output JPEG or WebP path.
        output: String,
        /// Encoded stream width.
        width: u32,
        /// Encoded stream height.
        height: u32,
        /// Lossy JPEG or WebP quality from 1 to 100.
        quality: f32,
    },
    /// Advance OnUpdate-driven frame time by an exact duration.
    AdvanceFrameTime {
        /// Total frame time to advance in seconds.
        seconds: f64,
        /// Number of equal OnUpdate steps used for the advance.
        steps: u32,
    },
    /// Move the in-app mouse cursor and dispatch hover scripts.
    MouseMove {
        /// Canvas-space x coordinate
        x: f32,
        /// Canvas-space y coordinate
        y: f32,
    },
    /// Click the in-app mouse at a canvas-space coordinate.
    MouseClick {
        /// Canvas-space x coordinate
        x: f32,
        /// Canvas-space y coordinate
        y: f32,
    },
}

/// Response from the Lua server.
#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    /// Execution result (captured print output)
    Output(String),
    /// Error message
    Error(String),
    /// Pong response
    Pong,
    /// Frame tree dump
    Tree(String),
    /// Live quad dump
    Quads(String),
}

/// Command sent to the app from the Lua server.
pub enum LuaCommand {
    Exec {
        code: String,
        respond: mpsc::Sender<Response>,
    },
    DumpTree {
        filter: Option<String>,
        filter_key: Option<String>,
        visible_only: bool,
        verbose: bool,
        respond: mpsc::Sender<Response>,
    },
    DumpQuads {
        filter: Option<String>,
        verbose: bool,
        respond: mpsc::Sender<Response>,
    },
    Screenshot {
        output: String,
        width: u32,
        height: u32,
        ui_scale: Option<f32>,
        filter: Option<String>,
        crop: Option<String>,
        manifest: Option<String>,
        requested_ids: Vec<String>,
        respond: mpsc::Sender<Response>,
    },
    StreamFrame {
        output: String,
        width: u32,
        height: u32,
        quality: f32,
        respond: mpsc::Sender<Response>,
    },
    AdvanceFrameTime {
        seconds: f64,
        steps: u32,
        respond: mpsc::Sender<Response>,
    },
    MouseMove {
        x: f32,
        y: f32,
        respond: mpsc::Sender<Response>,
    },
    MouseClick {
        x: f32,
        y: f32,
        respond: mpsc::Sender<Response>,
    },
}

/// Get the socket path for Lua REPL.
pub fn socket_path() -> PathBuf {
    PathBuf::from(format!("/tmp/wow-lua-{}.sock", std::process::id()))
}

/// Initialize the Lua server.
/// Returns a receiver for commands. The socket is cleaned up on drop.
pub fn init() -> mpsc::Receiver<LuaCommand> {
    // Clean up stale sockets from dead processes
    cleanup_stale_sockets();

    let (tx, rx) = mpsc::channel();
    let path = socket_path();

    // Clean up our own stale socket if it exists
    let _ = std::fs::remove_file(&path);

    // Store path and register signal handlers for cleanup on SIGTERM/SIGINT
    SOCKET_PATH.set(path.clone()).ok();
    register_signal_handlers();

    thread::spawn(move || {
        run_server(tx, path);
    });

    rx
}

/// Clean up stale sockets from dead processes.
fn cleanup_stale_sockets() {
    let pattern = "/tmp/wow-lua-*.sock";
    let Ok(entries) = glob::glob(pattern) else {
        return;
    };

    for entry in entries.flatten() {
        cleanup_stale_socket(&entry);
    }
}

fn cleanup_stale_socket(path: &Path) {
    let Some(pid) = socket_pid(path) else {
        return;
    };
    if process_exists(pid) {
        return;
    }
    if std::fs::remove_file(path).is_ok() {
        crate::logging::eprintln_elapsed(&format!(
            "[wow-sim] Cleaned up stale socket: {}",
            path.display()
        ));
    }
}

fn socket_pid(path: &Path) -> Option<i32> {
    let filename = path.file_name()?.to_str()?;
    let pid = filename
        .strip_prefix("wow-lua-")?
        .strip_suffix(".sock")?
        .parse()
        .ok()?;
    Some(pid)
}

fn process_exists(pid: i32) -> bool {
    // Check if process is still alive using kill(pid, 0).
    let status = unsafe { libc::kill(pid, 0) };
    status == 0
}

extern "C" fn signal_handler(sig: libc::c_int) {
    if let Some(path) = SOCKET_PATH.get() {
        let _ = std::fs::remove_file(path);
    }
    unsafe {
        libc::signal(sig, libc::SIG_DFL);
        libc::raise(sig);
    }
}

fn register_signal_handlers() {
    unsafe {
        libc::signal(
            libc::SIGTERM,
            signal_handler as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGINT,
            signal_handler as *const () as libc::sighandler_t,
        );
    }
}

fn run_server(cmd_tx: mpsc::Sender<LuaCommand>, path: PathBuf) {
    let listener = match UnixListener::bind(&path) {
        Ok(l) => {
            crate::logging::eprintln_elapsed(&format!("[wow-sim] Listening on {}", path.display()));
            l
        }
        Err(e) => {
            crate::logging::eprintln_elapsed(&format!("[wow-sim] Failed to bind: {}", e));
            return;
        }
    };

    // Clean up socket on exit
    struct SocketGuard(PathBuf);
    impl Drop for SocketGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _guard = SocketGuard(path);

    for stream in listener.incoming() {
        handle_incoming_stream(stream, &cmd_tx);
    }
}

fn handle_incoming_stream(stream: std::io::Result<UnixStream>, cmd_tx: &mpsc::Sender<LuaCommand>) {
    let Ok(stream) = stream else {
        let error = stream.expect_err("failed stream must contain an error");
        crate::logging::eprintln_elapsed(&format!("[wow-sim] Accept error: {}", error));
        return;
    };

    if let Err(error) = handle_connection(stream, cmd_tx) {
        crate::logging::eprintln_elapsed(&format!("[wow-sim] Connection error: {}", error));
    }
}

/// Send a command and wait for a response with timeout.
fn send_command(
    cmd_tx: &mpsc::Sender<LuaCommand>,
    build: impl FnOnce(mpsc::Sender<Response>) -> LuaCommand,
) -> Response {
    let (resp_tx, resp_rx) = mpsc::channel();
    if cmd_tx.send(build(resp_tx)).is_err() {
        return Response::Error("App closed".into());
    }
    crate::lua_server_contract::COMMAND_READY.notify_one();
    match resp_rx.recv_timeout(std::time::Duration::from_secs(30)) {
        Ok(r) => r,
        Err(_) => Response::Error("Timeout".into()),
    }
}

fn handle_connection(
    mut stream: UnixStream,
    cmd_tx: &mpsc::Sender<LuaCommand>,
) -> std::io::Result<()> {
    let reader = BufReader::new(stream.try_clone()?);

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        let request = match parse_request(&line) {
            Ok(request) => request,
            Err(response) => {
                write_response(&mut stream, &response)?;
                continue;
            }
        };

        let response = handle_request(request, cmd_tx);
        write_response(&mut stream, &response)?;
    }

    Ok(())
}

fn parse_request(line: &str) -> Result<Request, Response> {
    serde_json::from_str(line)
        .map_err(|error| Response::Error(format!("Invalid request: {}", error)))
}

fn handle_request(request: Request, cmd_tx: &mpsc::Sender<LuaCommand>) -> Response {
    match request {
        Request::Ping => Response::Pong,
        request => send_app_command_request(request, cmd_tx),
    }
}

fn send_app_command_request(request: Request, cmd_tx: &mpsc::Sender<LuaCommand>) -> Response {
    match request {
        Request::Ping => unreachable!("ping requests are handled before command dispatch"),
        Request::Exec { code } => send_exec_command(cmd_tx, code),
        Request::DumpTree {
            filter,
            filter_key,
            visible_only,
            verbose,
        } => send_dump_tree_command(cmd_tx, filter, filter_key, visible_only, verbose),
        Request::DumpQuads { filter, verbose } => send_dump_quads_command(cmd_tx, filter, verbose),
        Request::Screenshot {
            output,
            width,
            height,
            ui_scale,
            filter,
            crop,
            manifest,
            requested_ids,
        } => send_screenshot_command(
            cmd_tx,
            output,
            width,
            height,
            ui_scale,
            filter,
            crop,
            manifest,
            requested_ids,
        ),
        Request::StreamFrame {
            output,
            width,
            height,
            quality,
        } => send_stream_frame_command(cmd_tx, output, width, height, quality),
        Request::AdvanceFrameTime { seconds, steps } => {
            send_advance_frame_time_command(cmd_tx, seconds, steps)
        }
        Request::MouseMove { x, y } => send_mouse_move_command(cmd_tx, x, y),
        Request::MouseClick { x, y } => send_mouse_click_command(cmd_tx, x, y),
    }
}

fn send_exec_command(cmd_tx: &mpsc::Sender<LuaCommand>, code: String) -> Response {
    send_command(cmd_tx, |respond| LuaCommand::Exec { code, respond })
}

fn send_dump_tree_command(
    cmd_tx: &mpsc::Sender<LuaCommand>,
    filter: Option<String>,
    filter_key: Option<String>,
    visible_only: bool,
    verbose: bool,
) -> Response {
    send_command(cmd_tx, |respond| LuaCommand::DumpTree {
        filter,
        filter_key,
        visible_only,
        verbose,
        respond,
    })
}

fn send_dump_quads_command(
    cmd_tx: &mpsc::Sender<LuaCommand>,
    filter: Option<String>,
    verbose: bool,
) -> Response {
    send_command(cmd_tx, |respond| LuaCommand::DumpQuads {
        filter,
        verbose,
        respond,
    })
}

fn send_screenshot_command(
    cmd_tx: &mpsc::Sender<LuaCommand>,
    output: String,
    width: u32,
    height: u32,
    ui_scale: Option<f32>,
    filter: Option<String>,
    crop: Option<String>,
    manifest: Option<String>,
    requested_ids: Vec<String>,
) -> Response {
    send_command(cmd_tx, |respond| LuaCommand::Screenshot {
        output,
        width,
        height,
        ui_scale,
        filter,
        crop,
        manifest,
        requested_ids,
        respond,
    })
}

fn send_stream_frame_command(
    cmd_tx: &mpsc::Sender<LuaCommand>,
    output: String,
    width: u32,
    height: u32,
    quality: f32,
) -> Response {
    send_command(cmd_tx, |respond| LuaCommand::StreamFrame {
        output,
        width,
        height,
        quality,
        respond,
    })
}

fn send_advance_frame_time_command(
    cmd_tx: &mpsc::Sender<LuaCommand>,
    seconds: f64,
    steps: u32,
) -> Response {
    send_command(cmd_tx, |respond| LuaCommand::AdvanceFrameTime {
        seconds,
        steps,
        respond,
    })
}

fn send_mouse_move_command(cmd_tx: &mpsc::Sender<LuaCommand>, x: f32, y: f32) -> Response {
    send_command(cmd_tx, |respond| LuaCommand::MouseMove { x, y, respond })
}

fn send_mouse_click_command(cmd_tx: &mpsc::Sender<LuaCommand>, x: f32, y: f32) -> Response {
    send_command(cmd_tx, |respond| LuaCommand::MouseClick { x, y, respond })
}

fn write_response(stream: &mut UnixStream, response: &Response) -> std::io::Result<()> {
    writeln!(stream, "{}", serde_json::to_string(response).unwrap())
}

/// Client module for connecting to the Lua server.
pub mod client {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::path::Path;

    fn send_request<P: AsRef<Path>>(socket: P, request: Request) -> Result<Response, String> {
        let mut stream =
            UnixStream::connect(socket).map_err(|e| format!("Connect failed: {}", e))?;
        writeln!(stream, "{}", serde_json::to_string(&request).unwrap())
            .map_err(|e| format!("Write failed: {}", e))?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("Read failed: {}", e))?;

        serde_json::from_str(&line).map_err(|e| format!("Invalid response: {}", e))
    }

    pub(super) fn response_result<T>(
        response: Response,
        take_expected: impl FnOnce(Response) -> Option<T>,
    ) -> Result<T, String> {
        match response {
            Response::Error(e) => Err(e),
            response => take_expected(response).ok_or_else(|| "Unexpected response".into()),
        }
    }

    /// Connect to a Lua server and execute code.
    pub fn exec<P: AsRef<Path>>(socket: P, code: &str) -> Result<String, String> {
        let response = send_request(
            socket,
            Request::Exec {
                code: code.to_string(),
            },
        )?;
        match response {
            Response::Output(s) => Ok(s),
            Response::Error(e) => Err(e),
            Response::Pong => Err("Unexpected pong".into()),
            Response::Tree(_) => Err("Unexpected tree".into()),
            Response::Quads(_) => Err("Unexpected quads".into()),
        }
    }

    /// Render the current native stage into a reduced interactive frame.
    pub fn stream_frame<P: AsRef<Path>>(
        socket: P,
        output: &str,
        width: u32,
        height: u32,
        quality: f32,
    ) -> Result<String, String> {
        let response = send_request(
            socket,
            Request::StreamFrame {
                output: output.to_string(),
                width,
                height,
                quality,
            },
        )?;
        response_result(response, |response| match response {
            Response::Output(s) => Some(s),
            _ => None,
        })
    }

    /// Advance deterministic OnUpdate frame time in a running simulator.
    pub fn advance_frame_time<P: AsRef<Path>>(
        socket: P,
        seconds: f64,
        steps: u32,
    ) -> Result<String, String> {
        let response = send_request(socket, Request::AdvanceFrameTime { seconds, steps })?;
        response_result(response, |response| match response {
            Response::Output(s) => Some(s),
            _ => None,
        })
    }

    /// Move the in-app mouse cursor.
    pub fn mouse_move<P: AsRef<Path>>(socket: P, x: f32, y: f32) -> Result<String, String> {
        let response = send_request(socket, Request::MouseMove { x, y })?;
        response_result(response, |response| match response {
            Response::Output(s) => Some(s),
            _ => None,
        })
    }

    /// Click the in-app mouse and return the clicked/hovered frame.
    pub fn mouse_click<P: AsRef<Path>>(socket: P, x: f32, y: f32) -> Result<String, String> {
        let response = send_request(socket, Request::MouseClick { x, y })?;
        response_result(response, |response| match response {
            Response::Output(s) => Some(s),
            _ => None,
        })
    }

    /// Ping the server.
    pub fn ping<P: AsRef<Path>>(socket: P) -> Result<(), String> {
        let response = send_request(socket, Request::Ping)?;
        response_result(response, |response| {
            matches!(response, Response::Pong).then_some(())
        })
    }

    /// Find running wow-lua servers.
    pub fn find_servers() -> Vec<PathBuf> {
        glob::glob("/tmp/wow-lua-*.sock")
            .map(|paths| paths.filter_map(Result::ok).collect())
            .unwrap_or_default()
    }

    /// Take a screenshot (rendered by the server, saved to output path).
    pub fn screenshot<P: AsRef<Path>>(
        socket: P,
        output: &str,
        width: u32,
        height: u32,
        filter: Option<String>,
        crop: Option<String>,
    ) -> Result<String, String> {
        let response = send_request(
            socket,
            Request::Screenshot {
                output: output.to_string(),
                width,
                height,
                ui_scale: None,
                filter,
                crop,
                manifest: None,
                requested_ids: Vec::new(),
            },
        )?;
        match response {
            Response::Output(s) => Ok(s),
            Response::Error(e) => Err(e),
            _ => Err("Unexpected response".into()),
        }
    }

    /// Dump the frame tree.
    pub fn dump_tree<P: AsRef<Path>>(
        socket: P,
        filter: Option<String>,
        filter_key: Option<String>,
        visible_only: bool,
        verbose: bool,
    ) -> Result<String, String> {
        let response = send_request(
            socket,
            Request::DumpTree {
                filter,
                filter_key,
                visible_only,
                verbose,
            },
        )?;
        match response {
            Response::Tree(s) => Ok(s),
            Response::Error(e) => Err(e),
            _ => Err("Unexpected response".into()),
        }
    }

    /// Dump cached live GUI quads.
    pub fn dump_quads<P: AsRef<Path>>(
        socket: P,
        filter: Option<String>,
        verbose: bool,
    ) -> Result<String, String> {
        let response = send_request(socket, Request::DumpQuads { filter, verbose })?;
        response_result(response, |response| match response {
            Response::Quads(s) => Some(s),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LuaCommand, Request, Response, cleanup_stale_socket, client, handle_request, parse_request,
        socket_pid,
    };
    use std::fs::File;
    use std::path::Path;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn socket_pid_extracts_pid_from_wow_lua_socket_name() {
        let pid = socket_pid(Path::new("/tmp/wow-lua-12345.sock"));

        assert_eq!(pid, Some(12345));
    }

    #[test]
    fn socket_pid_rejects_non_matching_socket_names() {
        assert_eq!(socket_pid(Path::new("/tmp/auth-challenge.sock")), None);
        assert_eq!(socket_pid(Path::new("/tmp/wow-lua-not-a-pid.sock")), None);
        assert_eq!(socket_pid(Path::new("/tmp/wow-lua-12345")), None);
    }

    #[test]
    fn cleanup_stale_socket_ignores_files_without_socket_pid() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let socket_path = temp_dir.path().join("auth-challenge.sock");
        File::create(&socket_path).expect("test socket file should be created");

        cleanup_stale_socket(&socket_path);

        assert!(
            socket_path.exists(),
            "non wow-lua socket files must not be removed"
        );
    }

    #[test]
    fn cleanup_stale_socket_keeps_socket_for_live_process() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let pid = std::process::id();
        let socket_path = temp_dir.path().join(format!("wow-lua-{pid}.sock"));
        File::create(&socket_path).expect("test socket file should be created");

        cleanup_stale_socket(&socket_path);

        assert!(
            socket_path.exists(),
            "socket for current live process must not be removed"
        );
    }

    #[test]
    fn parse_request_accepts_dump_quads_payload() {
        let request = serde_json::to_string(&Request::DumpQuads {
            filter: Some("uigroupmanager".to_string()),
            verbose: true,
        })
        .unwrap();

        let parsed = parse_request(&request).expect("dump-quads request should parse");

        assert!(matches!(
            parsed,
            Request::DumpQuads {
                filter: Some(filter),
                verbose: true
            } if filter == "uigroupmanager"
        ));
    }

    #[test]
    fn parse_request_accepts_stream_frame_payload() {
        let request = serde_json::to_string(&Request::StreamFrame {
            output: "/tmp/frame.jpg".to_string(),
            width: 960,
            height: 540,
            quality: 70.0,
        })
        .unwrap();

        let parsed = parse_request(&request).expect("stream-frame request should parse");

        assert!(matches!(
            parsed,
            Request::StreamFrame {
                output,
                width: 960,
                height: 540,
                quality
            } if output == "/tmp/frame.jpg" && (quality - 70.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn handle_request_dispatches_stream_frame_commands() {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let response_thread = thread::spawn(move || {
            let command = cmd_rx.recv().expect("command should be sent");
            match command {
                LuaCommand::StreamFrame {
                    output,
                    width,
                    height,
                    quality,
                    respond,
                } => {
                    assert_eq!(output, "/tmp/frame.jpg");
                    assert_eq!((width, height), (960, 540));
                    assert!((quality - 70.0).abs() < f32::EPSILON);
                    respond
                        .send(Response::Output("streamed".to_string()))
                        .expect("response should be accepted");
                }
                _ => panic!("unexpected command"),
            }
        });

        let response = handle_request(
            Request::StreamFrame {
                output: "/tmp/frame.jpg".to_string(),
                width: 960,
                height: 540,
                quality: 70.0,
            },
            &cmd_tx,
        );

        response_thread.join().unwrap();
        assert!(matches!(response, Response::Output(output) if output == "streamed"));
    }

    #[test]
    fn parse_request_accepts_advance_frame_time_payload() {
        let request = serde_json::to_string(&Request::AdvanceFrameTime {
            seconds: 0.75,
            steps: 3,
        })
        .unwrap();

        let parsed = parse_request(&request).expect("advance-frame-time request should parse");

        assert!(matches!(
            parsed,
            Request::AdvanceFrameTime {
                seconds,
                steps: 3
            } if (seconds - 0.75).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn handle_request_dispatches_advance_frame_time_commands() {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let response_thread = thread::spawn(move || {
            let command = cmd_rx.recv().expect("command should be sent");
            match command {
                LuaCommand::AdvanceFrameTime {
                    seconds,
                    steps,
                    respond,
                } => {
                    assert!((seconds - 0.75).abs() < f64::EPSILON);
                    assert_eq!(steps, 3);
                    respond
                        .send(Response::Output("advanced".to_string()))
                        .unwrap();
                }
                _ => panic!("expected advance-frame-time command"),
            }
        });

        let response = handle_request(
            Request::AdvanceFrameTime {
                seconds: 0.75,
                steps: 3,
            },
            &cmd_tx,
        );

        response_thread.join().unwrap();
        assert!(matches!(response, Response::Output(body) if body == "advanced"));
    }

    #[test]
    fn handle_request_dispatches_dump_quads_commands() {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let response_thread = thread::spawn(move || {
            let command = cmd_rx.recv().expect("command should be sent");
            match command {
                LuaCommand::DumpQuads {
                    filter,
                    verbose,
                    respond,
                } => {
                    assert_eq!(filter.as_deref(), Some("uigroupmanager"));
                    assert!(verbose);
                    respond
                        .send(Response::Quads("quad dump".to_string()))
                        .unwrap();
                }
                _ => panic!("expected dump-quads command"),
            }
        });

        let response = handle_request(
            Request::DumpQuads {
                filter: Some("uigroupmanager".to_string()),
                verbose: true,
            },
            &cmd_tx,
        );

        response_thread.join().unwrap();
        assert!(matches!(response, Response::Quads(body) if body == "quad dump"));
    }

    #[test]
    fn screenshot_request_preserves_exact_manifest_ids() {
        let request = serde_json::to_string(&Request::Screenshot {
            output: "/tmp/capture.png".to_string(),
            width: 2560,
            height: 1440,
            ui_scale: Some(0.53),
            filter: None,
            crop: None,
            manifest: Some("/tmp/capture.json".to_string()),
            requested_ids: vec!["Aura A".to_string(), "Aura B".to_string()],
        })
        .unwrap();
        let parsed = parse_request(&request).expect("screenshot request should parse");
        assert!(matches!(
            parsed,
            Request::Screenshot {
                requested_ids,
                filter: None,
                ..
            } if requested_ids == ["Aura A", "Aura B"]
        ));
    }

    #[test]
    fn screenshot_request_defaults_missing_manifest_ids_for_older_clients() {
        let parsed = parse_request(
            r#"{"Screenshot":{"output":"/tmp/capture.png","width":2560,"height":1440,"ui_scale":0.53,"filter":null,"crop":null,"manifest":"/tmp/capture.json"}}"#,
        )
        .expect("legacy screenshot request should parse");
        assert!(matches!(
            parsed,
            Request::Screenshot { requested_ids, .. } if requested_ids.is_empty()
        ));
    }

    #[test]
    fn handle_request_forwards_exact_manifest_ids() {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let response_thread = thread::spawn(move || {
            let command = cmd_rx.recv().expect("command should be sent");
            match command {
                LuaCommand::Screenshot {
                    requested_ids,
                    filter,
                    respond,
                    ..
                } => {
                    assert_eq!(requested_ids, ["Aura A", "Aura B"]);
                    assert_eq!(filter, None);
                    respond
                        .send(Response::Output("captured".to_string()))
                        .unwrap();
                }
                _ => panic!("expected screenshot command"),
            }
        });
        let response = handle_request(
            Request::Screenshot {
                output: "/tmp/capture.png".to_string(),
                width: 2560,
                height: 1440,
                ui_scale: Some(0.53),
                filter: None,
                crop: None,
                manifest: Some("/tmp/capture.json".to_string()),
                requested_ids: vec!["Aura A".to_string(), "Aura B".to_string()],
            },
            &cmd_tx,
        );
        response_thread.join().unwrap();
        assert!(matches!(response, Response::Output(body) if body == "captured"));
    }

    #[test]
    fn client_response_result_maps_expected_error_and_unexpected() {
        let pong = client::response_result(Response::Pong, |response| {
            matches!(response, Response::Pong).then_some(())
        });
        assert!(pong.is_ok());

        let error = client::response_result(Response::Error("boom".to_string()), |_| Some(()));
        assert_eq!(error, Err("boom".to_string()));

        let unexpected = client::response_result(Response::Tree("tree".to_string()), |response| {
            matches!(response, Response::Pong).then_some(())
        });
        assert_eq!(unexpected, Err("Unexpected response".to_string()));
    }
}
