use std::{collections::HashMap, fs};

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

fn get_path_with_parent(folder: &Folder, folders: &Folders) -> String {
    match folder.parent {
        Some(parent_id) => {
            let parent = folders.get(&parent_id).expect("");
            let parent_path = match parent.parent {
                Some(_) => get_path_with_parent(parent, folders),
                None => parent.name.clone(),
            };
            format!("{parent_path}/{}", folder.name)
        }
        None => folder.name.clone(),
    }
}

fn main() -> Result<()> {
    fs::create_dir_all("out")?;

    let connection = sqlite::open("/home/billy/.config/joplin-desktop/database.sqlite")?;

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

    let folder_and_path: HashMap<u32, (&Folder, String)> = folders
        .iter()
        .map(|(id, folder)| {
            let path = get_path_with_parent(folder, &folders);
            fs::create_dir_all(format!("out/{}", &path)).unwrap();
            (*id, (folder, path))
        })
        .collect();

    connection.iterate("SELECT title,body,parent_id FROM notes", |row| {
        let title = row[0].1.expect("");
        let content = row[1].1.expect("");
        let parent_id = as_u32_le(&hex::decode(row[2].1.expect("")).unwrap()[..]);
        let (_, path) = folder_and_path.get(&parent_id).expect("");

        fs::write(format!("out/{path}/{title}.md"), content).unwrap();

        true
    })?;

    Ok(())
}
