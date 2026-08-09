//! C_SpecializationInfo and UIWidgetContainerMixin implementations.

use crate::lua_api::game_data::CLASS_LABELS;
use crate::lua_api::game_data::CastingState;
use crate::lua_api::globals::spellbook_data;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, frame_id_from_stack,
};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use crate::specializations;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

use super::helpers::set_global_val;

const CLASS_FILES: &[&str] = &[
    "WARRIOR",
    "PALADIN",
    "HUNTER",
    "ROGUE",
    "PRIEST",
    "DEATHKNIGHT",
    "SHAMAN",
    "MAGE",
    "WARLOCK",
    "MONK",
    "DRUID",
    "DEMONHUNTER",
    "EVOKER",
];

type LuaTableRef = GcRef<Table>;
type RustLuaFn = rilua::vm::closure::RustFn;

const C_SPECIALIZATION_INFO_METHODS: &[(&str, RustLuaFn)] = &[
    ("GetSpecialization", c_spec_get_specialization),
    ("GetSpecializationInfo", c_spec_get_specialization_info),
    ("GetClassIDFromSpecID", c_spec_get_class_id_from_spec_id),
    (
        "GetNumSpecializationsForClassID",
        c_spec_get_num_specializations_for_class_id,
    ),
    ("IsInitialized", c_spec_is_initialized),
    (
        "CanPlayerUseTalentSpecUI",
        c_spec_can_player_use_talent_spec_ui,
    ),
    ("CanPlayerUseTalentUI", c_spec_can_player_use_talent_ui),
    ("GetActiveSpecGroup", c_spec_get_active_spec_group),
    #[cfg(feature = "client-mists")]
    ("GetTalentInfo", c_spec_get_talent_info),
    ("GetSpellsDisplay", c_spec_get_spells_display),
    ("GetSpecIDs", c_spec_get_spec_ids),
    ("SetSpecialization", c_spec_set_specialization),
];

const LEGACY_SPECIALIZATION_GLOBALS: &[(&str, RustLuaFn)] = &[
    ("GetNumSpecGroups", get_num_spec_groups),
    ("GetNumSpecializations", get_num_specializations),
    (
        "GetSpecializationInfoForClassID",
        get_specialization_info_for_class_id,
    ),
    ("GetSpecializationInfoByID", get_specialization_info_by_id),
    ("GetInspectSpecialization", get_inspect_specialization),
    ("GetSpecializationRoleByID", get_specialization_role_by_id),
    ("GetSpecializationRole", get_specialization_role),
    ("GetSpecializationRoleEnum", get_specialization_role_enum),
    (
        "GetSpecializationRoleEnumByID",
        get_specialization_role_enum_by_id,
    ),
    ("GetLFGStringFromEnum", get_lfg_string_from_enum),
    ("SetSpecialization", set_specialization),
];

const SPEC_ACTIVATION_SPELL_ID: u32 = 200749;
const SPEC_ACTIVATION_CAST_SECONDS: f64 = 1.5;

pub fn register_c_specialization_info(state: &mut LuaState) -> LuaResult<()> {
    let t = create_table(state);
    let Val::Table(t_ref) = t else {
        unreachable!("create_table must return a table");
    };
    register_c_specialization_info_methods(state, t_ref)?;
    set_global_val(state, "C_SpecializationInfo", t);
    register_legacy_specialization_globals(state)?;
    Ok(())
}

fn register_c_specialization_info_methods(
    state: &mut LuaState,
    table_ref: LuaTableRef,
) -> LuaResult<()> {
    for (name, rust_fn) in C_SPECIALIZATION_INFO_METHODS {
        table_set_rust_fn_static(state, table_ref, name, *rust_fn)?;
    }
    Ok(())
}

pub fn register_widget_container_mixin(state: &mut LuaState) -> LuaResult<()> {
    let mixin = create_table(state);
    let Val::Table(mixin_ref) = mixin else {
        unreachable!("create_table must return a table");
    };
    table_set_rust_fn_static(state, mixin_ref, "OnLoad", ui_widget_container_on_load)?;
    table_set_rust_fn_static(
        state,
        mixin_ref,
        "GetNumWidgetsShowing",
        ui_widget_container_get_num_widgets_showing,
    )?;
    let key_ref = state.gc.intern_string(b"UIWidgetContainerMixin");
    let global_ref = state.global;
    if let Some(global) = state.gc.tables.get_mut(global_ref) {
        let _ = global.raw_set(Val::Str(key_ref), mixin, &state.gc.string_arena);
    }
    state.gc.barrier_back(global_ref);
    Ok(())
}

fn c_spec_get_specialization(state: &mut LuaState) -> LuaResult<u32> {
    let active_spec_index = borrow_state(state)?.player.active_spec_index;
    state.push(Val::Num(active_spec_index as f64));
    Ok(1)
}

fn c_spec_get_specialization_info(state: &mut LuaState) -> LuaResult<u32> {
    let requested_index = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => 1,
    };
    let spec = requested_or_active_spec(state, requested_index);
    let Some(spec) = spec else {
        return Ok(0);
    };
    push_specialization_info(state, spec);
    Ok(10)
}

fn c_spec_get_spec_ids(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = match stack_val(state, 1) {
        Val::Num(n) if n > 0.0 => n as u32,
        _ => borrow_state(state)?.player.class_index as u32,
    };
    let spec_ids = create_table(state);
    let Val::Table(spec_ids_ref) = spec_ids else {
        unreachable!("create_table must return a table");
    };
    for (index, spec) in specializations::specs_for_class(class_id).enumerate() {
        if let Some(table) = state.gc.tables.get_mut(spec_ids_ref) {
            let _ = table.raw_set(
                Val::Num((index + 1) as f64),
                Val::Num(spec.id as f64),
                &state.gc.string_arena,
            );
        }
    }
    state.gc.barrier_back(spec_ids_ref);
    state.push(spec_ids);
    Ok(1)
}

fn c_spec_get_class_id_from_spec_id(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = match stack_val(state, 1) {
        Val::Num(n) => n as u32,
        _ => 0,
    };
    let class_id = specializations::spec_by_id(spec_id)
        .map(|spec| spec.class_id as f64)
        .unwrap_or(0.0);
    state.push(Val::Num(class_id));
    Ok(1)
}

fn c_spec_get_num_specializations_for_class_id(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = match stack_val(state, 1) {
        Val::Num(n) => n as u32,
        _ => 0,
    };
    let count = specializations::specs_for_class(class_id).count() as f64;
    state.push(Val::Num(count));
    Ok(1)
}

fn c_spec_is_initialized(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_spec_can_player_use_talent_spec_ui(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    state.push(Val::Nil);
    Ok(2)
}

fn c_spec_can_player_use_talent_ui(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    state.push(Val::Nil);
    Ok(2)
}

fn c_spec_get_active_spec_group(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1.0));
    Ok(1)
}

#[cfg(feature = "client-mists")]
fn c_spec_get_talent_info(state: &mut LuaState) -> LuaResult<u32> {
    super::mists_talents::get_talent_info(state)
}

fn c_spec_get_spells_display(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => 0,
    };
    let Some(spell_ids) = spec_display_spell_ids(spec_id) else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let spells = create_table(state);
    let Val::Table(spells_ref) = spells else {
        unreachable!("create_table must return a table");
    };
    for (index, spell_id) in spell_ids.iter().copied().enumerate() {
        if let Some(table) = state.gc.tables.get_mut(spells_ref) {
            let _ = table.raw_set(
                Val::Num((index + 1) as f64),
                Val::Num(spell_id as f64),
                &state.gc.string_arena,
            );
        }
    }
    state.gc.barrier_back(spells_ref);
    state.push(spells);
    Ok(1)
}

fn c_spec_set_specialization(state: &mut LuaState) -> LuaResult<u32> {
    let requested_index = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => 0,
    };
    let can_set = player_spec_by_index(state, requested_index).is_some();
    if can_set && start_specialization_change(state, requested_index.max(1))? {
        let player = create_string(state, "player");
        let spell_id_val = Val::Num(SPEC_ACTIVATION_SPELL_ID as f64);
        fire_named_event_state(state, "UNIT_SPELLCAST_START", &[player, spell_id_val]);
    }
    state.push(Val::Bool(can_set));
    Ok(1)
}

fn start_specialization_change(state: &mut LuaState, target_index: i32) -> LuaResult<bool> {
    let mut sim = borrow_state_mut(state)?;
    if sim.player.active_spec_index == target_index && sim.player.pending_spec_change.is_none() {
        return Ok(false);
    }

    sim.player.pending_spec_change = Some(target_index);
    let now = sim.runtime_time_seconds();
    let cast_id = sim.next_cast_id;
    sim.next_cast_id = sim.next_cast_id.wrapping_add(1);
    sim.casting = Some(CastingState {
        spell_id: SPEC_ACTIVATION_SPELL_ID,
        spell_name: "Activate Specialization".to_string(),
        icon_path: String::new(),
        start_time: now,
        end_time: now + SPEC_ACTIVATION_CAST_SECONDS,
        cast_id,
        num_empower_stages: 0,
    });
    Ok(true)
}

fn set_specialization(state: &mut LuaState) -> LuaResult<u32> {
    let requested_index = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => 0,
    };
    let can_set = player_spec_by_index(state, requested_index).is_some();
    if can_set {
        activate_specialization_now(state, requested_index.max(1))?;
    }
    state.push(Val::Bool(can_set));
    Ok(1)
}

fn activate_specialization_now(state: &mut LuaState, target_index: i32) -> LuaResult<()> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.player.active_spec_index = target_index;
        sim.player.pending_spec_change = None;
        sim.casting = None;
    }

    let player = create_string(state, "player");
    fire_named_event_state(state, "PLAYER_SPECIALIZATION_CHANGED", &[player]);
    fire_named_event_state(state, "PLAYER_TALENT_UPDATE", &[]);
    fire_named_event_state(state, "ACTIVE_TALENT_GROUP_CHANGED", &[]);
    Ok(())
}

fn register_legacy_specialization_globals(state: &mut LuaState) -> LuaResult<()> {
    for (name, rust_fn) in LEGACY_SPECIALIZATION_GLOBALS {
        table_set_rust_fn_static(state, state.global, name, *rust_fn)?;
    }
    Ok(())
}

fn get_num_spec_groups(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(1.0));
    Ok(1)
}

fn get_num_specializations(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = borrow_state(state)?.player.class_index.max(1) as u32;
    let count = specializations::specs_for_class(class_id).count() as f64;
    state.push(Val::Num(count));
    Ok(1)
}

fn get_specialization_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let spec_id = match stack_val(state, 1) {
        Val::Num(n) => n as u32,
        _ => 0,
    };
    let Some(spec) = specializations::spec_by_id(spec_id) else {
        return Ok(0);
    };
    let class_index = spec.class_id.max(1) as usize - 1;
    let class_name = create_string(state, CLASS_LABELS.get(class_index).copied().unwrap_or(""));
    let class_file = create_string(state, CLASS_FILES.get(class_index).copied().unwrap_or(""));
    let spec_name = create_string(state, spec.name);
    let spec_description = create_string(state, spec.description);
    let spec_role = create_string(state, spec.role);
    state.push(Val::Num(spec.id as f64));
    state.push(spec_name);
    state.push(spec_description);
    state.push(Val::Num(spec.icon_file_data_id as f64));
    state.push(spec_role);
    state.push(class_file);
    state.push(class_name);
    Ok(7)
}

fn get_specialization_info_for_class_id(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = match stack_val(state, 1) {
        Val::Num(n) => n as u32,
        _ => 0,
    };
    let spec_index = match stack_val(state, 2) {
        Val::Num(n) if n >= 1.0 => n as usize,
        _ => return Ok(0),
    };
    let Some(spec) = specializations::specs_for_class(class_id).nth(spec_index - 1) else {
        return Ok(0);
    };

    push_class_specialization_info(state, spec);
    Ok(9)
}

fn get_inspect_specialization(state: &mut LuaState) -> LuaResult<u32> {
    let unit = match stack_val(state, 1) {
        Val::Str(s) => state
            .gc
            .string_arena
            .get(s)
            .and_then(|lua_str| std::str::from_utf8(lua_str.data()).ok())
            .unwrap_or_default()
            .to_owned(),
        _ => String::new(),
    };
    if unit.is_empty() {
        state.push(Val::Num(0.0));
        return Ok(1);
    }

    let Some(spec) = active_player_spec(state) else {
        state.push(Val::Num(0.0));
        return Ok(1);
    };
    state.push(Val::Num(spec.id as f64));
    Ok(1)
}

fn get_specialization_role(state: &mut LuaState) -> LuaResult<u32> {
    push_specialization_role(state, requested_spec_role)
}

fn get_specialization_role_by_id(state: &mut LuaState) -> LuaResult<u32> {
    push_specialization_role(state, requested_spec_role_by_id)
}

fn push_specialization_role(
    state: &mut LuaState,
    role_lookup: fn(&LuaState) -> Option<&'static str>,
) -> LuaResult<u32> {
    let Some(role) = role_lookup(state) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let role = create_string(state, role);
    state.push(role);
    Ok(1)
}

fn get_specialization_role_enum(state: &mut LuaState) -> LuaResult<u32> {
    let Some(role) = requested_spec_role(state) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    state.push(Val::Num(role_enum_value(role)));
    Ok(1)
}

fn get_specialization_role_enum_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let Some(role) = requested_spec_role_by_id(state) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    state.push(Val::Num(role_enum_value(role)));
    Ok(1)
}

fn requested_spec_role_by_id(state: &LuaState) -> Option<&'static str> {
    let spec_id = match stack_val(state, 1) {
        Val::Num(n) => n as u32,
        _ => 0,
    };
    specializations::spec_by_id(spec_id).map(|spec| spec.role)
}

fn get_lfg_string_from_enum(state: &mut LuaState) -> LuaResult<u32> {
    let role = match stack_val(state, 1) {
        Val::Num(0.0) => "TANK",
        Val::Num(1.0) => "HEALER",
        Val::Num(_) => "DAMAGER",
        _ => "",
    };
    let role = create_string(state, role);
    state.push(role);
    Ok(1)
}

fn requested_spec_role(state: &LuaState) -> Option<&'static str> {
    let requested_index = match stack_val(state, 1) {
        Val::Num(n) => n as i32,
        _ => 1,
    };
    requested_or_active_spec(state, requested_index).map(|spec| spec.role)
}

fn requested_or_active_spec(
    state: &LuaState,
    requested_index: i32,
) -> Option<&'static specializations::SpecInfo> {
    let (class_id, active_spec_index) = {
        let sim = borrow_state(state).ok()?;
        (sim.player.class_index as u32, sim.player.active_spec_index)
    };
    let requested_spec_index = requested_index.max(1);
    specializations::specs_for_class(class_id)
        .nth((requested_spec_index - 1) as usize)
        .or_else(|| {
            let active_spec_index = active_spec_index.max(1);
            specializations::specs_for_class(class_id).nth((active_spec_index - 1) as usize)
        })
}

fn player_spec_by_index(
    state: &LuaState,
    requested_index: i32,
) -> Option<&'static specializations::SpecInfo> {
    let class_id = borrow_state(state).ok()?.player.class_index as u32;
    let requested_spec_index = requested_index.max(1);
    specializations::specs_for_class(class_id).nth((requested_spec_index - 1) as usize)
}

fn active_player_spec(state: &LuaState) -> Option<&'static specializations::SpecInfo> {
    let (class_id, active_spec_index) = {
        let sim = borrow_state(state).ok()?;
        (sim.player.class_index as u32, sim.player.active_spec_index)
    };
    let active_spec_index = active_spec_index.max(1);
    specializations::specs_for_class(class_id).nth((active_spec_index - 1) as usize)
}

fn push_specialization_info(state: &mut LuaState, spec: &specializations::SpecInfo) {
    push_specialization_identity(state, spec);
    state.push(Val::Num(spec.primary_stat as f64));
    state.push(Val::Num(0.0));
    state.push(Val::Nil);
    state.push(Val::Num(0.0));
    state.push(Val::Bool(true));
}

fn push_class_specialization_info(state: &mut LuaState, spec: &specializations::SpecInfo) {
    push_specialization_identity(state, spec);
    state.push(Val::Bool(false));
    state.push(Val::Bool(true));
    state.push(Val::Nil);
    state.push(Val::Nil);
}

fn push_specialization_identity(state: &mut LuaState, spec: &specializations::SpecInfo) {
    let spec_name = create_string(state, spec.name);
    let spec_description = create_string(state, spec.description);
    let spec_role = create_string(state, spec.role);
    state.push(Val::Num(spec.id as f64));
    state.push(spec_name);
    state.push(spec_description);
    state.push(Val::Num(spec.icon_file_data_id as f64));
    state.push(spec_role);
}

fn spec_display_spell_ids(spec_id: i32) -> Option<Vec<u32>> {
    for skill_line_index in 1..=spellbook_data::num_skill_lines() {
        let skill_line = spellbook_data::get_skill_line(skill_line_index)?;
        if skill_line.spec_id == Some(spec_id) {
            return Some(
                skill_line
                    .spells
                    .iter()
                    .map(|entry| entry.spell_id)
                    .collect(),
            );
        }
    }
    None
}

fn role_enum_value(role: &str) -> f64 {
    match role {
        "TANK" => 0.0,
        "HEALER" => 1.0,
        _ => 2.0,
    }
}

pub fn player_get_timerunning_season_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?.timerunning_season_id.unwrap_or(0);
    state.push(Val::Num(id as f64));
    Ok(1)
}

pub fn player_is_timerunning(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.timerunning_season_id.is_some();
    state.push(Val::Bool(active));
    Ok(1)
}

fn ui_widget_container_get_num_widgets_showing(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let count = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(frame_id)
            .map(|frame| {
                frame
                    .children
                    .iter()
                    .filter(|&&child_id| {
                        sim.widgets
                            .get(child_id)
                            .map(|child| child.visible)
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0) as f64
    };
    state.push(Val::Num(count));
    Ok(1)
}

fn ui_widget_container_on_load(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
