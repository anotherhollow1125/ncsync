#[macro_use]
extern crate if_chain;
#[macro_use]
extern crate anyhow;

pub mod cli;
pub mod communicate;
mod entry;
mod errors;
pub mod login;
mod path;
pub mod setting;

#[cfg(test)]
mod tests {
    use crate::communicate::ls;
    use crate::setting::readwrite;

    #[tokio::test]
    async fn rls_test() {
        dotenv::dotenv().ok();
        env_logger::init();
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let clienthub_fp = format!("{}/profiles.toml", manifest_dir);
        let localinfo_fp = format!("{}/localinfo.toml", manifest_dir);
        let (client_hub, local_info) =
            readwrite::setting_from_toml(clienthub_fp.as_str(), localinfo_fp.as_str()).unwrap();

        println!("client_hub: {:?}", client_hub);
        println!("local_info: {:?}", local_info);

        let entry = ls("for_test", &client_hub, "/".as_ref()).await.unwrap();

        println!("{}", entry.get_tree(&local_info.get_exclude_list(), false));
    }
}
