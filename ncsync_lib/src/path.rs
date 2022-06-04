use crate::setting::NCInfo;
use anyhow::{Context, Result};
use reqwest::Url;
use std::path::{Path, PathBuf};

pub trait AsNCUrl {
    fn as_nc_url(&self, nc_info: &NCInfo) -> Result<Url>;
}

// もう少しバリデーションを追加する必要が出てくるかもしれない
impl AsNCUrl for Path {
    fn as_nc_url(&self, nc_info: &NCInfo) -> Result<Url> {
        let input = self.to_string_lossy().replace("\\", "/");
        let input = input.trim_start_matches("/");
        let prefix = nc_info
            .get_root_prefix()
            .context("Cloud Not Get Prefix! Did you login?")?;
        let prefix = Path::new(&prefix);
        let path = prefix.join(input);
        let host = nc_info
            .get_host()?
            .context("Cloud Not Get Host! Did you login?")?;
        let res = host.join(&path.to_string_lossy())?;
        Ok(res)
    }
}

pub fn url2path(url: &str, prefix: &str) -> Result<PathBuf> {
    let path = url.strip_prefix(prefix).context("Invalid URL @ url2path")?;
    // let path = path.trim_start_matches("/");
    let path = path.trim_end_matches("/");

    let path = if path.starts_with("/") {
        path.to_string()
    } else {
        format!("/{}", path)
    };

    let path = PathBuf::from(path);
    Ok(path)
}

pub fn check_absolute(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.is_absolute()
}
