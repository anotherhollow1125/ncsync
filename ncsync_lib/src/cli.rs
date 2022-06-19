use crate::login;
use crate::setting::readwrite::save_client_hub_to_toml;
use crate::setting::{ClientHub, LoginStatus};
use anyhow::Result;
use std::io::stdin;
use std::io::{stdout, Write};

pub async fn login_request(
    target_name: &str,
    client_hub: &mut ClientHub,
    client_hub_file_path: &str,
) -> Result<()> {
    let mut client = client_hub.get_mut_client(target_name)?;

    let mut host = String::new();
    print!("host url? : ");
    stdout().flush().unwrap();
    stdin().read_line(&mut host).unwrap();
    let host = host.trim();

    let res = login::login_request(&client.client_hub, host).await?;

    println!("\nPlease log in your Next Cloud from:\n\n\t{}\n", res.login);

    client.get_mut_profile().login_status = LoginStatus::Polling {
        token: res.poll.token,
        end_point: res.poll.endpoint,
    };
    save_client_hub_to_toml(&client.client_hub, client_hub_file_path)?;

    Ok(())
}

pub async fn poll(
    target_name: &str,
    client_hub: &mut ClientHub,
    client_hub_file_path: &str,
) -> Result<()> {
    let mut client = client_hub.get_mut_client(target_name)?;

    let (token, end_point) = match &client.get_profile().login_status {
        LoginStatus::Polling { token, end_point } => (token.clone(), end_point.clone()),
        _ => return Err(anyhow::anyhow!("not polling")),
    };

    let res = login::polling(client.client_hub, &token, &end_point).await?;

    let res = match res {
        Some(v) => v,
        _ => return Err(anyhow::anyhow!("You are not logged in yet.")),
    };

    client.get_mut_profile().login_status = LoginStatus::LoggedIn {
        host: res.server,
        username: res.login_name,
        password: res.app_password,
    };
    save_client_hub_to_toml(&client.client_hub, client_hub_file_path)?;

    println!("You are logged in NextCloud.");

    Ok(())
}
