use crate::setting::{ClientHub, ExcludeList, LocalInfo, LoginStatus};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
struct ProfileRaw {
    name: String,
    login_status: LoginStatus,
}

// ClientHubはclient_idを持たなければならないのでLocalInfoRawにDefaultを持たせてはいけない
#[derive(Debug, Serialize, Deserialize)]
struct ClientHubRaw {
    client_id: String,
    default_profile: Option<String>,
    profiles: Vec<ProfileRaw>,
}

impl ClientHubRaw {
    fn from(client_hub: &ClientHub) -> ClientHubRaw {
        ClientHubRaw {
            client_id: client_hub.client_id.clone(),
            profiles: client_hub
                .profiles
                .iter()
                .map(|p| ProfileRaw {
                    name: p.0.clone(),
                    login_status: p.1.login_status.clone(),
                })
                .collect(),
            default_profile: client_hub.default_profile.clone(),
        }
    }

    fn to(self) -> Result<ClientHub> {
        let mut hub = ClientHub::load_without_profiles(self.client_id)?;
        for profile in self.profiles {
            hub.add_profile(profile.name, profile.login_status)?;
        }
        hub.default_profile = self.default_profile;
        Ok(hub)
    }
}

pub fn client_hub_from_env() -> Result<ClientHub> {
    let host = std::env::var("NC_HOST")?;
    let username = std::env::var("NC_USERNAME")?;
    let password = std::env::var("NC_PASSWORD")?;
    let hub = ClientHub::new_with_auth(username, password, host)?;
    Ok(hub)
}

pub fn client_hub_from_toml(file_path: impl AsRef<Path>) -> Result<ClientHub> {
    let file_path = file_path.as_ref();
    let toml_str = fs::read_to_string(file_path);
    let mut hub = match toml_str {
        Ok(s) => {
            let raw: ClientHubRaw = toml::from_str(&s)?;
            raw.to()?
        }
        Err(e) => {
            log::info!("{}: {:?}", file_path.display(), e);
            ClientHub::new()?
        }
    };

    if_chain! {
        if let Some(ref default_profile) = hub.default_profile;
        if hub.get_profile(default_profile).is_none();
        then {
            hub.default_profile = None;
        }
    }

    Ok(hub)
}

pub fn save_client_hub_to_toml(hub: &ClientHub, file_path: impl AsRef<Path>) -> Result<()> {
    let file_path = file_path.as_ref();
    let hub = ClientHubRaw::from(hub);
    let toml_str = toml::to_string(&hub)?;
    fs::write(file_path, toml_str)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ExcludeListRaw {
    blackpaths: Vec<PathBuf>,
    whitepaths: Vec<PathBuf>,
    blackregexes: Vec<String>,
    whiteregexes: Vec<String>,
}

impl ExcludeListRaw {
    pub fn to(self) -> ExcludeList {
        ExcludeList::new(
            self.blackpaths,
            self.whitepaths,
            self.blackregexes,
            self.whiteregexes,
        )
    }

    pub fn from(exclude_list: &ExcludeList) -> Self {
        ExcludeListRaw {
            blackpaths: exclude_list.paths.original_blacks.clone(),
            whitepaths: exclude_list.paths.original_whites.clone(),
            blackregexes: exclude_list.regexes.original_blacks.clone(),
            whiteregexes: exclude_list.regexes.original_whites.clone(),
        }
    }
}

pub fn exc_list_from_toml(file_path: impl AsRef<Path>) -> Result<ExcludeList> {
    let file_path = file_path.as_ref();
    let toml_str = fs::read_to_string(file_path)?;
    let exc_list: ExcludeListRaw = toml::from_str(&toml_str)?;
    let exc_list = exc_list.to();
    Ok(exc_list)
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct LocalInfoRaw {
    excludes: ExcludeListRaw,
}

impl LocalInfoRaw {
    fn to(self) -> LocalInfo {
        LocalInfo::new(self.excludes.to())
    }

    fn from(local_info: &LocalInfo) -> Self {
        Self {
            excludes: ExcludeListRaw::from(&local_info.excludes),
        }
    }
}

// reqwest clientについては仮置き
pub fn localinfo_from_toml(file_path: impl AsRef<Path>) -> Result<LocalInfo> {
    let file_path = file_path.as_ref();
    let toml_str = fs::read_to_string(file_path);
    let local_info: LocalInfoRaw = match toml_str {
        Ok(s) => toml::from_str(&s)?,
        Err(e) => {
            log::info!("{}: {:?}", file_path.display(), e);
            LocalInfoRaw::default()
        }
    };
    let local_info = local_info.to();
    Ok(local_info)
}

pub fn save_localinfo_to_toml(local_info: &LocalInfo, file_path: impl AsRef<Path>) -> Result<()> {
    let file_path = file_path.as_ref();
    let local_info = LocalInfoRaw::from(local_info);
    let toml_str = toml::to_string(&local_info)?;
    fs::write(file_path, toml_str)?;
    Ok(())
}

pub fn setting_from_toml<P, Q>(ncinfo_fp: P, localinfo_fp: Q) -> Result<(ClientHub, LocalInfo)>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let hub = client_hub_from_toml(ncinfo_fp)?;
    let local_info = localinfo_from_toml(localinfo_fp)?;
    Ok((hub, local_info))
}

use crate::setting::CurDirSetting;

#[derive(Debug, Serialize, Deserialize)]
pub struct CurDirSettingRaw {
    pub current_dir: PathBuf,
}

impl CurDirSettingRaw {
    pub fn to(self) -> CurDirSetting {
        CurDirSetting::new(self.current_dir)
    }

    pub fn from(curdir_setting: &CurDirSetting) -> Self {
        Self {
            current_dir: curdir_setting.current_dir.clone(),
        }
    }
}

pub fn curdir_setting_from_toml(file_path: impl AsRef<Path>) -> Result<CurDirSetting> {
    let file_path = file_path.as_ref();
    let toml_str = fs::read_to_string(file_path)?;
    let curdir_setting: CurDirSettingRaw = toml::from_str(&toml_str)?;
    let curdir_setting = curdir_setting.to();
    Ok(curdir_setting)
}

pub fn save_curdir_setting_to_toml(
    curdir_setting: &CurDirSetting,
    file_path: impl AsRef<Path>,
) -> Result<()> {
    let file_path = file_path.as_ref();
    let curdir_setting = CurDirSettingRaw::from(curdir_setting);
    let toml_str = toml::to_string(&curdir_setting)?;
    fs::write(file_path, toml_str)?;
    Ok(())
}
