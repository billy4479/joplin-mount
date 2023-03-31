use std::{fs, path::PathBuf};

use anyhow::Result;
use notes::Note;
use replace::{
    replace_curly_braces, replace_latex, replace_links, replace_md_to_html, replace_width,
};
use resources::Resource;

#[macro_use]
extern crate lazy_static;

mod notes;
mod replace;
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
        let path = notes_dir.clone().join(&note.folder_path);
        fs::create_dir_all(&path)?;
        let md_path = path.join(format!("{}.md", note.title));

        let replaced = replace_links(&note, &resources, &notes);
        let replaced = replace_width(replaced);
        fs::write(md_path, &replaced)?;

        let replaced = replace_latex(replaced, false);
        let replaced = replace_curly_braces(replaced);
        let replaced = replace_md_to_html(replaced);

        let html_path = path.join(format!("{}.html", note.title));
        let html_content = markdown::to_html_with_options(
            &replaced,
            &markdown::Options {
                parse: markdown::ParseOptions::mdx(),
                compile: markdown::CompileOptions {
                    allow_dangerous_html: true,
                    allow_dangerous_protocol: true,
                    ..markdown::CompileOptions::default()
                },
            },
        )
        .unwrap();
        let replaced = replace_latex(html_content, true);

        fs::write(
            html_path,
            format!(
                r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">

        <!-- Katex -->
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/katex.min.css" integrity="sha384-vKruj+a13U8yHIkAyGgK1J3ArTLzrFGBbBc0tDp4ad/EyewESeXE/Iv67Aj8gKZ0" crossorigin="anonymous">
        <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/katex.min.js" integrity="sha384-PwRUT/YqbnEjkZO0zZxNqcxACrXe+j766U2amXcgMg5457rve2Y7I6ZJSm2A0mS4" crossorigin="anonymous"></script>
        <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/contrib/mhchem.min.js" integrity="sha384-RTN08a0AXIioPBcVosEqPUfKK+rPp+h1x/izR7xMkdMyuwkcZCWdxO+RSwIFtJXN"  crossorigin="anonymous"></script>
        <script src="https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/contrib/copy-tex.min.js" integrity="sha384-ww/583aHhxWkz5DEVn6OKtNiIaLi2iBRNZXfJRiY1Ai7tnJ9UXpEsyvOITVpTl4A" crossorigin="anonymous"></script>
        <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/contrib/auto-render.min.js" integrity="sha384-+VBxd3r6XgURycqtZ117nYw44OOcIax56Z4dCRWbxyPt0Koah1uHoK0o4+/RRE05" crossorigin="anonymous"></script>
        <script>
            document.addEventListener("DOMContentLoaded", function() {{
                    renderMathInElement(document.body, {{
                    // customised options
                    // • auto-render specific keys, e.g.:
                    delimiters: [
                        {{left: '$$', right: '$$', display: true}},
                        {{left: '$', right: '$', display: false}},
                        {{left: '\\(', right: '\\)', display: false}},
                        {{left: '\\[', right: '\\]', display: true}}
                    ],
                    // • rendering keys, e.g.:
                    throwOnError: true
                }});
            }});
        </script>
    </head>

    <body>
        {}
    </body>
</html>
"#,
                replaced
            ),
        )?;

        // println!("{:?}", note.folder_path.join(&note.title));
    }

    Ok(())
}
