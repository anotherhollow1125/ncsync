use crate::setting::{ExcludeList, LocalInfo, LoginStatus, NCInfo};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs;
use std::path::{Path, PathBuf};

// NCInfoはclient_idを持たなければならないのでLocalInfoRawにDefaultを持たせてはいけない
#[derive(Debug, Serialize, Deserialize)]
struct NCInfoRaw {
    client_id: String,
    login_status: LoginStatus,
}

impl NCInfoRaw {
    fn from(nc_info: &NCInfo) -> NCInfoRaw {
        NCInfoRaw {
            client_id: nc_info.client_id.clone(),
            login_status: nc_info.login_status.clone(),
        }
    }

    fn to(self) -> Result<NCInfo> {
        NCInfo::load(self.login_status, &self.client_id)
    }
}

pub fn ncinfo_from_json(file_path: impl AsRef<Path>) -> Result<NCInfo> {
    let file_path = file_path.as_ref();
    let json_str = fs::read_to_string(file_path)?;
    let nc_info: NCInfoRaw = serde_json::from_str(&json_str)?;
    let nc_info = nc_info.to()?;
    Ok(nc_info)
}

pub fn ncinfo_from_env() -> Result<NCInfo> {
    let host = std::env::var("NC_HOST")?;
    let username = std::env::var("NC_USERNAME")?;
    let password = std::env::var("NC_PASSWORD")?;
    let nc_info = NCInfo::new_with_auth(username, password, host)?;
    Ok(nc_info)
}

pub fn ncinfo_from_toml(file_path: impl AsRef<Path>) -> Result<NCInfo> {
    let file_path = file_path.as_ref();
    let toml_str = fs::read_to_string(file_path);
    let nc_info = match toml_str {
        Ok(s) => {
            let raw: NCInfoRaw = toml::from_str(&s)?;
            raw.to()?
        }
        Err(e) => {
            log::info!("{}: {:?}", file_path.display(), e);
            NCInfo::new()?
        }
    };
    Ok(nc_info)
}

pub fn save_ncinfo_to_toml(nc_info: &NCInfo, file_path: impl AsRef<Path>) -> Result<()> {
    let file_path = file_path.as_ref();
    let nc_info = NCInfoRaw::from(nc_info);
    let toml_str = toml::to_string(&nc_info)?;
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

/* 作った後に気づいたけどexcludesとパスワードが一緒に見えてしまうのは問題
#[derive(Debug, Serialize, Deserialize)]
pub struct Setting {
    nextcloud: Option<NCInfoRaw>,
    local: Option<LocalInfoRaw>,
}

pub fn setting_from_toml(file_path: impl AsRef<Path>) -> Result<(NCInfo, LocalInfo)> {
    let file_path = file_path.as_ref();
    let toml_str = fs::read_to_string(file_path).unwrap_or("".to_string());
    let setting: Setting = toml::from_str(&toml_str)?;
    let nextcloud = match setting.nextcloud {
        Some(nc_info) => nc_info.to()?,
        None => NCInfoRaw::default().to()?,
    };

    let local = match setting.local {
        Some(local_info) => local_info.to()?,
        None => LocalInfo::new(ExcludeList::default())?,
    };

    Ok((nextcloud, local))
}

pub fn save_setting_to_toml(
    nc_info: Option<&NCInfo>,
    local_info: Option<&LocalInfo>,
    file_path: impl AsRef<Path>,
) -> Result<()> {
    let file_path = file_path.as_ref();
    let setting = Setting {
        nextcloud: match nc_info {
            Some(nc_info) => Some(NCInfoRaw::from(nc_info)),
            None => None,
        },
        local: match local_info {
            Some(local_info) => Some(LocalInfoRaw::from(local_info)),
            None => None,
        },
    };
    let toml_str = toml::to_string(&setting)?;
    fs::write(file_path, toml_str)?;
    Ok(())
}
*/

pub fn setting_from_toml<P, Q>(ncinfo_fp: P, localinfo_fp: Q) -> Result<(NCInfo, LocalInfo)>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let nc_info = ncinfo_from_toml(ncinfo_fp)?;
    let local_info = localinfo_from_toml(localinfo_fp)?;
    Ok((nc_info, local_info))
}

pub fn save_setting_to_toml(
    nc_info: Option<&NCInfo>,
    local_info: Option<&LocalInfo>,
    file_path: impl AsRef<Path>,
) -> Result<()> {
    let file_path = file_path.as_ref();
    match nc_info {
        Some(nc_info) => save_ncinfo_to_toml(nc_info, file_path)?,
        None => (),
    }
    match local_info {
        Some(local_info) => save_localinfo_to_toml(local_info, file_path)?,
        None => (),
    }

    Ok(())
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
