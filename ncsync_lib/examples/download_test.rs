use anyhow::{Context, Result};
use ncsync_lib::cli::{login_request, poll};
use ncsync_lib::communicate::download::download;
use ncsync_lib::setting::readwrite::setting_from_toml;
use ncsync_lib::setting::{ClientHub, LocalInfo, LoginStatus::*};
use std::env;
use std::io::{stdin, stdout, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let profile_name = env::args().nth(1).context("Please pass profile name.")?;
    let target = env::args().nth(2).context("Please pass target path.")?;

    dotenv::dotenv().ok();
    env_logger::init();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let client_hub_file_path = format!("{}/profiles.toml", manifest_dir);
    let local_info_file_path = format!("{}/localinfo.toml", manifest_dir);
    let (mut hub, local_info) = setting_from_toml(&client_hub_file_path, &local_info_file_path)?;

    println!("client_hub: {:?}", hub);
    println!("local_info: {:?}", local_info);

    let (profile_name, login_status) = {
        let profile_exist = hub.get_profile(&profile_name).is_some();

        if !profile_exist {
            hub.add_profile(profile_name.clone(), NotYet)?;
        }

        let profile = hub.get_profile(&profile_name).unwrap();
        (profile.name.to_string(), profile.login_status.clone())
    };

    match login_status {
        NotYet => login_request(&profile_name, &mut hub, &client_hub_file_path).await?,
        Polling { .. } => {
            poll(&profile_name, &mut hub, &client_hub_file_path).await?;
            loggedin(&profile_name, &hub, &local_info, &target).await?;
        }
        LoggedIn { .. } => loggedin(&profile_name, &hub, &local_info, &target).await?,
    }

    Ok(())
}

use std::fs;
use std::path::Path;

async fn loggedin(
    profile_name: &str,
    client_hub: &ClientHub,
    local_info: &LocalInfo,
    target: impl AsRef<Path>,
) -> Result<()> {
    let target = target.as_ref();

    if !local_info.get_exclude_list().judge(target) {
        let mut input = String::new();
        print!("This path is excluded. Are you sure download? (y/N): ");
        stdout().flush()?;
        stdin().read_line(&mut input)?;
        if input.trim() != "y" {
            return Ok(());
        }
    }

    let bytes = download(profile_name, client_hub, target).await?;
    let path = target
        .file_name()
        .context("Invalid File Name! Save failed.")?;
    fs::write(path, bytes)?;

    println!("Saved: {:?}", target);

    Ok(())
}
