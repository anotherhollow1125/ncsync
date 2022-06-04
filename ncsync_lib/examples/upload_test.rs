use anyhow::{Context, Result};
use ncsync_lib::cli::{login_request, poll};
use ncsync_lib::communicate::upload::upload;
use ncsync_lib::setting::readwrite::setting_from_toml;
use ncsync_lib::setting::{LocalInfo, LoginStatus::*, NCInfo};
use std::env;
use std::io::{stdin, stdout, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let resource = env::args().nth(1).context("Please pass resource path.")?;
    let target = env::args().nth(2).context("Please pass target path.")?;

    dotenv::dotenv().ok();
    env_logger::init();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let ncinfo_file_path = format!("{}/ncinfo.toml", manifest_dir);
    let local_info_file_path = format!("{}/localinfo.toml", manifest_dir);
    let (mut nc_info, local_info) = setting_from_toml(&ncinfo_file_path, &local_info_file_path)?;

    println!("nc_info: {:?}", nc_info);
    println!("local_info: {:?}", local_info);

    match nc_info.login_status.clone() {
        NotYet => login_request(&mut nc_info, &ncinfo_file_path).await?,
        Polling { .. } => {
            poll(&mut nc_info, &ncinfo_file_path).await?;
            loggedin(&nc_info, &local_info, &resource, &target).await?;
        }
        LoggedIn { .. } => loggedin(&nc_info, &local_info, &resource, &target).await?,
    }

    Ok(())
}

use std::fs;
use std::path::Path;

async fn loggedin<P, Q>(
    nc_info: &NCInfo,
    local_info: &LocalInfo,
    resource: P,
    target: Q,
) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let target = target.as_ref();

    if !local_info.get_exclude_list().judge(target) {
        let mut input = String::new();
        print!("This path is excluded. Are you sure upload? (y/N): ");
        stdout().flush()?;
        stdin().read_line(&mut input)?;
        if input.trim() != "y" {
            return Ok(());
        }
    }

    let bytes = fs::read(resource.as_ref())?;
    upload(nc_info, target, bytes).await?;

    println!("Send: {:?}", target);

    Ok(())
}
