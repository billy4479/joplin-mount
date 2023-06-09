use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;

use crate::utils::{hex_to_id, ID};

struct Folder {
    name: String,
    parent: Option<ID>,
}

type Folders = HashMap<ID, Folder>;

impl Folder {
    pub fn get_path_with_parent(&self, folders: &Folders) -> PathBuf {
        match self.parent {
            Some(parent_id) => {
                let parent = folders.get(&parent_id).expect("");
                parent.get_path_with_parent(folders).join(self.name.clone())
            }
            None => PathBuf::from(self.name.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Note {
    pub id: ID,
    pub title: String,
    pub content: String,
}

pub(crate) type Notebook = HashMap<PathBuf, Vec<Note>>;

impl Note {
    pub(crate) fn extract(db: &PathBuf) -> Result<Notebook> {
        let connection = sqlite::open(db)?;

        let mut folders = Folders::new();

        connection.iterate("SELECT id,title,parent_id FROM folders", |row| {
            let id = hex_to_id(row[0].1.expect("")).unwrap();
            let name = row[1].1.expect("").to_owned();
            let parent = row[2].1.expect("");
            let parent = if parent.is_empty() {
                None
            } else {
                Some(hex_to_id(parent).unwrap())
            };

            folders.insert(id, Folder { name, parent });

            true
        })?;

        let folder_and_path: HashMap<ID, (&Folder, PathBuf)> = folders
            .iter()
            .map(|(id, folder)| {
                let path = folder.get_path_with_parent(&folders);
                (*id, (folder, path))
            })
            .collect();

        let mut result = Notebook::new();
        connection.iterate("SELECT title,body,parent_id,id FROM notes", |row| {
            let title = row[0].1.expect("").replace('/', ".");
            let content = row[1].1.expect("");
            let parent_id = hex_to_id(row[2].1.expect("")).unwrap();
            let (_, path) = folder_and_path.get(&parent_id).expect("");
            let id = hex_to_id(row[3].1.expect("")).unwrap();

            let note = Note {
                id,
                title,
                content: content.to_owned(),
            };

            match result.get_mut(path) {
                Some(x) => {
                    x.push(note);
                }
                None => {
                    result.insert(path.to_owned(), vec![note]);
                }
            }

            true
        })?;

        Ok(result)
    }
}
