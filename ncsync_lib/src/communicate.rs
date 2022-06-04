use crate::entry::{Entry, EntryType, Etag};
use crate::errors::NcsError::*;
use crate::path::{check_absolute, url2path, AsNCUrl};
use crate::setting::NCInfo;
use anyhow::{Context, Result};
use async_recursion::async_recursion;
use chrono::DateTime;
use reqwest::Method;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use urlencoding::decode;

pub mod download;
pub mod upload;

pub const WEBDAV_BODY: &str = r#"<?xml version="1.0"?>
<d:propfind  xmlns:d="DAV:" xmlns:oc="http://owncloud.org/ns" xmlns:nc="http://nextcloud.org/ns">
  <d:prop>
        <d:getetag />
        <d:getcontenttype />
        <d:getlastmodified />
        <oc:size />
  </d:prop>
</d:propfind>
"#;

async fn reqest(nc_info: &NCInfo, target: &Path) -> Result<Vec<Entry>> {
    if !check_absolute(target) {
        return Err(BadPath.into());
    }

    let url = target.as_nc_url(nc_info)?;

    log::debug!("reqest: {}", url);

    let ref client = nc_info.get_client();

    let (username, password, root_prefix) = match nc_info.get_userinfo() {
        Some(v) => v,
        _ => return Err(NotLoggedIn.into()),
    };

    let mut counter = 0;
    let res = loop {
        let res = client
            .request(Method::from_bytes(b"PROPFIND").unwrap(), url.as_str())
            .basic_auth(&username, Some(&password))
            .header("Depth", "Infinity")
            .body(WEBDAV_BODY)
            .send()
            .await?;
        if res.status().is_success() {
            break res;
        }

        if res.status().as_u16() == 401 {
            return Err(NotAuthorized.into());
        }

        if counter >= 3 {
            return Err(BadStatusError(res.status().as_u16()).into());
        }

        counter += 1;
        sleep(Duration::from_millis(100)).await;
    };

    let text = res.text_with_charset("utf-8").await?;
    let document = roxmltree::Document::parse(&text)?;
    let response = webdav_xml2responses(&document, &root_prefix)?;

    Ok(response)
}

async fn get(nc_info: &NCInfo, target: &Path) -> Result<Entry> {
    log::debug!("target: {:?}", target);
    let response = reqest(nc_info, target).await?;
    let entry = response
        .into_iter()
        .map(|e| {
            // log::debug!("beep: {:?}", e.path);
            e
        })
        .filter(|e| e.path == target)
        .nth(0)
        .context("Entry Not Found")?;

    Ok(entry)
}

async fn get_children(nc_info: &NCInfo, target: &Path) -> Result<Vec<Entry>> {
    let response = reqest(nc_info, target).await?;
    let response = response.into_iter().filter(|e| e.path != target).collect();

    Ok(response)
}

#[async_recursion]
async fn ls_rec(nc_info: &NCInfo, entry: &mut Entry) -> Result<()> {
    let (path, children) = match entry {
        Entry {
            path,
            entry_type: EntryType::Dir { children },
            ..
        } => (path, children),
        _ => return Ok(()),
    };

    let entries = get_children(nc_info, path).await?;

    for mut entry in entries.into_iter() {
        ls_rec(nc_info, &mut entry).await?;
        let p = entry.path.clone();
        children.insert(p, entry);
    }

    Ok(())
}

/*
ls, pull, push系に渡すパスはすべて絶対パスで解決済みとしたい。
*/

pub async fn ls(nc_info: &NCInfo, target: &str) -> Result<Entry> {
    // check target is absolute path
    if !check_absolute(target) {
        return Err(BadPath.into());
    }

    let path = Path::new(&target);

    let mut entry = get(nc_info, path).await?;
    ls_rec(nc_info, &mut entry).await?;

    Ok(entry)
}

fn webdav_xml2responses(document: &roxmltree::Document, root_prefix: &str) -> Result<Vec<Entry>> {
    let res = document
        .root_element()
        .children()
        .map(|n| {
            if n.tag_name().name() != "response" {
                return Err(anyhow!("Invalid document"));
            }

            let mut path_w = None;
            let mut etag_w = None;
            let mut type_w = None;
            let mut last_modified_w = None;
            let mut size_w = None;

            for m in n.children() {
                match m.tag_name().name() {
                    "href" => {
                        if let Some(href) = m.text() {
                            let path = decode(&href)?;
                            let path = url2path(&path, root_prefix)?;
                            path_w = Some(path);
                        }
                    }
                    "propstat" => {
                        for d in m.descendants() {
                            match d.tag_name().name() {
                                "getetag" => {
                                    etag_w = d.text().and_then(|s| Some(Etag::new(&s)));
                                }
                                "getcontenttype" => {
                                    type_w = match d.text() {
                                        Some(ref s) if s != &"" => Some(EntryType::new_file(None)),
                                        _ => Some(EntryType::new_dir()),
                                    };
                                }
                                "getlastmodified" => {
                                    last_modified_w = d.text().and_then(|s| {
                                        let s = s.trim();
                                        let dt = match DateTime::parse_from_rfc2822(s) {
                                            Ok(dt) => dt,
                                            Err(_) => return None,
                                        };
                                        Some(dt.into())
                                    });
                                }
                                "size" => {
                                    if let Some(s) = d.text() {
                                        if let Ok(s) = s.parse::<usize>() {
                                            size_w = Some(s);
                                        }
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }
            if_chain! {
                if let Some(path) = path_w;
                if let Some(etag) = etag_w;
                if let Some(type_) = type_w;
                if let Some(last_modified) = last_modified_w;
                if let Some(size) = size_w;
                then {
                    let type_ = if let EntryType::File {..} = type_ {
                        EntryType::new_file(Some(etag))
                    } else {
                        type_
                    };

                    Ok(Some(Entry::new(path, type_, last_modified, size)))
                } else {
                    Ok(None)
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter_map(|v| v)
        .collect();

    Ok(res)
}
