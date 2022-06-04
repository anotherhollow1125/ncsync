use crate::setting::NCInfo;
use anyhow::Result;
use reqwest::Url;
use serde::{Deserialize, Serialize};

const LOGINREQUESTURL: &str = "/index.php/login/v2";
// const POLLINGURL: &str = "/login/v2/poll";

#[derive(Debug, Serialize, Deserialize)]
pub struct Poll {
    pub token: String,
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReqLoginResponseJson {
    pub poll: Poll,
    pub login: String,
}

pub async fn login_request(nc_info: &NCInfo, host: &str) -> Result<ReqLoginResponseJson> {
    let host = Url::parse(host)?;
    let client = nc_info.get_client();
    let url = host.join(LOGINREQUESTURL)?;
    let res = client.post(url).send().await?;
    let json: ReqLoginResponseJson = res.json().await?;

    Ok(json)
}

#[derive(Debug, Deserialize)]
pub struct PollResponseJson {
    pub server: String,
    #[serde(rename = "loginName")]
    pub login_name: String,
    #[serde(rename = "appPassword")]
    pub app_password: String,
}

pub async fn polling(
    nc_info: &NCInfo,
    token: &str,
    end_point: &str,
) -> Result<Option<PollResponseJson>> {
    let end_point = Url::parse(end_point)?;
    let client = nc_info.get_client();
    let res = client
        .post(end_point)
        .form(&[("token", token)])
        .send()
        .await?;

    if res.status() != 200 {
        return Ok(None);
    }

    let json: PollResponseJson = res.json().await?;

    Ok(Some(json))
}
