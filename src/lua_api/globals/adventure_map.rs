//! `C_AdventureMap` namespace — Broken Isles / Garrison-style adventure-map
//! surface consumed by the Blizzard_AdventureMap addon.
//!
//! Currently implements `GetMapID()`, `Close()`, `GetNumMapInsets()`,
//! `GetMapInsetInfo()`, `GetMapInsetDetailTileInfo()`,
//! `GetNumZoneChoices()`, `GetZoneChoiceInfo()`, `GetNumQuestOffers()`,
//! `GetQuestOfferInfo()`, `GetQuestInfo()`, `GetQuestPortraitInfo()`,
//! `StartQuest()`, and `GetAdventureMapTextureKit()`. Future commits
//! will fill in the rest of the surface (decline/abstain flows, dialog
//! hooks).

use crate::event::{Event, EventArg};
use crate::lua_api::AdventureMapQuestPortrait;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, create_table};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::LuaApiMut;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const NAMESPACE: &str = "C_AdventureMap";
type LuaTableRef = GcRef<Table>;
type RustLuaFn = rilua::vm::closure::RustFn;

const ADVENTURE_MAP_METHODS: &[(&str, RustLuaFn)] = &[
    ("GetMapID", get_map_id),
    ("Close", close),
    ("GetNumMapInsets", get_num_map_insets),
    ("GetMapInsetInfo", get_map_inset_info),
    ("GetMapInsetDetailTileInfo", get_map_inset_detail_tile_info),
    ("GetNumZoneChoices", get_num_zone_choices),
    ("GetZoneChoiceInfo", get_zone_choice_info),
    ("GetNumQuestOffers", get_num_quest_offers),
    ("GetQuestOfferInfo", get_quest_offer_info),
    ("GetQuestInfo", get_quest_info),
    ("GetQuestPortraitInfo", get_quest_portrait_info),
    ("StartQuest", start_quest),
    ("GetAdventureMapTextureKit", get_adventure_map_texture_kit),
];

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    let state = lua.state_mut();
    let table_ref = ensure_namespace_table(state);
    register_adventure_map_methods(state, table_ref)?;
    Ok(())
}

fn register_adventure_map_methods(state: &mut LuaState, table_ref: LuaTableRef) -> LuaResult<()> {
    for (name, rust_fn) in ADVENTURE_MAP_METHODS {
        table_set_rust_fn_static(state, table_ref, name, *rust_fn)?;
    }
    Ok(())
}

fn ensure_namespace_table(state: &mut LuaState) -> GcRef<Table> {
    let key = state.gc.intern_string_static(NAMESPACE.as_bytes());
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|t| t.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(table_ref)) = existing {
        return table_ref;
    }
    let new_val = create_table(state);
    let Val::Table(new_ref) = new_val else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), new_val, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    new_ref
}

fn get_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = borrow_state(state)?.adventure_map.map_id;
    state.push(Val::Num(map_id as f64));
    Ok(1)
}

fn get_adventure_map_texture_kit(state: &mut LuaState) -> LuaResult<u32> {
    let kit = borrow_state(state)?.adventure_map.texture_kit.clone();
    let val = create_string(state, &kit);
    state.push(val);
    Ok(1)
}

fn close(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let elapsed = sim.runtime_time_seconds();
    sim.adventure_map.last_closed = Some(elapsed);
    Ok(0)
}

fn get_num_map_insets(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .adventure_map
        .insets
        .as_ref()
        .map(|insets| insets.len());
    match count {
        Some(n) => state.push(Val::Num(n as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_map_inset_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot_index) = lua_one_based_index_to_slot(stack_val(state, 1)) else {
        return Ok(0);
    };
    let descriptor = {
        let sim = borrow_state(state)?;
        sim.adventure_map
            .insets
            .as_ref()
            .and_then(|insets| insets.get(slot_index).cloned())
    };
    let Some(inset) = descriptor else {
        return Ok(0);
    };
    let title = create_string(state, &inset.title);
    let description = create_string(state, &inset.description);
    let collapsed_icon = create_string(state, &inset.collapsed_icon);
    state.push(Val::Num(inset.map_id as f64));
    state.push(title);
    state.push(description);
    state.push(collapsed_icon);
    state.push(Val::Num(inset.area_table_id as f64));
    state.push(Val::Num(inset.num_detail_tiles as f64));
    state.push(Val::Num(inset.normalized_x));
    state.push(Val::Num(inset.normalized_y));
    Ok(8)
}

fn get_map_inset_detail_tile_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(inset_slot) = lua_one_based_index_to_slot(stack_val(state, 1)) else {
        return Ok(0);
    };
    let Some(tile_slot) = lua_one_based_index_to_slot(stack_val(state, 2)) else {
        return Ok(0);
    };
    let file_data_id = {
        let sim = borrow_state(state)?;
        sim.adventure_map
            .insets
            .as_ref()
            .and_then(|insets| insets.get(inset_slot))
            .and_then(|inset| inset.detail_tiles.get(tile_slot).copied())
    };
    let Some(id) = file_data_id else {
        return Ok(0);
    };
    state.push(Val::Num(id as f64));
    Ok(1)
}

fn get_num_zone_choices(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.adventure_map.zone_choices.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_zone_choice_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot_index) = lua_one_based_index_to_slot(stack_val(state, 1)) else {
        return Ok(0);
    };
    let descriptor = {
        let sim = borrow_state(state)?;
        sim.adventure_map.zone_choices.get(slot_index).cloned()
    };
    let Some(choice) = descriptor else {
        return Ok(0);
    };
    let texture_kit = create_string(state, &choice.texture_kit);
    let name = create_string(state, &choice.name);
    let zone_description = create_string(state, &choice.zone_description);
    state.push(Val::Num(choice.quest_id as f64));
    state.push(texture_kit);
    state.push(name);
    state.push(zone_description);
    state.push(Val::Num(choice.normalized_x));
    state.push(Val::Num(choice.normalized_y));
    Ok(6)
}

fn get_num_quest_offers(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.adventure_map.quest_offers.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_quest_offer_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot_index) = lua_one_based_index_to_slot(stack_val(state, 1)) else {
        return Ok(0);
    };
    let descriptor = {
        let sim = borrow_state(state)?;
        sim.adventure_map.quest_offers.get(slot_index).cloned()
    };
    let Some(offer) = descriptor else {
        return Ok(0);
    };
    let title = create_string(state, &offer.title);
    let description = create_string(state, &offer.description);
    let inset_index = match offer.inset_index {
        Some(value) => Val::Num(value as f64),
        None => Val::Nil,
    };
    state.push(Val::Num(offer.quest_id as f64));
    state.push(Val::Bool(offer.is_trivial));
    state.push(Val::Num(offer.frequency as f64));
    state.push(Val::Bool(offer.is_legendary));
    state.push(title);
    state.push(description);
    state.push(Val::Num(offer.normalized_x));
    state.push(Val::Num(offer.normalized_y));
    state.push(inset_index);
    Ok(9)
}

fn get_quest_info(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Num(quest_id_f) = stack_val(state, 1) else {
        return Ok(0);
    };
    let quest_id = quest_id_f as i64;
    let info = {
        let sim = borrow_state(state)?;
        sim.adventure_map.quest_info.get(&quest_id).cloned()
    };
    let Some(info) = info else {
        return Ok(0);
    };
    let title = create_string(state, &info.title);
    let description = create_string(state, &info.description);
    let objective_text = create_string(state, &info.objective_text);
    state.push(title);
    state.push(description);
    state.push(objective_text);
    Ok(3)
}

fn get_quest_portrait_info(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Num(quest_id_f) = stack_val(state, 1) else {
        return Ok(0);
    };
    let quest_id = quest_id_f as i64;
    let portrait = {
        let sim = borrow_state(state)?;
        sim.adventure_map.quest_portraits.get(&quest_id).cloned()
    };
    let Some(portrait) = portrait else {
        return Ok(0);
    };
    let table_val = build_quest_portrait_table(state, &portrait);
    state.push(table_val);
    Ok(1)
}

fn build_quest_portrait_table(state: &mut LuaState, portrait: &AdventureMapQuestPortrait) -> Val {
    let model_scene = match portrait.model_scene_id {
        Some(value) => Val::Num(value as f64),
        None => Val::Nil,
    };
    let text = create_string(state, &portrait.text);
    let name = create_string(state, &portrait.name);
    let table_val = create_table(state);
    set_table_field(
        state,
        table_val,
        "portraitDisplayID",
        Val::Num(portrait.portrait_display_id as f64),
    );
    set_table_field(
        state,
        table_val,
        "mountPortraitDisplayID",
        Val::Num(portrait.mount_portrait_display_id as f64),
    );
    set_table_field(state, table_val, "modelSceneID", model_scene);
    set_table_field(state, table_val, "text", text);
    set_table_field(state, table_val, "name", name);
    table_val
}

fn start_quest(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Num(quest_id_f) = stack_val(state, 1) else {
        return Ok(0);
    };
    if quest_id_f < 0.0 {
        return Ok(0);
    }
    let quest_id_u32 = quest_id_f as u32;
    let quest_id_i64 = quest_id_f as i64;
    {
        let mut sim = borrow_state_mut(state)?;
        let already_logged = sim.quest_log.iter().any(|id| *id == quest_id_u32);
        if !already_logged {
            sim.quest_log.push(quest_id_u32);
        }
        sim.adventure_map
            .quest_offers
            .retain(|offer| offer.quest_id != quest_id_i64);
        sim.adventure_map
            .zone_choices
            .retain(|choice| choice.quest_id != quest_id_i64);
    }
    push_quest_accepted_event(state, quest_id_f)?;
    Ok(0)
}

fn push_quest_accepted_event(state: &mut LuaState, quest_id: f64) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: "QUEST_ACCEPTED".to_string(),
        args: vec![EventArg::Number(quest_id)],
    });
    Ok(())
}

fn set_table_field(state: &mut LuaState, table_val: Val, key: &'static str, value: Val) {
    let Val::Table(table_ref) = table_val else {
        return;
    };
    let key_ref = state.gc.intern_string_static(key.as_bytes());
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}

/// Convert a Lua-facing 1-based index to a 0-based slot index. Returns
/// `None` for non-numeric or non-positive arguments so the caller can
/// short-circuit with no return values, matching WoW's "unknown index"
/// path.
fn lua_one_based_index_to_slot(arg: Val) -> Option<usize> {
    let Val::Num(index) = arg else {
        return None;
    };
    if index < 1.0 {
        return None;
    }
    Some(index as usize - 1)
}
