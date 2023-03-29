use anyhow::Result;

pub(crate) fn as_u32_le(array: &[u8]) -> u32 {
    ((array[0] as u32) << 0)
        + ((array[1] as u32) << 8)
        + ((array[2] as u32) << 16)
        + ((array[3] as u32) << 24)
}

pub(crate) fn hex_to_u32(hex_string: &str) -> Result<u32> {
    Ok(as_u32_le(&hex::decode(hex_string)?[..]))
}
