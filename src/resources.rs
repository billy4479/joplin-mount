use anyhow::Result;
use std::{collections::HashMap, path::PathBuf};

use crate::utils::{hex_to_id, ID};

#[derive(Debug)]
pub(crate) struct Resource {
    pub name: String,
    pub path: PathBuf,
    pub mime_type: String,
}

impl Resource {
    pub(crate) fn extract(db_path: &PathBuf) -> Result<HashMap<ID, Resource>> {
        let mut result = HashMap::<ID, Resource>::new();
        let connection = sqlite::open(db_path)?;

        connection.iterate(
            "SELECT id,title,mime,file_extension FROM resources",
            |row| {
                let id_str = row[0].1.expect("");
                let id = hex_to_id(id_str).unwrap();
                let title = row[1].1.expect("");
                let mime = row[2].1.expect("");
                let extension = row[3].1.expect("");

                result.insert(
                    id,
                    Resource {
                        name: title.to_owned(),
                        mime_type: mime.to_owned(),
                        path: PathBuf::from(format!("{id_str}.{extension}")),
                    },
                );

                true
            },
        )?;

        Ok(result)
    }
}
