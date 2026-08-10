//! Windows stub for the `lua_server` module.
//!
//! The Linux/macOS implementation in `lua_server.rs` uses Unix-domain sockets
//! (`std::os::unix::net`) that aren't available on Windows. This stub mirrors
//! the public surface so the rest of the crate compiles, but every entry point
//! is a no-op or returns an error. The IPC bridge between `wow-sim` and
//! `wow-cli` is therefore unavailable on Windows until it's ported (named
//! pipes or TCP loopback).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc;

const UNSUPPORTED: &str = "lua_server IPC is not supported on Windows yet";

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Exec {
        code: String,
    },
    Ping,
    DumpTree {
        filter: Option<String>,
        filter_key: Option<String>,
        visible_only: bool,
        verbose: bool,
    },
    DumpQuads {
        filter: Option<String>,
        verbose: bool,
    },
    AdvanceFrameTime {
        seconds: f64,
        steps: u32,
    },
    Screenshot {
        output: String,
        width: u32,
        height: u32,
        ui_scale: Option<f32>,
        filter: Option<String>,
        crop: Option<String>,
        manifest: Option<String>,
        #[serde(default)]
        requested_ids: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Output(String),
    Error(String),
    Pong,
    Tree(String),
    Quads(String),
}

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

pub fn socket_path() -> PathBuf {
    PathBuf::from(format!("wow-lua-{}.sock", std::process::id()))
}

pub fn init() -> mpsc::Receiver<LuaCommand> {
    let (_tx, rx) = mpsc::channel();
    rx
}

pub mod client {
    use super::UNSUPPORTED;
    use std::path::{Path, PathBuf};

    pub fn exec<P: AsRef<Path>>(_socket: P, _code: &str) -> Result<String, String> {
        Err(UNSUPPORTED.into())
    }

    pub fn advance_frame_time<P: AsRef<Path>>(
        _socket: P,
        _seconds: f64,
        _steps: u32,
    ) -> Result<String, String> {
        Err(UNSUPPORTED.into())
    }

    pub fn mouse_move<P: AsRef<Path>>(_socket: P, _x: f32, _y: f32) -> Result<String, String> {
        Err(UNSUPPORTED.into())
    }

    pub fn mouse_click<P: AsRef<Path>>(_socket: P, _x: f32, _y: f32) -> Result<String, String> {
        Err(UNSUPPORTED.into())
    }

    pub fn ping<P: AsRef<Path>>(_socket: P) -> Result<(), String> {
        Err(UNSUPPORTED.into())
    }

    pub fn find_servers() -> Vec<PathBuf> {
        Vec::new()
    }

    pub fn screenshot<P: AsRef<Path>>(
        _socket: P,
        _output: &str,
        _width: u32,
        _height: u32,
        _filter: Option<String>,
        _crop: Option<String>,
    ) -> Result<String, String> {
        Err(UNSUPPORTED.into())
    }

    pub fn dump_tree<P: AsRef<Path>>(
        _socket: P,
        _filter: Option<String>,
        _filter_key: Option<String>,
        _visible_only: bool,
        _verbose: bool,
    ) -> Result<String, String> {
        Err(UNSUPPORTED.into())
    }

    pub fn dump_quads<P: AsRef<Path>>(
        _socket: P,
        _filter: Option<String>,
        _verbose: bool,
    ) -> Result<String, String> {
        Err(UNSUPPORTED.into())
    }
}
