//! Callback registration and dispatch: scroll-changed, display-refreshed,
//! line-right-clicked, text-copied, mark-display-dirty, fade resets, font-string-by-id.

use crate::lua_api::methods::{
    borrow_state_mut, call_function_state, frame_id_from_stack, frame_ref,
    get_or_create_frame_fields, table_get, table_set,
};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) const ON_SCROLL_CHANGED_CB: &str = "onScrollChangedCallback";
pub(super) const ON_LINE_RIGHT_CLICKED_CB: &str = "onLineRightClickedCallback";
pub(super) const ON_DISPLAY_REFRESHED_CBS: &str = "onDisplayRefreshedCallbacks";
pub(super) const ON_TEXT_COPIED_CB: &str = "onTextCopiedCallback";
pub(super) const ON_TEXT_COPIED_ORIG: &str = "_onTextCopiedCallback_orig";

pub(super) fn set_on_scroll_changed_callback(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let func = stack_val(state, 2);
    let fields = get_or_create_frame_fields(state, id);
    let stored = match func {
        Val::Function(_) => func,
        _ => Val::Nil,
    };
    table_set(state, fields, ON_SCROLL_CHANGED_CB, stored);
    Ok(0)
}

pub(super) fn set_on_line_right_clicked_callback(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let func = stack_val(state, 2);
    let fields = get_or_create_frame_fields(state, id);
    let stored = match func {
        Val::Function(_) => func,
        _ => Val::Nil,
    };
    table_set(state, fields, ON_LINE_RIGHT_CLICKED_CB, stored);
    Ok(0)
}

pub(super) fn add_on_display_refreshed_callback(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let func = stack_val(state, 2);
    let Val::Function(_) = func else {
        return Ok(0);
    };
    let fields = get_or_create_frame_fields(state, id);
    let callbacks = get_or_create_display_refreshed_callbacks(state, fields);
    let Val::Table(cbs_ref) = callbacks else {
        return Ok(0);
    };
    let next_idx = {
        let arena = &state.gc.string_arena;
        state
            .gc
            .tables
            .get(cbs_ref)
            .map(|t| t.len(arena) as i64 + 1)
            .unwrap_or(1)
    };
    if let Some(t) = state.gc.tables.get_mut(cbs_ref) {
        let _ = t.raw_set(Val::Num(next_idx as f64), func, &state.gc.string_arena);
    }
    state.gc.barrier_back(cbs_ref);
    Ok(0)
}

pub(super) fn set_on_text_copied_callback(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let func = stack_val(state, 2);
    let fields = get_or_create_frame_fields(state, id);
    match func {
        Val::Function(_) => {
            table_set(state, fields, ON_TEXT_COPIED_ORIG, func);
            // Store an alias under the public key too (the original does a wrapper,
            // but we simplify: both keys point to the same function).
            table_set(state, fields, ON_TEXT_COPIED_CB, func);
        }
        _ => {
            table_set(state, fields, ON_TEXT_COPIED_CB, Val::Nil);
            table_set(state, fields, ON_TEXT_COPIED_ORIG, Val::Nil);
        }
    }
    Ok(0)
}

pub(super) fn mark_display_dirty(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    set_display_dirty_and_fire_callbacks(state, id)
}

pub(super) fn reset_all_fade_times(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    {
        let mut sim = borrow_state_mut(state)?;
        let now = sim.runtime_time_seconds();
        sim.message_frames
            .entry(id)
            .or_default()
            .override_fade_timestamp = now;
    }
    set_display_dirty_and_fire_callbacks(state, id)
}

pub(super) fn reset_message_fade_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let msg_id = {
        use super::super::shared::val_to_f64;
        val_to_f64(stack_val(state, 2)) as i64
    };
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let now = sim.runtime_time_seconds();
        let Some(data) = sim.message_frames.get_mut(&id) else {
            return Ok(0);
        };
        let Some(message) = data
            .messages
            .iter_mut()
            .rev()
            .find(|m| m.message_id == Some(msg_id))
        else {
            return Ok(0);
        };
        message.timestamp = now;
        true
    };
    if changed {
        set_display_dirty_and_fire_callbacks(state, id)?;
    }
    Ok(0)
}

// ── Callback/display-dirty helpers ────────────────────────────────────────────

pub(super) fn set_display_dirty_and_fire_callbacks(
    state: &mut LuaState,
    frame_id: u64,
) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.message_frames
            .entry(frame_id)
            .or_default()
            .display_dirty = true;
    }
    call_display_refreshed_callbacks(state, frame_id)?;
    Ok(0)
}

pub(super) fn call_display_refreshed_callbacks(
    state: &mut LuaState,
    frame_id: u64,
) -> LuaResult<()> {
    let fields = get_or_create_frame_fields(state, frame_id);
    let callbacks = get_or_create_display_refreshed_callbacks(state, fields);
    let Val::Table(cbs_ref) = callbacks else {
        return Ok(());
    };
    let len = {
        let arena = &state.gc.string_arena;
        state
            .gc
            .tables
            .get(cbs_ref)
            .map(|t| t.len(arena))
            .unwrap_or(0)
    };
    if len == 0 {
        return Ok(());
    }
    let self_val = frame_ref(state, frame_id)?;
    for i in 1..=len {
        let cb = state
            .gc
            .tables
            .get(cbs_ref)
            .map(|t| t.get_int(i as i64))
            .unwrap_or(Val::Nil);
        if matches!(cb, Val::Function(_)) {
            let _ = call_function_state(state, cb, &[self_val]);
        }
    }
    Ok(())
}

pub(super) fn call_scroll_changed_callback(
    state: &mut LuaState,
    frame_id: u64,
    offset: i32,
) -> LuaResult<()> {
    let fields = get_or_create_frame_fields(state, frame_id);
    let cb = table_get(state, fields, ON_SCROLL_CHANGED_CB);
    if !matches!(cb, Val::Function(_)) {
        return Ok(());
    }
    let self_val = frame_ref(state, frame_id)?;
    let _ = call_function_state(state, cb, &[self_val, Val::Num(offset as f64)]);
    Ok(())
}

pub(super) fn get_or_create_display_refreshed_callbacks(state: &mut LuaState, fields: Val) -> Val {
    let existing = table_get(state, fields, ON_DISPLAY_REFRESHED_CBS);
    if matches!(existing, Val::Table(_)) {
        return existing;
    }
    let new_table = Val::Table(state.gc.alloc_table(rilua::vm::table::Table::new()));
    table_set(state, fields, ON_DISPLAY_REFRESHED_CBS, new_table);
    new_table
}
