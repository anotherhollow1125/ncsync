use crate::path::AsNCUrl;
use crate::setting::ClientHub;
use anyhow::Result;
use reqwest::Method;
use std::path::Path;

pub async fn upload(
    profile_name: &str,
    client_hub: &ClientHub,
    path: impl AsRef<Path>,
    bytes: Vec<u8>,
) -> Result<()> {
    let ref client = client_hub.get_client(profile_name)?;

    let url = path.as_ref().as_nc_url(client)?;
    let _ = client
        .get_request_builder(Method::PUT, url)?
        .body(bytes)
        .send()
        .await?;

    Ok(())
}

pub async fn upload_with_progress() -> Result<()> {
    todo!()
}
