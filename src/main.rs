use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use clap::Parser;
use itertools::Itertools;
use notes::Note;
use replace::{
    replace_center_tag, replace_curly_braces, replace_gt_in_quote, replace_latex, replace_links,
    replace_md_to_html, replace_width,
};
use resources::extract_resources;
use rust_embed::RustEmbed;

#[macro_use]
extern crate lazy_static;

mod notes;
mod replace;
mod resources;
mod utils;

#[derive(RustEmbed)]
#[folder = "www"]
struct WWW;

#[derive(Parser)]
struct Config {
    #[arg(long)]
    joplin_data_path: Option<PathBuf>,

    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    pdf: bool,

    #[arg(long, default_value_t = false)]
    clean: bool,

    #[arg(long, default_value_t = false)]
    print_out_dir: bool,
}

fn main() -> Result<()> {
    let config = Config::parse();
    let joplin_data_path = config.joplin_data_path.unwrap_or(
        dirs::config_dir()
            .ok_or(anyhow!("Config directory not found"))?
            .join("joplin-desktop"),
    );
    let joplin_db_path = joplin_data_path.join("database.sqlite");
    let joplin_resources_path = joplin_data_path.join("resources");
    let out_dir = config.output.unwrap_or(PathBuf::from("output"));

    if config.clean {
        match fs::remove_dir_all(&out_dir) {
            Ok(()) => Ok(()),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => Ok(()),
                _ => Err(e),
            },
        }?;
    }

    fs::create_dir_all(&out_dir)?;
    fs::write(out_dir.join(".gitignore"), "*")?;

    let out_resources_dir = out_dir.join("resources");
    fs::create_dir_all(&out_resources_dir)?;

    let resources = extract_resources(&joplin_db_path)?;
    for resource_path in resources.values() {
        let original_path = joplin_resources_path.join(resource_path);
        let out_path = out_resources_dir.join(resource_path);

        fs::copy(original_path, out_path)?;
    }

    for asset in WWW::iter() {
        fs::write(
            out_dir.join(asset.to_string()),
            WWW::get(&asset)
                .ok_or(anyhow!("Asset {asset} not found"))?
                .data,
        )?;
    }

    let mut notes_html = String::new();

    let notes_out_dir = out_dir.join("notes");
    let notebooks = Note::extract(&joplin_db_path)?;
    let sorted_notebooks = notebooks.iter().sorted_by(|a, b| {
        let (a, _) = *a;
        let (b, _) = *b;
        Ord::cmp(&a.to_string_lossy(), &b.to_string_lossy())
    });

    for (notebook_path, notes) in sorted_notebooks {
        let full_notebook_path = notes_out_dir.join(notebook_path);
        fs::create_dir_all(&full_notebook_path)?;

        notes_html += format!("<li> {} <ul> \n", &notebook_path.to_string_lossy()).as_str();

        let mut notes = (*notes).clone();
        notes.sort_by(|a, b| Ord::cmp(&a.title, &b.title));

        for note in notes {
            let html_name = format!("{}.html", note.title);
            notes_html += format!(
                "<li> <a href=\"/notes/{}/{}\">{}</a> </li>\n",
                &notebook_path.to_string_lossy(),
                &html_name,
                &note.title
            )
            .as_str();

            let md_path = full_notebook_path.join(format!("{}.md", note.title));

            let replaced = replace_links(&note, &resources, &notebooks);
            let replaced = replace_width(replaced);
            let replaced = replace_center_tag(replaced);
            let replaced = replace_latex(replaced, false);
            fs::write(md_path, &replaced)?;

            let replaced = replace_curly_braces(replaced);
            let replaced = replace_md_to_html(replaced);

            let html_path = full_notebook_path.join(html_name);
            let html_content = markdown::to_html_with_options(
                &replaced,
                &markdown::Options {
                    parse: markdown::ParseOptions::gfm(),
                    compile: markdown::CompileOptions {
                        allow_dangerous_html: true,
                        allow_dangerous_protocol: true,
                        ..markdown::CompileOptions::default()
                    },
                },
            )
            .map_err(|x: String| anyhow!(x))?;

            let replaced = replace_latex(html_content, true);
            let replaced = replace_gt_in_quote(replaced);

            fs::write(
                html_path,
                format!(
                    r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <link rel="stylesheet" href="/styles.css"/>

        <!-- Katex -->
        <!-- TODO: Remove the use of CDNs and make this local -->
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
    <a href="/">Back</a>
        {}
    </body>
</html>
"#,
                    replaced
                ),
            )?;
        }

        notes_html += "</ul> </li> \n";
    }

    fs::write(
        out_dir.join("index.html"),
        format!(
            r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <meta charset="utf-8">
                    <link rel="stylesheet" href="/styles.css"/>
                </head>
            
                <body>
                    <ul>
                        {}
                    </ul>
                </body>
            </html>
            
    "#,
            notes_html
        ),
    )?;

    if config.print_out_dir {
        println!("{}", out_dir.to_string_lossy())
    }

    Ok(())
}
