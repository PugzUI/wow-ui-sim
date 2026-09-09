//! Native bridge for BackdropTemplateMixin state used by the renderer.

use super::helpers::{frame_id, opt_f32};
use crate::lua_api::methods::{borrow_state_mut, table_get_static, val_to_string};
use crate::lua_bridge::stack_val;
use crate::widget::Color;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
fn info_string(state: &mut LuaState, info: Val, key: &'static str) -> Option<String> {
    if !matches!(info, Val::Table(_)) {
        return None;
    }
    let value = table_get_static(state, info, key);
    val_to_string(state, value)
}

/// Synchronize BackdropTemplateMixin:SetBackdrop into the render-owned Frame.
///
/// The public WoW-facing method is supplied by the Mists compatibility mixin;
/// this deliberately has a private bridge name so a Lua mixin method cannot
/// shadow the native handler.
pub fn set_backdrop_native(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let info = stack_val(state, 2);
    let bg_file = info_string(state, info, "bgFile");
    let edge_file = info_string(state, info, "edgeFile");
    let edge_size = if matches!(info, Val::Table(_)) {
        match table_get_static(state, info, "edgeSize") {
            Val::Num(value) => value as f32,
            _ => 0.0,
        }
    } else {
        0.0
    };
    let insets = if matches!(info, Val::Table(_)) {
        match table_get_static(state, info, "insets") {
            Val::Num(value) => value as f32,
            Val::Table(_) => match table_get_static(state, info, "insets") {
                Val::Table(inset_table) => {
                    let left = table_get_static(state, Val::Table(inset_table), "left");
                    match left {
                        Val::Num(value) => value as f32,
                        _ => 0.0,
                    }
                }
                _ => 0.0,
            },
            _ => 0.0,
        }
    } else {
        0.0
    };

    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.backdrop.enabled = bg_file.is_some() || edge_file.is_some() || edge_size > 0.0;
        frame.backdrop.bg_file = bg_file;
        frame.backdrop.edge_file = edge_file;
        frame.backdrop.edge_size = edge_size;
        frame.backdrop.insets = insets;
    }
    Ok(0)
}

fn set_backdrop_color(state: &mut LuaState, border: bool) -> LuaResult<u32> {
    let id = frame_id(state, 1)?;
    let color = Color::new(
        opt_f32(state, 2),
        opt_f32(state, 3),
        opt_f32(state, 4),
        match stack_val(state, 5) {
            Val::Num(value) => value as f32,
            _ => 1.0,
        },
    );
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        if border {
            frame.backdrop.border_color = color;
        } else {
            frame.backdrop.bg_color = color;
        }
    }
    Ok(0)
}

/// Synchronize BackdropTemplateMixin:SetBackdropColor.
pub fn set_backdrop_color_native(state: &mut LuaState) -> LuaResult<u32> {
    set_backdrop_color(state, false)
}

/// Synchronize BackdropTemplateMixin:SetBackdropBorderColor.
pub fn set_backdrop_border_color_native(state: &mut LuaState) -> LuaResult<u32> {
    set_backdrop_color(state, true)
}
