//! C_TableUtil, tCompare, and tInvert implementations.

use crate::lua_api::methods::table_get_static;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaError, LuaResult, RuntimeError, Val};

/// tInvert(t) — return a new table with keys/values swapped.
///
/// `tInvert(tbl)` — return `{[v] = k for k, v in pairs(tbl)}`.
///
/// Mirrors Blizzard_SharedXMLBase/TableUtil.lua. Implemented in Rust
/// because the stub version (which pushed `nil`) shadows the Lua
/// definition — `EnumUtil.MakeEnum(...)` goes through this and every
/// downstream enum (ObjectiveTrackerModuleState, etc.) ends up nil
/// when the stub wins.
pub fn t_invert(state: &mut LuaState) -> LuaResult<u32> {
    let input = stack_val(state, 1);
    let Val::Table(tbl_ref) = input else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let (array_values, hash_entries) = collect_table_entries(state, tbl_ref);
    let inverted_ref = state.gc.alloc_table(Table::new());
    insert_inverted_array(state, inverted_ref, array_values);
    insert_inverted_hash(state, inverted_ref, hash_entries);
    state.push(Val::Table(inverted_ref));
    Ok(1)
}

/// tCompare(lhsTable, rhsTable [, depth]) — compare table entries recursively.
pub fn t_compare(state: &mut LuaState) -> LuaResult<u32> {
    let left = stack_val(state, 1);
    let right = stack_val(state, 2);
    let depth = match stack_val(state, 3) {
        // Lua's `depth = depth or 1`: nil and false both select the default,
        // while other values are deliberately left untouched until a nested
        // table comparison evaluates `depth > 1`.
        Val::Nil | Val::Bool(false) => Val::Num(1.0),
        value => value,
    };
    let (Val::Table(left_ref), Val::Table(right_ref)) = (left, right) else {
        let argument = if !matches!(left, Val::Table(_)) { 1 } else { 2 };
        let value = if argument == 1 { left } else { right };
        return Err(LuaError::Runtime(RuntimeError {
            message: format!(
                "bad argument #{argument} to 'tCompare' (table expected, got {})",
                value.type_name()
            ),
            level: 0,
            traceback: vec![],
        }));
    };
    let result = compare_tables(state, left_ref, right_ref, depth)?;
    state.push(Val::Bool(result));
    Ok(1)
}

fn compare_tables(
    state: &mut LuaState,
    left_ref: rilua::vm::gc::arena::GcRef<Table>,
    right_ref: rilua::vm::gc::arena::GcRef<Table>,
    depth: Val,
) -> LuaResult<bool> {
    let left_entries = state
        .gc
        .tables
        .get(left_ref)
        .map(|table| {
            let mut entries = Vec::new();
            let mut key = Val::Nil;
            while let Some((next_key, value)) = table.next(key, &state.gc.string_arena)? {
                entries.push((next_key, value));
                key = next_key;
            }
            Ok::<_, LuaError>(entries)
        })
        .transpose()?
        .unwrap_or_default();

    for (key, left_value) in left_entries {
        let right_value = lua_gettable(state, Val::Table(right_ref), key)?;
        if !compare_values(state, left_value, right_value, depth)? {
            return Ok(false);
        }
    }

    let right_keys = state
        .gc
        .tables
        .get(right_ref)
        .map(|table| {
            let mut keys = Vec::new();
            let mut key = Val::Nil;
            while let Some((next_key, _)) = table.next(key, &state.gc.string_arena)? {
                keys.push(next_key);
                key = next_key;
            }
            Ok::<_, LuaError>(keys)
        })
        .transpose()?
        .unwrap_or_default();
    for key in right_keys {
        if lua_gettable(state, Val::Table(left_ref), key)?.is_nil() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn compare_values(state: &mut LuaState, left: Val, right: Val, depth: Val) -> LuaResult<bool> {
    match (left, right) {
        // At shallow depth, tables compare equal by type, not identity.
        (Val::Table(left_ref), Val::Table(right_ref)) => {
            if depth_greater_than_one(state, depth)? {
                let next_depth = decrement_depth(state, depth)?;
                compare_tables(state, left_ref, right_ref, next_depth)
            } else {
                Ok(true)
            }
        }
        (Val::Table(_), _) | (_, Val::Table(_)) => Ok(false),
        // Lua ~= is the negation of Lua equality, including shared __eq.
        (left, right) => state.api_equal(left, right),
    }
}

fn depth_greater_than_one(state: &mut LuaState, depth: Val) -> LuaResult<bool> {
    if let Some(metamethod) = value_metamethod(state, depth, b"__lt")? {
        if matches!(metamethod, Val::Function(_)) {
            return Ok(call_lua_function(state, metamethod, &[Val::Num(1.0), depth])?.is_truthy());
        }
    }
    state.api_lessthan(Val::Num(1.0), depth)
}

fn lua_gettable(state: &mut LuaState, table: Val, key: Val) -> LuaResult<Val> {
    let mut current = table;
    for _ in 0..100 {
        let Val::Table(table_ref) = current else {
            return Err(index_error(current));
        };
        let result = state
            .gc
            .tables
            .get(table_ref)
            .map_or(Val::Nil, |table| table.get(key, &state.gc.string_arena));
        if !result.is_nil() {
            return Ok(result);
        }
        let Some(metamethod) = table_metamethod(state, table_ref, b"__index")? else {
            return Ok(Val::Nil);
        };
        if matches!(metamethod, Val::Function(_)) {
            return call_lua_function(state, metamethod, &[current, key]);
        }
        current = metamethod;
    }
    Err(runtime_error("loop in gettable"))
}

fn decrement_depth(state: &mut LuaState, depth: Val) -> LuaResult<Val> {
    if let Val::Num(value) = depth {
        return Ok(Val::Num(value - 1.0));
    }
    let Some(metamethod) = value_metamethod(state, depth, b"__sub")? else {
        return Err(arithmetic_error(depth));
    };
    if !matches!(metamethod, Val::Function(_)) {
        return Err(arithmetic_error(depth));
    }
    call_lua_function(state, metamethod, &[depth, Val::Num(1.0)])
}

fn value_metamethod(state: &mut LuaState, value: Val, name: &[u8]) -> LuaResult<Option<Val>> {
    let Val::Table(table_ref) = value else {
        return Ok(None);
    };
    table_metamethod(state, table_ref, name)
}

fn table_metamethod(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<Table>,
    name: &[u8],
) -> LuaResult<Option<Val>> {
    let Some(table) = state.gc.tables.get(table_ref) else {
        return Err(runtime_error("invalid table reference"));
    };
    let Some(metatable_ref) = table.metatable() else {
        return Ok(None);
    };
    let key = state.gc.intern_string(name);
    Ok(state.gc.tables.get(metatable_ref).and_then(|mt| {
        let value = mt.get(Val::Str(key), &state.gc.string_arena);
        (!value.is_nil()).then_some(value)
    }))
}

fn index_error(value: Val) -> LuaError {
    runtime_error(&format!("attempt to index a {} value", value.type_name()))
}

fn call_lua_function(state: &mut LuaState, function: Val, args: &[Val]) -> LuaResult<Val> {
    let saved_top = state.top;
    let base = state.top;
    state.ensure_stack(base + args.len() + 2);
    state.stack_set(base, function);
    for (index, arg) in args.iter().copied().enumerate() {
        state.stack_set(base + index + 1, arg);
    }
    state.top = base + args.len() + 1;
    state.call_function(base, 1)?;
    let result = state.stack_get(base);
    state.top = saved_top;
    Ok(result)
}

fn arithmetic_error(value: Val) -> LuaError {
    runtime_error(&format!(
        "attempt to perform arithmetic on a {} value",
        value.type_name()
    ))
}

fn runtime_error(message: &str) -> LuaError {
    LuaError::Runtime(RuntimeError {
        message: message.to_string(),
        level: 0,
        traceback: vec![],
    })
}

fn collect_table_entries(
    state: &LuaState,
    tbl_ref: rilua::vm::gc::arena::GcRef<Table>,
) -> (Vec<Val>, Vec<(Val, Val)>) {
    let array_values = state
        .gc
        .tables
        .get(tbl_ref)
        .map(|t| t.array_slice().to_vec())
        .unwrap_or_default();
    let hash_entries = state
        .gc
        .tables
        .get(tbl_ref)
        .map(|t| t.hash_entries())
        .unwrap_or_default();
    (array_values, hash_entries)
}

fn insert_inverted_array(
    state: &mut LuaState,
    inverted_ref: rilua::vm::gc::arena::GcRef<Table>,
    array_values: Vec<Val>,
) {
    for (index, value) in array_values.into_iter().enumerate() {
        if matches!(value, Val::Nil) {
            continue;
        }
        let key = Val::Num((index + 1) as f64);
        if let Some(t) = state.gc.tables.get_mut(inverted_ref) {
            let _ = t.raw_set(value, key, &state.gc.string_arena);
        }
        state.gc.barrier_back(inverted_ref);
    }
}

fn insert_inverted_hash(
    state: &mut LuaState,
    inverted_ref: rilua::vm::gc::arena::GcRef<Table>,
    hash_entries: Vec<(Val, Val)>,
) {
    for (key, value) in hash_entries {
        if matches!(value, Val::Nil) {
            continue;
        }
        if let Some(t) = state.gc.tables.get_mut(inverted_ref) {
            let _ = t.raw_set(value, key, &state.gc.string_arena);
        }
        state.gc.barrier_back(inverted_ref);
    }
}

/// TableUtil.FindIndexedMismatch(left, right [, comparator]) — first index where arrays differ.
pub fn table_util_find_indexed_mismatch(state: &mut LuaState) -> LuaResult<u32> {
    let left = stack_val(state, 1);
    let right = stack_val(state, 2);
    let comparator = stack_val(state, 3);
    let (Val::Table(left_ref), Val::Table(right_ref)) = (left, right) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let left_values = copy_array_slice(state, left_ref);
    let right_values = copy_array_slice(state, right_ref);
    let mismatch = find_first_mismatch(state, &left_values, &right_values, comparator)?;
    match mismatch {
        Some(index) => state.push(Val::Num(index as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

/// table.count(tbl) — count total entries and array-index entries.
pub fn table_count(state: &mut LuaState) -> LuaResult<u32> {
    let input = stack_val(state, 1);
    if matches!(input, Val::Nil) {
        return Err(table_count_nil_error());
    }

    let Val::Table(table_ref) = input else {
        return Ok(0);
    };

    let counts = count_table_entries(state, table_ref)?;
    state.push(Val::Num(counts.total_nodes as f64));
    state.push(Val::Num(counts.array_nodes as f64));
    state.push(Val::Num(counts.max_array_index as f64));
    Ok(3)
}

fn table_count_nil_error() -> LuaError {
    LuaError::Runtime(RuntimeError {
        message: "bad argument #1 to 'count' (table expected, got nil)".into(),
        level: 0,
        traceback: vec![],
    })
}

#[derive(Default)]
struct TableCounts {
    total_nodes: usize,
    array_nodes: usize,
    max_array_index: usize,
}

fn count_table_entries(
    state: &LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<Table>,
) -> LuaResult<TableCounts> {
    let Some(table) = state.gc.tables.get(table_ref) else {
        return Ok(TableCounts::default());
    };

    let mut counts = TableCounts::default();
    let mut key = Val::Nil;
    while let Some((next_key, _)) = table.next(key, &state.gc.string_arena)? {
        counts.total_nodes += 1;
        count_array_key(next_key, &mut counts);
        key = next_key;
    }
    Ok(counts)
}

fn count_array_key(key: Val, counts: &mut TableCounts) {
    let Val::Num(number) = key else {
        return;
    };
    let array_index = number as usize;
    if array_index == 0 || array_index as f64 != number {
        return;
    }
    counts.array_nodes += 1;
    counts.max_array_index = counts.max_array_index.max(array_index);
}

fn copy_array_slice(
    state: &LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> Vec<Val> {
    state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default()
}

fn find_first_mismatch(
    state: &mut LuaState,
    left: &[Val],
    right: &[Val],
    comparator: Val,
) -> LuaResult<Option<usize>> {
    let count = left.len().max(right.len());
    for index in 0..count {
        let left_val = left.get(index).copied().unwrap_or(Val::Nil);
        let right_val = right.get(index).copied().unwrap_or(Val::Nil);
        let equal = if matches!(comparator, Val::Function(_)) {
            call_table_util_comparator(state, comparator, left_val, right_val, index + 1)?
        } else {
            left_val == right_val
        };
        if !equal {
            return Ok(Some(index + 1));
        }
    }
    Ok(None)
}

fn call_table_util_comparator(
    state: &mut LuaState,
    comparator: Val,
    left: Val,
    right: Val,
    index: usize,
) -> LuaResult<bool> {
    let call_base = state.top;
    state.ensure_stack(call_base + 5);
    state.stack_set(call_base, comparator);
    state.stack_set(call_base + 1, left);
    state.stack_set(call_base + 2, right);
    state.stack_set(call_base + 3, Val::Num(index as f64));
    state.top = call_base + 4;
    state.call_function(call_base, 1)?;
    let result = state.stack_get(call_base);
    state.top = call_base;
    Ok(!matches!(result, Val::Nil | Val::Bool(false)))
}

pub fn register_table_util(state: &mut LuaState) -> LuaResult<()> {
    register_table_library_extensions(state)?;

    let table_ref = state.gc.alloc_table(Table::new());
    table_set_rust_fn_static(
        state,
        table_ref,
        "FindIndexedMismatch",
        table_util_find_indexed_mismatch,
    )?;
    let key_ref = state.gc.intern_string(b"C_TableUtil");
    let global_ref = state.global;
    if let Some(global) = state.gc.tables.get_mut(global_ref) {
        let _ = global.raw_set(
            Val::Str(key_ref),
            Val::Table(table_ref),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(global_ref);
    Ok(())
}

fn register_table_library_extensions(state: &mut LuaState) -> LuaResult<()> {
    let table_library = table_get_static(state, Val::Table(state.global), "table");
    let Val::Table(table_ref) = table_library else {
        return Ok(());
    };
    table_set_rust_fn_static(state, table_ref, "count", table_count)
}
