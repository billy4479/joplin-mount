use std::{collections::HashMap, mem, path::PathBuf};

use anyhow::Result;

fn as_u32_le(array: &[u8]) -> u32 {
    ((array[0] as u32) << 0)
        + ((array[1] as u32) << 8)
        + ((array[2] as u32) << 16)
        + ((array[3] as u32) << 24)
}

struct Folder {
    name: String,
    parent: Option<u32>,
}

type Folders = HashMap<u32, Folder>;

fn get_path_with_parent(folder: &Folder, folders: &Folders) -> PathBuf {
    match folder.parent {
        Some(parent_id) => {
            let parent = folders.get(&parent_id).expect("");
            let parent_path = match parent.parent {
                Some(_) => get_path_with_parent(parent, folders),
                None => PathBuf::from(parent.name.clone()),
            };

            let mut path = PathBuf::from(parent_path);
            path.push(folder.name.clone());

            path
        }
        None => PathBuf::from(folder.name.clone()),
    }
}

pub(crate) struct Note {
    pub folder_path: PathBuf,
    pub title: String,
    pub content: String,
}

pub(crate) fn extract_notes(db: PathBuf) -> Result<Vec<Note>> {
    let connection = sqlite::open(db)?;

    let mut folders = Folders::new();

    connection.iterate("SELECT id,title,parent_id FROM folders", |row| {
        let id = as_u32_le(&hex::decode(row[0].1.expect("")).unwrap()[..]);
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

    let mut folder_and_path: HashMap<u32, (&Folder, PathBuf)> = folders
        .iter()
        .map(|(id, folder)| {
            let path = get_path_with_parent(folder, &folders);
            (*id, (folder, path))
        })
        .collect();

    let mut result = Vec::<Note>::new();
    connection.iterate("SELECT title,body,parent_id FROM notes", |row| {
        let title = row[0].1.expect("");
        let content = row[1].1.expect("");
        let parent_id = as_u32_le(&hex::decode(row[2].1.expect("")).unwrap()[..]);
        let (_, path) = folder_and_path.get_mut(&parent_id).expect("");

        result.push(Note {
            folder_path: mem::take(path),
            title: title.to_owned(),
            content: content.to_owned(),
        });

        true
    })?;

    Ok(result)
}
