use anyhow::Result;
// use once_cell::sync::Lazy;
use crate::errors::NcsError::*;
use regex::Regex;
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use uuid::Uuid;

pub mod readwrite;

const NC_ROOT_PREFIX: &str = "/remote.php/dav/files/";
pub const OCS_ROOT: &str = "/ocs/v2.php/apps/activity/api/v2/activity/all";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "info")]
pub enum LoginStatus {
    NotYet,
    Polling {
        token: String,
        end_point: String,
    },
    LoggedIn {
        host: String,
        username: String,
        password: String,
    },
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub login_status: LoginStatus,
}

impl Profile {
    pub fn new(name: String) -> Self {
        let login_status = LoginStatus::NotYet;
        Self { name, login_status }
    }

    pub fn load(name: String, login_status: LoginStatus) -> Self {
        Self { name, login_status }
    }

    fn make_root_prefix(username: &str) -> String {
        let tmp = format!("{}{}", NC_ROOT_PREFIX, username);
        fix_root(&tmp)
    }

    pub fn get_root_prefix(&self) -> Result<String> {
        let username = match &self.login_status {
            LoginStatus::LoggedIn { username, .. } => username.clone(),
            _ => return Err(NotLoggedIn.into()),
        };
        let root_prefix = Profile::make_root_prefix(&username);

        Ok(root_prefix)
    }

    pub fn get_authinfo(&self) -> Option<(String, String, String)> {
        match &self.login_status {
            LoginStatus::LoggedIn {
                host: _,
                username,
                password,
            } => {
                let root_prefix = Profile::make_root_prefix(username);
                Some((username.clone(), password.clone(), root_prefix))
            }
            _ => return None,
        }
    }

    pub fn get_host(&self) -> Result<Option<Url>> {
        let host = match self.login_status {
            LoginStatus::LoggedIn { ref host, .. } => host,
            _ => return Ok(None),
        };

        let host = fix_host(&host);
        let host = Url::parse(&host)?;

        Ok(Some(host))
    }

    pub fn get_request_builder(
        &self,
        method: Method,
        url: Url,
        req_client: &reqwest::Client,
    ) -> Result<reqwest::RequestBuilder> {
        let (username, password) = match &self.login_status {
            LoginStatus::LoggedIn {
                username, password, ..
            } => (username, password),
            _ => return Err(anyhow!("not logged in")),
        };

        Ok(req_client
            .request(method, url)
            .basic_auth(username, Some(password)))
    }
}

#[derive(Debug)]
pub struct ClientHub {
    client_id: String,
    profiles: HashMap<String, Profile>,
    default_profile: Option<String>,
    req_client: reqwest::Client,
}

impl ClientHub {
    pub fn new() -> Result<Self> {
        let client_id = Uuid::new_v4().to_string();
        let req_client = Self::client_builder(&client_id).build()?;
        let client_hub = Self {
            profiles: HashMap::new(),
            default_profile: None,
            client_id,
            req_client,
        };
        Ok(client_hub)
    }

    pub fn load_without_profiles(client_id: String) -> Result<Self> {
        let req_client = Self::client_builder(&client_id).build()?;
        let client_hub = Self {
            profiles: HashMap::new(),
            default_profile: None,
            client_id,
            req_client,
        };
        Ok(client_hub)
    }

    pub fn add_profile(&mut self, name: String, login_status: LoginStatus) -> Result<()> {
        if self.profiles.contains_key(&name) {
            return Err(anyhow!("profile already exists"));
        }

        if self.profiles.is_empty() {
            self.default_profile = Some(name.clone());
        }

        self.profiles
            .insert(name.clone(), Profile::load(name, login_status));
        Ok(())
    }

    pub fn new_with_auth(username: String, password: String, host: String) -> Result<Self> {
        // let host = fix_host(&host);
        // let host = Url::parse(&host)?;
        let client_id = Uuid::new_v4().to_string();
        let profile = Profile::load(
            username.clone(),
            LoginStatus::LoggedIn {
                host,
                username: username.clone(),
                password,
            },
        );
        let default_profile = Some(username.clone());
        let profiles = vec![(username, profile)].into_iter().collect();
        Ok(Self {
            client_id: client_id.clone(),
            req_client: Self::client_builder(&client_id).build()?,
            profiles,
            default_profile,
        })
    }

    pub(crate) fn client_builder(client_id: &str) -> reqwest::ClientBuilder {
        let mut headers = header::HeaderMap::new();
        headers.insert("OCS-APIRequest", header::HeaderValue::from_static("true"));
        let builder = reqwest::ClientBuilder::new();
        let name = format!(
            "ncsync_v{}_{}",
            env!("CARGO_PKG_VERSION"),
            if client_id.len() >= 10 {
                &client_id[0..10]
            } else {
                client_id
            }
        );
        builder.user_agent(&name).default_headers(headers)
    }

    pub fn get_reqclient(&self) -> &reqwest::Client {
        &self.req_client
    }

    pub fn new_reqclient(&self, _proxy: Option<String>) -> Result<reqwest::Client> {
        Ok(Self::client_builder(&self.client_id).build()?)
    }

    pub fn get_all_profiles(&self) -> Vec<&Profile> {
        self.profiles.values().collect::<Vec<_>>()
    }

    pub fn get_profile(&self, profile_name: &str) -> Option<&Profile> {
        self.profiles.get(profile_name)
    }

    pub fn get_mut_profile(&mut self, profile_name: &str) -> Option<&mut Profile> {
        self.profiles.get_mut(profile_name)
    }

    pub(crate) fn get_default_profile(&self) -> Result<Option<&Profile>> {
        match self.default_profile {
            Some(ref profile_name) => {
                let res = self
                    .profiles
                    .get(profile_name)
                    .ok_or_else(|| InvalidProfile)?;
                Ok(Some(res))
            }
            None => Ok(None),
        }
    }

    pub(crate) fn get_mut_default_profile(&mut self) -> Result<Option<&mut Profile>> {
        match self.default_profile {
            Some(ref profile_name) => {
                let res = self
                    .profiles
                    .get_mut(profile_name)
                    .ok_or_else(|| InvalidProfile)?;
                Ok(Some(res))
            }
            None => Ok(None),
        }
    }

    pub(crate) fn get_client(&self, profile_name: &str) -> Result<Client> {
        let profile = self
            .get_profile(profile_name)
            .ok_or_else(|| ProfileNotFound(profile_name.to_string()))?;
        let client = Client {
            client_hub: self,
            profile,
        };
        Ok(client)
    }

    pub(crate) fn get_mut_client(&mut self, profile_name: &str) -> Result<ClientMut> {
        match self.get_profile(profile_name) {
            Some(_) => (),
            None => return Err(ProfileNotFound(profile_name.to_string()).into()),
        }
        let client = ClientMut {
            client_hub: self,
            profile_name: profile_name.to_string(),
        };
        Ok(client)
    }
}

// target profile name でClientを固定した後のClient。
// 入れ子にすることもできちゃうことを考えるとアンチパターン...?

pub(crate) struct Client<'a> {
    client_hub: &'a ClientHub,
    profile: &'a Profile,
}

impl<'a> Client<'a> {
    /*
    pub fn get_id(&'a self) -> &'a str {
        &self.client_hub.client_id
    }*/

    pub fn get_reqclient(&'a self) -> &'a reqwest::Client {
        &self.client_hub.req_client
    }

    /*
    pub fn new_reqclient(&'a self, proxy: Option<String>) -> Result<reqwest::Client> {
        self.client_hub.new_reqclient(proxy)
    }
    */

    pub fn get_request_builder(&self, method: Method, url: Url) -> Result<reqwest::RequestBuilder> {
        let (username, password) = match &self.profile.login_status {
            LoginStatus::LoggedIn {
                username, password, ..
            } => (username, password),
            _ => return Err(NotLoggedIn.into()),
        };
        let req_client = self.get_reqclient();

        Ok(req_client
            .request(method, url)
            .basic_auth(username, Some(password)))
    }

    pub fn get_root_prefix(&self) -> Result<String> {
        self.profile.get_root_prefix()
    }

    pub fn get_host(&self) -> Result<Option<Url>> {
        self.profile.get_host()
    }
}

pub(crate) struct ClientMut<'a> {
    pub client_hub: &'a mut ClientHub,
    profile_name: String,
}

impl<'a> ClientMut<'a> {
    pub fn get_profile(&self) -> &Profile {
        self.client_hub.get_profile(&self.profile_name).unwrap()
    }

    pub fn get_mut_profile(&mut self) -> &mut Profile {
        self.client_hub.get_mut_profile(&self.profile_name).unwrap()
    }
}

#[derive(Debug)]
pub struct LocalInfo {
    excludes: ExcludeList,
}

use reqwest::header;

impl LocalInfo {
    pub fn new(excludes: ExcludeList) -> Self {
        Self { excludes }
    }

    pub fn get_exclude_list(&self) -> &ExcludeList {
        &self.excludes
    }
}

/*
pub fn add_head_slash(s: &str) -> String {
    if RE_HAS_HEAD_SLASH.is_match(s) {
        s.to_string()
    } else {
        format!("/{}", s)
    }
}

pub fn add_last_slash(s: &str) -> String {
    if RE_HAS_LAST_SLASH.is_match(s) {
        s.to_string()
    } else {
        format!("{}/", s)
    }
}
*/

fn fix_host(host: &str) -> &str {
    host.trim_end_matches("/")
}

fn fix_root(root_prefix: &str) -> String {
    let root_prefix = root_prefix.trim_end_matches("/");

    let root_prefix = if root_prefix.starts_with("/") {
        root_prefix.to_string()
    } else {
        format!("/{}", root_prefix)
    };

    root_prefix
}

use globset::{Glob, GlobMatcher};

#[derive(Debug, Clone, Default)]
pub struct ExcludePaths {
    blacks: Vec<GlobMatcher>,
    pub(crate) original_blacks: Vec<PathBuf>,
    whites: Vec<GlobMatcher>,
    pub(crate) original_whites: Vec<PathBuf>,
}

impl ExcludePaths {
    pub fn new(original_blacks: Vec<PathBuf>, original_whites: Vec<PathBuf>) -> Self {
        let blacks = original_blacks
            .iter()
            .filter_map(|p| {
                let s = p.to_string_lossy();
                Glob::new(&s).ok().map(|g| g.compile_matcher())
            })
            .collect();
        let whites = original_whites
            .iter()
            .filter_map(|p| {
                let s = p.to_string_lossy();
                Glob::new(&s).ok().map(|g| g.compile_matcher())
            })
            .collect();
        Self {
            original_blacks,
            blacks,
            original_whites,
            whites,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExcludeRegexes {
    blacks: Vec<Regex>,
    pub(crate) original_blacks: Vec<String>,
    whites: Vec<Regex>,
    pub(crate) original_whites: Vec<String>,
}

impl ExcludeRegexes {
    pub fn new(original_blacks: Vec<String>, original_whites: Vec<String>) -> Self {
        let mut blacks = original_blacks
            .iter()
            .filter_map(|s| Regex::new(s).ok())
            .collect::<Vec<_>>();
        blacks.push(Regex::new(r"^\.").unwrap());
        blacks.push(Regex::new(r"^~").unwrap());
        let whites = original_whites
            .iter()
            .filter_map(|s| Regex::new(s).ok())
            .collect::<Vec<_>>();
        Self {
            blacks,
            original_blacks,
            whites,
            original_whites,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExcludeList {
    pub(crate) paths: ExcludePaths,
    pub(crate) regexes: ExcludeRegexes,
}

use std::path::Path;

impl ExcludeList {
    pub fn new(
        blackpaths: Vec<PathBuf>,
        whitepaths: Vec<PathBuf>,
        blackregexes: Vec<String>,
        whiteregexes: Vec<String>,
    ) -> Self {
        Self {
            paths: ExcludePaths::new(blackpaths, whitepaths),
            regexes: ExcludeRegexes::new(blackregexes, whiteregexes),
        }
    }

    pub fn judge(&self, p: impl AsRef<Path>) -> bool {
        let path = p.as_ref();

        // path white > path black > regex white > regex black

        for g in &self.paths.whites {
            if g.is_match(path) {
                return true;
            }

            let mut p = path;
            while let Some(parent) = p.parent() {
                if g.is_match(parent) {
                    return true;
                }

                p = parent;
            }
        }

        for g in &self.paths.blacks {
            if g.is_match(path) {
                return false;
            }

            let mut p = path;
            while let Some(parent) = p.parent() {
                if g.is_match(parent) {
                    return false;
                }

                p = parent;
            }
        }

        'compcheck: for c in path.components() {
            let s = c.as_os_str().to_string_lossy();

            for r in self.regexes.whites.iter() {
                if r.is_match(&s) {
                    continue 'compcheck;
                }
            }

            for r in self.regexes.blacks.iter() {
                if r.is_match(&s) {
                    return false;
                }
            }
        }

        true
    }
}

use std::path::PathBuf;

#[derive(Debug)]
pub struct CurDirSetting {
    pub current_dir: PathBuf,
}

impl CurDirSetting {
    pub fn new(current_dir: PathBuf) -> Self {
        Self { current_dir }
    }
}

#[derive(Debug)]
pub struct ContextSetting {
    pub curdir_setting: CurDirSetting,
    pub proxy: Option<String>,
}
