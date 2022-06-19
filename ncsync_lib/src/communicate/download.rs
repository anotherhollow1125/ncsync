use crate::communicate::get;
use crate::path::AsNCUrl;
use crate::setting::ClientHub;
use anyhow::Result;
use bytes::Bytes;
use reqwest::Method;
use std::path::Path;

pub async fn download(
    profile_name: &str,
    client_hub: &ClientHub,
    path: impl AsRef<Path>,
) -> Result<Bytes> {
    let ref client = client_hub.get_client(profile_name)?;

    let entry = get(client, path.as_ref()).await?;

    if entry.is_dir() {
        return Err(anyhow!("Not a file: {}", path.as_ref().display()));
    }

    let url = path.as_ref().as_nc_url(client)?;
    let res = client.get_request_builder(Method::GET, url)?.send().await?;

    Ok(res.bytes().await?)
}

pub async fn download_with_progress() -> Result<()> {
    todo!()
}
