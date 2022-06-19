use crate::errors::NcsError::*;
use crate::setting::Client;
use crate::setting::{ClientHub, Profile};
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Url;
use std::path::{Path, PathBuf};

pub(crate) trait AsNCUrl {
    fn as_nc_url(&self, client: &Client<'_>) -> Result<Url>;
}

// もう少しバリデーションを追加する必要が出てくるかもしれない
impl AsNCUrl for Path {
    fn as_nc_url(&self, client: &Client<'_>) -> Result<Url> {
        let input = self.to_string_lossy().replace("\\", "/");
        let input = input.trim_start_matches("/");
        let prefix = client
            .get_root_prefix()
            .context("Cloud Not Get Prefix! Did you login?")?;
        let prefix = Path::new(&prefix);
        let path = prefix.join(input);
        let host = client
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

pub struct NCPath<'a> {
    path: PathBuf,
    profile: &'a Profile,
}

static RE_PATHRESOLVE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?P<username>[^:]*):(?P<path>.*)$").unwrap());

impl<'a> NCPath<'a> {
    pub fn new(path: impl AsRef<Path>, profile: &'a Profile) -> Self {
        let path = path.as_ref().to_owned();
        Self { path, profile }
    }

    pub fn get_username(target: &str) -> Option<String> {
        let caps = RE_PATHRESOLVE.captures(target);

        caps.map(|c| c["username"].to_string())
    }

    pub fn resolve(
        hub: &'a ClientHub,
        target: &str,
        use_default: bool, // this argument is fool proof.
    ) -> Result<Self> {
        let caps = RE_PATHRESOLVE.captures(target);

        let (profile, path) = match caps {
            Some(caps) => {
                let username = &caps["username"];
                let profile = hub
                    .get_profile(username)
                    .ok_or_else(|| ProfileNotFound(username.to_string()))?;
                let path = caps["path"].to_string();
                (profile, path)
            }
            None => {
                if_chain! {
                    if use_default;
                    if let Some(profile) = hub.get_default_profile()?;
                    then {
                        (profile, target.to_string())
                    }
                    else {
                        return Err(anyhow!("There is no default user @ NCPath::resolve: {}", target));
                    }
                }
            }
        };

        Ok(Self::new(path, profile))
    }
}
