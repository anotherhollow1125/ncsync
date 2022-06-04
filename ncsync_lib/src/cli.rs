use crate::login;
use crate::setting::readwrite::save_ncinfo_to_toml;
use crate::setting::{LoginStatus, NCInfo};
use anyhow::Result;
use std::io::stdin;
use std::io::{stdout, Write};

pub async fn login_request(nc_info: &mut NCInfo, ncinfo_file_path: &str) -> Result<()> {
    let mut host = String::new();
    print!("host url? : ");
    stdout().flush().unwrap();
    stdin().read_line(&mut host).unwrap();
    let host = host.trim();

    let res = login::login_request(nc_info, host).await?;

    println!("\nPlease log in your Next Cloud from:\n\n\t{}\n", res.login);

    nc_info.login_status = LoginStatus::Polling {
        token: res.poll.token,
        end_point: res.poll.endpoint,
    };
    save_ncinfo_to_toml(nc_info, ncinfo_file_path)?;

    Ok(())
}

pub async fn poll(nc_info: &mut NCInfo, ncinfo_file_path: &str) -> Result<()> {
    let (token, end_point) = match &nc_info.login_status {
        LoginStatus::Polling { token, end_point } => (token.clone(), end_point.clone()),
        _ => return Err(anyhow::anyhow!("not polling")),
    };

    let res = login::polling(nc_info, &token, &end_point).await?;

    let res = match res {
        Some(v) => v,
        _ => return Err(anyhow::anyhow!("You are not logged in yet.")),
    };

    nc_info.login_status = LoginStatus::LoggedIn {
        host: res.server,
        username: res.login_name,
        password: res.app_password,
    };
    save_ncinfo_to_toml(nc_info, ncinfo_file_path)?;

    println!("You are logged in NextCloud.");

    Ok(())
}
