use anyhow::Result;
use std::{collections::HashMap, path::PathBuf};

use crate::utils::{hex_to_id, ID};

pub(crate) fn extract_resources(db_path: &PathBuf) -> Result<HashMap<ID, PathBuf>> {
    let mut result = HashMap::<ID, PathBuf>::new();
    let connection = sqlite::open(db_path)?;

    connection.iterate("SELECT id,file_extension FROM resources", |row| {
        let id_str = row[0].1.expect("");
        let id = hex_to_id(id_str).unwrap();
        let extension = row[1].1.expect("");

        result.insert(id, PathBuf::from(format!("{id_str}.{extension}")));

        true
    })?;

    Ok(result)
}
