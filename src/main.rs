use std::{fs, path::PathBuf};

use anyhow::Result;
use notes::Note;
use replace_links::replace_links;
use resources::Resource;

#[macro_use]
extern crate lazy_static;

mod notes;
mod replace_links;
mod resources;
mod utils;

fn main() -> Result<()> {
    let out_dir = PathBuf::from("out");

    let joplin_data_path = dirs::config_dir()
        .expect("Missing config folder")
        .join("joplin-desktop");
    let db_path = joplin_data_path.clone().join("database.sqlite");
    let resources_path = joplin_data_path.clone().join("resources");

    fs::create_dir_all(out_dir.join("resources"))?;

    let mut resources = Resource::extract(&db_path)?;
    for resource in &mut resources {
        let original_path = resources_path.clone().join(&resource.1.path);
        let out_path = PathBuf::from("resources").join(&resource.1.path);
        resource.1.path = out_path.clone();

        fs::copy(original_path, out_dir.join(out_path))?;
    }

    let notes_dir = out_dir.clone().join("notes");
    let notes = Note::extract(&db_path)?;
    for note in &notes {
        let mut path = notes_dir.clone().join(&note.folder_path);
        fs::create_dir_all(&path)?;
        path.push(format!("{}.md", note.title));

        let replaced = replace_links(&note, &resources, &notes);
        fs::write(path, replaced)?;
    }

    Ok(())
}
