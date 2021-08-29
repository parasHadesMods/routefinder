use std::convert::TryInto;
use rlua::{Value, Context};
const LUABINS_CNIL: u8 = 0x2D;
const LUABINS_CFALSE: u8 = 0x30;
const LUABINS_CTRUE: u8 = 0x31;
const LUABINS_CNUMBER: u8 = 0x4E;
const LUABINS_CSTRING: u8 = 0x53 ;
const LUABINS_CTABLE: u8 = 0x54;

fn refine<'a>(s: &'a str, n: &'static str) -> String {
  s.to_owned() + " " + n
}

fn read_byte(loadstate: &mut &[u8], err: String) -> Result<u8, String> {
  match loadstate.split_first() {
    Some((first, rest)) => {
      *loadstate = rest;
      Ok(*first)
    },
    None => Err(err)
  }
}

fn read_i32(loadstate: &mut &[u8], err: String) -> Result<i32, String> {
    if loadstate.len() >= 4 {
        let (i32_bytes, rest) = loadstate.split_at(4);
        *loadstate = rest;
        Ok(i32::from_ne_bytes(i32_bytes.try_into().unwrap()))
    } else {
        Err(err)
    }
}

fn read_u32(loadstate: &mut &[u8], err: String) -> Result<u32, String> {
    if loadstate.len() >= 4 {
        let (u32_bytes, rest) = loadstate.split_at(4);
        *loadstate = rest;
        Ok(u32::from_ne_bytes(u32_bytes.try_into().unwrap()))
    } else {
        Err(err)
    }
}

fn read_f64(loadstate: &mut &[u8], err: String) -> Result<f64, String> {
    if loadstate.len() >= 8 {
        let (f64_bytes, rest) = loadstate.split_at(8);
        *loadstate = rest;
        Ok(f64::from_ne_bytes(f64_bytes.try_into().unwrap()))
    } else {
        Err(err)
    }
}

fn read_bytes<'a>(loadstate: &'a mut &[u8], len: usize, err: String) -> Result<&'a [u8], String> {
    if loadstate.len() >= len {
        let (bytes, rest) = loadstate.split_at(len);
        *loadstate = rest;
        Ok(bytes)
    } else {
        Err(err)
    }
}

fn load_number(loadstate: &mut &[u8], err: String) -> Result<rlua::Number, String> {
  Ok(read_f64(loadstate, err)?)
}

fn load_string<'lua>(loadstate: &mut &[u8], context: Context<'lua>, err: String) -> Result<rlua::String<'lua>, String> {
  let len = read_u32(loadstate, refine(&err, "size"))?;
  let str_bytes = read_bytes(loadstate, len.try_into().unwrap(), refine(&err, "string"))?;
  match context.create_string(str_bytes) {
    Ok(r) => Ok(r),
    Err(_) => Err(refine(&err, "create"))
  }
}

fn load_value<'a>(loadstate: &mut &[u8], context: Context<'a>, err: String) -> Result<Value<'a>, String> {
  let tbyte = read_byte(loadstate, refine(&err, "type"))?;
  match tbyte {
    LUABINS_CNIL => Ok(Value::Nil),
    LUABINS_CFALSE => Ok(Value::Boolean(false)),   
    LUABINS_CTRUE => Ok(Value::Boolean(true)),   
    LUABINS_CNUMBER => Ok(Value::Number(load_number(loadstate, refine(&err, "number"))?)),
    LUABINS_CSTRING => Ok(Value::String(load_string(loadstate, context, refine(&err, "string"))?)),
    LUABINS_CTABLE => Ok(Value::Table(load_table(loadstate, context, refine(&err, "table"))?)),
    _ => Err(refine(&err, "type mismatch"))
  }
}

fn load_table<'lua>(loadstate: &mut &[u8], context: Context<'lua>, err: String) -> Result<rlua::Table<'lua>, String> {
   let array_size = read_i32(loadstate, refine(&err, "array_size"))?;
   let hash_size = read_i32(loadstate, refine(&err, "hash_size"))?;
   let total_size = array_size + hash_size;
   let table: rlua::Table<'lua> = match context.create_table() {
     Ok(t) => Ok(t),
     Err(_) => Err(refine(&err, "create"))
   }?;

   for i in 1..total_size {
     let key = load_value(loadstate, context, refine(&err, "key"))?;
     let value = load_value(loadstate, context, refine(&err, "value"))?;
     match table.set(key, value) {
       Ok(_) => Ok(()),
       Err(_) => Err(refine(&err, "set"))
     }?;
   }
   Ok(table)
}

fn load<'lua>(loadstate: &mut &[u8], context: Context<'lua>, err: String) -> Result<Vec<Value<'lua>>, String> {
    let num_items = read_byte(loadstate, refine(&err, "num_items"))?;
    let mut vec = Vec::new();
    for i in 1..num_items {
        let value = load_value(loadstate, context, refine(&err, "load"))?;
        vec.push(value);
    }
    Ok(vec)
}
