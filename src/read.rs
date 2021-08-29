use std::convert::TryInto;

pub fn byte(loadstate: &mut &[u8], err: String) -> Result<u8, String> {
  match loadstate.split_first() {
    Some((first, rest)) => {
      *loadstate = rest;
      Ok(*first)
    },
    None => Err(err)
  }
}

pub fn i32(loadstate: &mut &[u8], err: String) -> Result<i32, String> {
    if loadstate.len() >= 4 {
        let (i32_bytes, rest) = loadstate.split_at(4);
        *loadstate = rest;
        Ok(i32::from_ne_bytes(i32_bytes.try_into().unwrap()))
    } else {
        Err(err)
    }
}

pub fn u32(loadstate: &mut &[u8], err: String) -> Result<u32, String> {
    if loadstate.len() >= 4 {
        let (u32_bytes, rest) = loadstate.split_at(4);
        *loadstate = rest;
        Ok(u32::from_ne_bytes(u32_bytes.try_into().unwrap()))
    } else {
        Err(err)
    }
}

pub fn u64(loadstate: &mut &[u8], err: String) -> Result<u64, String> {
    if loadstate.len() >= 8 {
        let (u64_bytes, rest) = loadstate.split_at(8);
        *loadstate = rest;
        Ok(u64::from_ne_bytes(u64_bytes.try_into().unwrap()))
    } else {
        Err(err)
    }
}

pub fn f64(loadstate: &mut &[u8], err: String) -> Result<f64, String> {
    if loadstate.len() >= 8 {
        let (f64_bytes, rest) = loadstate.split_at(8);
        *loadstate = rest;
        Ok(f64::from_ne_bytes(f64_bytes.try_into().unwrap()))
    } else {
        Err(err)
    }
}

pub fn bytes<'a>(loadstate: &'a mut &[u8], len: usize, err: String) -> Result<&'a [u8], String> {
    println!("read_bytes {} of {}", len, loadstate.len());
    if loadstate.len() >= len {
        let (bytes, rest) = loadstate.split_at(len);
        *loadstate = rest;
        Ok(bytes)
    } else {
        Err(err)
    }
}
