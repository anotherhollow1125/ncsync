use crate::communicate::get;
use crate::path::AsNCUrl;
use crate::setting::NCInfo;
use anyhow::Result;
use bytes::Bytes;
use reqwest::Method;
use std::path::Path;

pub async fn download(nc_info: &NCInfo, path: impl AsRef<Path>) -> Result<Bytes> {
    let entry = get(nc_info, path.as_ref()).await?;

    if entry.is_dir() {
        return Err(anyhow!("Not a file: {}", path.as_ref().display()));
    }

    let url = path.as_ref().as_nc_url(nc_info)?;
    let res = nc_info
        .get_request_builder(Method::GET, url)?
        .send()
        .await?;

    Ok(res.bytes().await?)
}

pub async fn download_with_progress() -> Result<()> {
    todo!()
}
