use std::collections::HashMap;

use regex::{Captures, Regex};

use crate::{
    notes::Note,
    resources::Resource,
    utils::{hex_to_id, ID},
};

pub(crate) fn replace_links(
    note: &Note,
    resources: &HashMap<ID, Resource>,
    notes: &Vec<Note>,
) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r":/([0-9a-f]{32})").unwrap();
    }
    let replaced = RE.replace_all(&note.content, |caps: &Captures| {
        let reference_id = hex_to_id(&caps[1]).unwrap();
        let resource = resources.get(&reference_id);
        let substitution = match resource {
            Some(resource) => String::from("/") + resource.path.to_str().expect(""),
            None => {
                let note_reference = notes.iter().find(|x| x.id == reference_id);
                match note_reference {
                    Some(note_reference) => {
                        let note_path = note_reference
                            .folder_path
                            .join(note_reference.title.clone())
                            .to_str()
                            .expect("")
                            .to_owned();

                        format!("/notes/{}.md", note_path)
                    }
                    None => return format!("RESOURCE NOT FOUND: {}", hex::encode(reference_id)),
                }
            }
        };

        substitution
    });
    replaced.to_string()
}
