use anyhow::Result;
use ncsync_lib::cli::{login_request, poll};
use ncsync_lib::communicate::ls;
use ncsync_lib::setting::readwrite::setting_from_toml;
use ncsync_lib::setting::{LocalInfo, LoginStatus::*, NCInfo};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let target = env::args().nth(1).unwrap_or("/".to_string());

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
            loggedin(&nc_info, &local_info, &target).await?;
        }
        LoggedIn { .. } => loggedin(&nc_info, &local_info, &target).await?,
    }

    Ok(())
}

async fn loggedin(nc_info: &NCInfo, local_info: &LocalInfo, target: &str) -> Result<()> {
    let entry = ls(nc_info, target.as_ref()).await?;
    println!("{}", entry.get_tree(&local_info.get_exclude_list(), true));
    // println!("{:?}", entry);

    Ok(())
}
