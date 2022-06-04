use crate::path::AsNCUrl;
use crate::setting::NCInfo;
use anyhow::Result;
use reqwest::Method;
use std::path::Path;

pub async fn upload(nc_info: &NCInfo, path: impl AsRef<Path>, bytes: Vec<u8>) -> Result<()> {
    let url = path.as_ref().as_nc_url(nc_info)?;
    let _ = nc_info
        .get_request_builder(Method::PUT, url)?
        .body(bytes)
        .send()
        .await?;

    Ok(())
}

pub async fn upload_with_progress() -> Result<()> {
    todo!()
}
