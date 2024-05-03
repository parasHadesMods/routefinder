use super::read;
use std::convert::TryInto;

pub trait UncompressedSize {
    const UNCOMPRESSED_SIZE: i32;
}

pub struct HadesSaveV16 {
    pub version: u32,
    pub timestamp: u64,
    pub location: String,
    pub runs: u32,
    pub active_meta_points: u32,
    pub active_shrine_points: u32,
    pub god_mode_enabled: bool,
    pub hell_mode_enabled: bool,
    pub lua_keys: Vec<String>,
    pub current_map_name: String,
    pub start_next_map: String,
    pub lua_state_lz4: Vec<u8>,
}

impl UncompressedSize for HadesSaveV16 {
    const UNCOMPRESSED_SIZE: i32 = 9388032;
}

fn refine<'a>(s: &'a str, n: &'static str) -> String {
    s.to_owned() + " " + n
}

fn string(loadstate: &mut &[u8], err: String) -> Result<String, String> {
    let size = read::u32(loadstate, refine(&err, "size"))?;
    let str_bytes = read::bytes(loadstate, size.try_into().unwrap(), refine(&err, "bytes"))?;
    match String::from_utf8(str_bytes.to_vec()) {
        Ok(s) => Ok(s),
        Err(_) => Err(refine(&err, "utf8")),
    }
}

pub fn read(loadstate: &mut &[u8], err: String) -> Result<HadesSaveV16, String> {
    let signature = read::bytes(loadstate, 4, refine(&err, "signature"))?;
    if signature != "SGB1".as_bytes() {
        return Err("Not a Hades save file".to_string());
    }
    let _checksum = read::bytes(loadstate, 4, refine(&err, "checksum"))?;
    let version = read::u32(loadstate, refine(&err, "version"))?;
    if version != 16 {
        return Err("unknown version".to_string());
    };
    let timestamp = read::u64(loadstate, refine(&err, "timestamp"))?;
    let location = string(loadstate, refine(&err, "location"))?;
    let runs = read::u32(loadstate, refine(&err, "runs"))?;
    let active_meta_points = read::u32(loadstate, refine(&err, "active_meta_points"))?;
    let active_shrine_points = read::u32(loadstate, refine(&err, "active_shrine_points"))?;
    let god_mode_enabled = read::byte(loadstate, refine(&err, "god_mode_enabled"))? != 0;
    let hell_mode_enabled = read::byte(loadstate, refine(&err, "hell_mode_enabled"))? != 0;

    let mut lua_keys = Vec::new();
    let size = read::u32(loadstate, refine(&err, "lua_keys size"))?;
    for _ in 0..size {
        let lua_key = string(loadstate, refine(&err, "lua_keys"))?;
        lua_keys.push(lua_key);
    }

    let current_map_name = string(loadstate, refine(&err, "current_map_name"))?;
    let start_next_map = string(loadstate, refine(&err, "start_next_map"))?;
    let lua_state_size = read::u32(loadstate, refine(&err, "lua_state size"))?;
    let lua_state_lz4 = read::bytes(
        loadstate,
        lua_state_size.try_into().unwrap(),
        refine(&err, "lua_state bytes"),
    )?;

    Ok(HadesSaveV16 {
        version: version,
        timestamp: timestamp,
        location: location,
        runs: runs,
        active_meta_points: active_meta_points,
        active_shrine_points: active_shrine_points,
        god_mode_enabled: god_mode_enabled,
        hell_mode_enabled: hell_mode_enabled,
        lua_keys: lua_keys,
        current_map_name: current_map_name,
        start_next_map: start_next_map,
        lua_state_lz4: lua_state_lz4.to_vec(),
    })
}
