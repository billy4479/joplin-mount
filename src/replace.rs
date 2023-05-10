use std::{collections::HashMap, path::PathBuf};

use regex::{Captures, Regex};

use crate::{
    notes::{Note, Notebook},
    utils::{hex_to_id, ID},
};

pub(crate) fn replace_links(
    note: &Note,
    resources: &HashMap<ID, PathBuf>,
    notebooks: &Notebook,
) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r":/([0-9a-f]{32})").unwrap();
    }
    let replaced = RE.replace_all(&note.content, |caps: &Captures| {
        let reference_id = hex_to_id(&caps[1]).unwrap();
        let resource = resources.get(&reference_id);
        let substitution = match resource {
            Some(resource) => String::from("/") + resource.to_str().expect(""),
            None => {
                for (notebook_path, notes) in notebooks {
                    for note in notes {
                        if note.id == reference_id {
                            let note_path = notebook_path
                                .join(&note.title)
                                .to_string_lossy()
                                .replace(' ', "%20");
                            return format!("/notes/{}.md", note_path);
                        }
                    }
                }

                format!("RESOURCE NOT FOUND: {}", hex::encode(reference_id))
            }
        };

        substitution
    });
    replaced.to_string()
}

pub(crate) fn replace_md_to_html(content: String) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\[[\w\* ]*\]\([\w/%\d]*.(md)\)").unwrap();
    }

    RE.replace_all(&content, |caps: &Captures| {
        caps[0].replace(&caps[1], "html")
    })
    .to_string()
}

pub(crate) fn replace_width(content: String) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"width=(\d+)").unwrap();
    }

    let replaced = RE.replace_all(content.as_str(), |caps: &Captures| {
        format!("width=\"{}\"", &caps[1])
    });

    replaced.to_string()
}

pub(crate) fn replace_curly_braces(content: String) -> String {
    content.replace('{', "\\{").replace('}', "\\}")
}

pub(crate) fn replace_center_tag(content: String) -> String {
    content
        .replace("<center>", "<div style=\"text-align:center\">")
        .replace("</center>", "</div>")
}

pub(crate) fn replace_latex(content: String, after_html: bool) -> String {
    let mut iter = content.chars().peekable();
    let mut result = String::new();
    result.reserve(content.len());

    let mut is_in_latex = false;
    let mut is_inline = false;

    while let Some(c) = iter.next() {
        if c == '$' {
            if is_in_latex {
                // Ending
                if is_inline {
                    is_in_latex = false;
                    is_inline = false;
                } else if let Some(peek) = iter.peek() {
                    if *peek == '$' {
                        is_in_latex = false;
                        if after_html {
                            result.push('\n');
                        }
                        result.push(*peek);
                        iter.next();
                    } else {
                        is_inline = true;
                    }
                }
            } else {
                // Start
                if let Some(peek) = iter.peek() {
                    match *peek {
                        '$' => {
                            is_inline = false;
                            is_in_latex = true;
                            if after_html {
                                result.push('\n');
                            }
                            result.push(*peek);
                            iter.next();
                        }
                        _ => {
                            is_inline = true;
                            is_in_latex = false;
                        }
                    }
                }
            }
        }

        if is_in_latex && !after_html {
            match c {
                '\n' => result.push(' '),
                '\\' => result += "\\\\",
                '<' => result += "\\lt ",
                '>' => result += "\\gt ",
                '_' => result += "\\_",
                _ => result.push(c),
            }
        } else {
            result.push(c);
        }
    }

    result
}
pub(crate) fn replace_gt_in_quote(content: String) -> String {
    let lines = content.split('\n');
    let mut is_in_quote = false;
    let pattern = "\\gt";

    let mut result = String::new();

    for line in lines {
        if line == "<blockquote>" {
            is_in_quote = true
        } else if line == "</blockquote>" {
            is_in_quote = false
        }
        let trimmed = line.trim_end();

        if is_in_quote && trimmed.ends_with(pattern) {
            result += &trimmed[..trimmed.len() - pattern.len()]
        } else {
            result += line;
        }
        result.push('\n')
    }

    result
}
