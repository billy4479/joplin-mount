use std::{fs, path::PathBuf};

use anyhow::Result;
use extractor::extract_notes;

mod extractor;

fn main() -> Result<()> {
    let out_dir = PathBuf::from("out");
    fs::create_dir_all(&out_dir)?;

    let mut db_path = dirs::config_dir().expect("Missing config folder");
    db_path.push("joplin-desktop");
    db_path.push("database.sqlite");

    let notes = extract_notes(db_path)?;
    for note in notes {
        let mut path = out_dir.clone();
        path.push(note.folder_path);
        fs::create_dir_all(&path)?;
        path.push(format!("{}.md", note.title));

        fs::write(path, note.content)?;
    }

    Ok(())
}
