use std::{fs, path::PathBuf};

use anyhow::Result;
use notes::Note;
use regex::{Captures, Regex};
use resources::Resource;
use utils::hex_to_u32;

mod notes;
mod resources;
mod utils;

fn main() -> Result<()> {
    let out_dir = PathBuf::from("out");

    let joplin_data_path = dirs::config_dir()
        .expect("Missing config folder")
        .join("joplin-desktop");
    let db_path = joplin_data_path.clone().join("database.sqlite");
    let resources_path = joplin_data_path.clone().join("resources");

    let re = Regex::new(r":/([0-9a-f]{32})")?;

    let resources_dir = out_dir.clone().join("resources");
    fs::create_dir_all(&resources_dir)?;

    let mut resources = Resource::extract(&db_path)?;
    for resource in &mut resources {
        let original_path = resources_path.clone().join(&resource.1.path);
        let out_path = resources_dir.clone().join(&resource.1.path);

        fs::copy(original_path, &out_path)?;
        resource.1.path = out_path;
    }

    let notes_dir = out_dir.clone().join("notes");
    let notes = Note::extract(&db_path)?;
    for note in &notes {
        let mut path = notes_dir.clone().join(&note.folder_path);
        fs::create_dir_all(&path)?;
        path.push(format!("{}.md", note.title));

        // let mut resource_ids = Vec::<u32>::new();

        // println!("{}", &note.title);
        // {
        //     let captures = re.captures_iter(&note.content);
        //     for capture in captures {
        //         let id_hex = &capture[1];
        //         println!("{}", id_hex);
        //         let id = hex_to_u32(id_hex)?;
        //         resource_ids.push(id);
        //     }
        // }

        let mut i = 0;
        let replaced = re.replace_all(&note.content, |caps: &Captures| {
            let reference_id = hex_to_u32(&caps[1]).unwrap();
            let resource = resources.get(&reference_id);
            let substitution = match resource {
                Some(resource) => resource.path.to_str().expect("").to_owned(),
                None => {
                    let note_reference = notes.iter().find(|x| x.id == reference_id);
                    match note_reference {
                        Some(note_reference) => note_reference
                            .folder_path
                            .join(note_reference.title.clone())
                            .to_str()
                            .expect("")
                            .to_owned(),
                        None => {
                            return format!(
                                "RESOURCE NOT FOUND: {}",
                                hex::encode(reference_id.to_le_bytes())
                            )
                        }
                    }
                }
            };
            i += 1;

            substitution
        });

        fs::write(path, replaced.to_string())?;
    }

    Ok(())
}
