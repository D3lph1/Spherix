use std::fmt::Write;

use anyhow::anyhow;

pub fn decode(s: &str) -> Result<Vec<u8>, anyhow::Error> {
    (0..s.len())
        .step_by(2)
        .map(
            |i| u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| anyhow!("at index {i}: {e}"))
        )
        .collect()
}

pub fn encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

pub fn decode_stringed_hex(hex: &str) -> Result<Vec<u8>, anyhow::Error> {
    let hex = hex.replace("\n", "")
        .replace("\t", "")
        .replace(" ", "");

    decode(&hex)
}
