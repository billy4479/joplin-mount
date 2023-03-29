use std::{collections::HashMap, mem, path::PathBuf};

use anyhow::Result;

use crate::utils::{as_u32_le, hex_to_u32};

struct Folder {
    name: String,
    parent: Option<u32>,
}

type Folders = HashMap<u32, Folder>;

impl Folder {
    pub fn get_path_with_parent(&self, folders: &Folders) -> PathBuf {
        match self.parent {
            Some(parent_id) => {
                let parent = folders.get(&parent_id).expect("");
                PathBuf::from(parent.get_path_with_parent(folders)).join(self.name.clone())
            }
            None => PathBuf::from(self.name.clone()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Note {
    pub id: u32,
    pub folder_path: PathBuf,
    pub title: String,
    pub content: String,
}

impl Note {
    pub(crate) fn extract(db: &PathBuf) -> Result<Vec<Note>> {
        let connection = sqlite::open(db)?;

        let mut folders = Folders::new();

        connection.iterate("SELECT id,title,parent_id FROM folders", |row| {
            let id = hex_to_u32(row[0].1.expect("")).unwrap();
            let name = row[1].1.expect("").to_owned();
            let parent = row[2].1.expect("");
            let parent = if parent.is_empty() {
                None
            } else {
                Some(as_u32_le(&hex::decode(parent).unwrap()[..]))
            };

            folders.insert(id, Folder { name, parent });

            true
        })?;

        let folder_and_path: HashMap<u32, (&Folder, PathBuf)> = folders
            .iter()
            .map(|(id, folder)| {
                let path = folder.get_path_with_parent(&folders);
                (*id, (folder, path))
            })
            .collect();

        let mut result = Vec::<Note>::new();
        connection.iterate("SELECT title,body,parent_id,id FROM notes", |row| {
            let title = row[0].1.expect("");
            let content = row[1].1.expect("");
            let parent_id = hex_to_u32(row[2].1.expect("")).unwrap();
            let (_, path) = folder_and_path.get(&parent_id).expect("");
            let id = hex_to_u32(row[3].1.expect("")).unwrap();

            result.push(Note {
                id,
                folder_path: path.clone(),
                title: title.to_owned(),
                content: content.to_owned(),
            });

            true
        })?;

        Ok(result)
    }
}
