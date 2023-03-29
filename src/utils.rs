use anyhow::Result;

pub(crate) type ID = [u8; 16];

pub(crate) fn hex_to_id(hex_string: &str) -> Result<ID> {
    let decoded_vec = hex::decode(hex_string)?;
    let mut result: ID = [0u8; 16];
    for (i, v) in decoded_vec.iter().enumerate() {
        result[i] = *v
    }

    Ok(result)
}
