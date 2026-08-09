//! AddMessage / BackFillMessage / add helpers.

use super::super::shared::{opt_f32, opt_string};
use super::scroll::truncate_messages;
use crate::lua_api::message_frame::{Message, MessageFrameData};
use crate::lua_api::methods::{borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn add_message(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    add_message_core_from_stack(state, id, true);
    Ok(0)
}

pub(super) fn add_msg(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    add_message_core_from_stack(state, id, true);
    Ok(0)
}

pub(super) fn add_message_silent(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    add_message_core_from_stack(state, id, false);
    Ok(0)
}

pub(super) fn backfill_message(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let text = match opt_string(state, 2) {
        Some(t) => t,
        None => return Ok(0),
    };
    let r = opt_f32(state, 3).unwrap_or(1.0);
    let g = opt_f32(state, 4).unwrap_or(1.0);
    let b = opt_f32(state, 5).unwrap_or(1.0);
    let a = opt_f32(state, 6).unwrap_or(1.0);

    let mut sim = borrow_state_mut(state)?;
    log_message_sim(&sim, id, &text);
    let timestamp = sim.runtime_time_seconds();
    let data = sim.message_frames.entry(id).or_default();
    data.messages.insert(
        0,
        Message {
            text,
            r,
            g,
            b,
            a,
            message_id: None,
            timestamp,
        },
    );
    if data.messages.len() > data.max_lines {
        data.messages.pop();
    }
    data.display_dirty = true;
    Ok(0)
}

/// Read args from the Lua stack (positions 2+) and insert a message.
///
/// This borrows `sim` mutably via `borrow_state_mut` before calling, so the
/// caller must NOT hold any borrow of `SimState` when calling this.
///
/// Stack layout expected: arg1=self(idx1), arg2=text(idx2), arg3=r, arg4=g,
/// arg5=b, arg6=a, arg7=message_id.
pub(super) fn add_message_core_from_stack(state: &mut LuaState, id: u64, log: bool) {
    let text = match opt_string(state, 2) {
        Some(t) => t,
        None => return,
    };
    let r = opt_f32(state, 3).unwrap_or(1.0);
    let g = opt_f32(state, 4).unwrap_or(1.0);
    let b = opt_f32(state, 5).unwrap_or(1.0);
    let a = opt_f32(state, 6).unwrap_or(1.0);
    let message_id = match stack_val(state, 7) {
        Val::Num(n) => Some(n as i64),
        _ => None,
    };

    let mut sim = match borrow_state_mut(state) {
        Ok(s) => s,
        Err(_) => return,
    };
    if log {
        log_message_sim(&sim, id, &text);
    }
    let timestamp = sim.runtime_time_seconds();
    let data = sim.message_frames.entry(id).or_default();
    insert_message(data, text, r, g, b, a, message_id, timestamp);
    truncate_messages(data);
    data.display_dirty = true;
}

pub(super) fn insert_message(
    data: &mut MessageFrameData,
    text: String,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    message_id: Option<i64>,
    timestamp: f64,
) {
    let msg = Message {
        text,
        r,
        g,
        b,
        a,
        message_id,
        timestamp,
    };
    if data.insert_mode == "TOP" {
        data.messages.insert(0, msg);
    } else {
        data.messages.push(msg);
    }
}

pub(super) fn log_message_sim(sim: &crate::lua_api::SimState, id: u64, text: &str) {
    let name = sim
        .widgets
        .get(id)
        .and_then(|w| w.name.as_deref())
        .unwrap_or("?");
    let clean = crate::dump::strip_wow_escapes(text);
    eprintln!("[{name}] {clean}");
}
