use anyhow::Result;
use fantoccini::ClientBuilder;
use std::path::Path;

pub(crate) async fn create_pdfs(outdir: &Path, webdriver_url: &str) -> Result<()> {
    let c = ClientBuilder::native().connect(webdriver_url).await?;

    Ok(())
}
