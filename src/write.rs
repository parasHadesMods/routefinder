
pub fn byte(vec: &mut Vec<u8>, byte: u8) {
    vec.push(byte)
}

pub fn i32(vec: &mut Vec<u8>, i: i32) {
    let mut i32_bytes = i32::to_ne_bytes(i);
    vec.extend_from_slice(&mut i32_bytes)
}

pub fn u32(vec: &mut Vec<u8>, u: u32) {
    let mut u32_bytes = u32::to_ne_bytes(u);
    vec.extend_from_slice(&mut u32_bytes)
}

pub fn u64(vec: &mut Vec<u8>, u: u64) {
    let mut u64_bytes = u64::to_ne_bytes(u);
    vec.extend_from_slice(&mut u64_bytes)
}

pub fn f64(vec: &mut Vec<u8>, f: f64) {
    let mut f64_bytes = f64::to_ne_bytes(f);
    vec.extend_from_slice(&mut f64_bytes)
}

pub fn bytes(vec: &mut Vec<u8>, bytes: &[u8]) {
    vec.extend_from_slice(bytes)
}