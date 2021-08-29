use std::convert::TryInto;
const LUABINS_CNIL: u8 = 0x2D;
const LUABINS_CFALSE: u8 = 0x30;
const LUABINS_CTRUE: u8 = 0x31;
const LUABINS_CNUMBER: u8 = 0x4E;
const LUABINS_CSTRING: u8 = 0x53 ;
const LUABINS_CTABLE: u8 = 0x54;

fn refine<'a>(s: &'a str, n: &'static str) -> String {
  s.to_owned() + n
}

fn read_byte(loadstate: &mut &[u8], context: String) -> Result<u8, String> {
  match loadstate.split_first() {
    Some((first, rest)) => {
      *loadstate = rest;
      Ok(*first)
    },
    None => Err(context)
  }
}

fn read_lint<'a>(loadstate: &mut &[u8], context: String) -> Result<i32, String> {
  if loadstate.len() >= 4 {
    let (int_bytes, rest) = loadstate.split_at(4);
    *loadstate = rest;
    Ok(i32::from_ne_bytes(int_bytes.try_into().unwrap()))
  } else {
    Err(context)
  }
}

fn load_nil<'a>(loadstate: &mut &[u8], context: String) -> Result<(), String> {
  Ok(())
}

fn load_value<'a>(loadstate: &mut &[u8], context: String) -> Result<(), String> {
  let tbyte = read_byte(loadstate, refine(&context, " type"))?;
  match tbyte {
    LUABINS_CNIL => load_nil(loadstate, refine(&context, " nil")),
    LUABINS_CFALSE => Ok(()),   
    LUABINS_CTRUE => Ok(()),   
    LUABINS_CNUMBER => Ok(()),   
    LUABINS_CSTRING => Ok(()),   
    LUABINS_CTABLE => load_table(loadstate, refine(&context, "table")),
    _ => Err(refine(&context, " type mismatch"))
  }
}

fn load_table(loadstate: &mut &[u8], context: String) -> Result<(), String> {
   let array_size = read_lint(loadstate, refine(&context, " array_size"))?;
   let hash_size = read_lint(loadstate, refine(&context, "hash_size"))?;
   let total_size = array_size + hash_size;

   for i in 1..total_size {
   }
   Ok(())
}
