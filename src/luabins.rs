use super::read;

use mlua::{Lua, Value};
use std::convert::TryInto;

const LUABINS_CNIL: u8 = 0x2D;
const LUABINS_CFALSE: u8 = 0x30;
const LUABINS_CTRUE: u8 = 0x31;
const LUABINS_CNUMBER: u8 = 0x4E;
const LUABINS_CSTRING: u8 = 0x53;
const LUABINS_CTABLE: u8 = 0x54;

fn refine<'a>(s: &'a str, n: &'static str) -> String {
    s.to_owned() + " " + n
}

fn load_number<'lua>(loadstate: &mut &[u8], err: String) -> Result<mlua::Value<'lua>, String> {
    let float = read::f64(loadstate, err)?;
    if float.fract() == 0.0 {
        Ok(Value::Integer(float.trunc() as i64))
    } else {
        Ok(Value::Number(float))
    }
}

fn load_string<'lua>(
    lua: &'lua Lua,
    loadstate: &mut &[u8],
    err: String,
) -> Result<mlua::String<'lua>, String> {
    let len = read::u32(loadstate, refine(&err, "size"))?;
    let str_bytes = read::bytes(loadstate, len.try_into().unwrap(), refine(&err, "string"))?;

    match lua.create_string(str_bytes) {
        Ok(r) => Ok(r),
        Err(_) => Err(refine(&err, "create")),
    }
}

pub fn load_value<'lua>(
    lua: &'lua Lua,
    loadstate: &mut &[u8],
    err: String,
) -> Result<Value<'lua>, String> {
    let tbyte = read::byte(loadstate, refine(&err, "type"))?;
    match tbyte {
        LUABINS_CNIL => Ok(Value::Nil),
        LUABINS_CFALSE => Ok(Value::Boolean(false)),
        LUABINS_CTRUE => Ok(Value::Boolean(true)),
        LUABINS_CNUMBER => Ok(load_number(loadstate, refine(&err, "number"))?),
        LUABINS_CSTRING => Ok(Value::String(load_string(
            lua,
            loadstate,
            refine(&err, "string"),
        )?)),
        LUABINS_CTABLE => Ok(Value::Table(load_table(
            &lua,
            loadstate,
            refine(&err, "table"),
        )?)),
        _ => Err(refine(&err, "type mismatch")),
    }
}

fn load_table<'lua>(
    lua: &'lua Lua,
    loadstate: &mut &[u8],
    err: String,
) -> Result<mlua::Table<'lua>, String> {
    let array_size = read::i32(loadstate, refine(&err, "array_size"))?;
    let hash_size = read::i32(loadstate, refine(&err, "hash_size"))?;
    let total_size = array_size + hash_size;
    let table: mlua::Table<'lua> = match lua.create_table() {
        Ok(t) => Ok(t),
        Err(_) => Err(refine(&err, "create")),
    }?;

    for _ in 0..total_size {
        let key = load_value(lua, loadstate, refine(&err, "key"))?;
        let value = load_value(lua, loadstate, refine(&err, "value"))?;
        match table.set(key, value) {
            Ok(_) => Ok(()),
            Err(_) => Err(refine(&err, "set")),
        }?;
    }
    Ok(table)
}

pub fn load<'lua>(
    lua: &'lua Lua,
    loadstate: &mut &[u8],
    err: String,
) -> Result<Vec<Value<'lua>>, String> {
    let num_items = read::byte(loadstate, refine(&err, "num_items"))?;
    let mut vec = Vec::new();
    for _ in 0..num_items {
        let value = load_value(lua, loadstate, refine(&err, "load"))?;
        vec.push(value);
    }
    Ok(vec)
}
