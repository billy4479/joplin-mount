use anyhow::Result;
use headless_chrome::{types::PrintToPdfOptions, Browser, LaunchOptions};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub(crate) fn create_pdfs(outdir: &PathBuf) -> Result<()> {
    let handle =
        tokio::spawn(warp::serve(warp::fs::dir(outdir.to_owned())).run(([127, 0, 0, 1], 4479)));

    defer! {
        handle.abort();
    }

    let browser = Browser::new(LaunchOptions {
        headless: true,
        ..Default::default()
    })?;
    let tab = browser.new_tab()?;

    let pdf_dir = outdir.join("pdf");
    let outdir_string = outdir.to_str().expect("");
    fs::create_dir_all(&pdf_dir)?;

    for entry in WalkDir::new(outdir) {
        let entry = entry?;
        if entry.path().extension().is_some_and(|x| x != "html")
            || !entry.file_type().is_file()
            || entry.file_name().to_string_lossy().starts_with('.')
        {
            continue;
        }

        let url = format!(
            "http://localhost:4479{}",
            entry.path().to_string_lossy().replace(outdir_string, "")
        );

        let out_file_path = {
            let e = entry
                .path()
                .parent()
                .map(|x| x.to_string_lossy().replace("/notes/", "/pdf/"))
                .unwrap();

            fs::create_dir_all(&e)?;

            let result = Path::new(&e).join(
                entry
                    .path()
                    .file_name()
                    .map(|x| x.to_string_lossy().replace(".html", ".pdf"))
                    .unwrap(),
            );
            println!("{}", result.display());
            result
        };

        tab.navigate_to(url.as_str())?;
        tab.wait_until_navigated()?;
        let pdf = tab.print_to_pdf(Some(PrintToPdfOptions {
            print_background: Some(true),
            ..Default::default()
        }))?;
        fs::write(out_file_path, pdf)?;
    }

    Ok(())
}
