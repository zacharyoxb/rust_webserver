use std::env;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Timelike, Utc};
use hyper::body::Bytes;
use hyper::Uri;
use tokio::fs;

// returns the resource, or an error
// TODO: Make this work for many resources, not just text
pub(crate) async fn retrieve_resource(uri: &Uri) -> Option<(Bytes, Option<SystemTime>)> {
    // check if file exists
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources");

    if uri == "/" {
        path.push("index.html");
    } else {
        let path_uri = match uri.to_string().strip_prefix('/') {
            Some(stripped_uri) => stripped_uri.to_string(),
            None => uri.to_string(),
        };

        path.push(path_uri);
    }

    let path_exists = match path.try_exists() {
        Ok(path_exists) => path_exists,
        Err(_) => return None,
    };

    match path_exists {
        true => {
            // read the content and get the last modified in SystemTime
            let resource_content = match fs::read(&path).await {
                Ok(resource_content) => resource_content,
                Err(_) => return None,
            };

            let last_modified = match fs::metadata(&path).await {
                Ok(metadata) => match metadata.modified() {
                    Ok(last_modified) => last_modified,
                    Err(_) => return None,
                },
                Err(_) => return None,
            };

            let resource_type = match &path.extension() {
                Some(extension) => match extension.to_string_lossy().to_lowercase().as_str() {
                    "html" => "text/html; charset=utf-8",
                    "ico" => "image/x-icon",
                    _ => return None,
                },
                None => return None,
            }
            .to_string();

            // convert SystemTime to DateTime then round/convert back
            let datetime_last_mod: DateTime<Utc> = DateTime::from(last_modified);
            let datetime_trunc = datetime_last_mod.with_nanosecond(0).unwrap();
            let last_modified_rounded =
                SystemTime::UNIX_EPOCH + Duration::new(datetime_trunc.timestamp() as u64, 0);

            // form tuple and send off
            let return_data = (Bytes::from(resource_content), Some(last_modified_rounded));
            Some(return_data)
        }
        false => {
            let mut not_found_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            not_found_path.push("resources");
            not_found_path.push("404.html");

            let resource_content = match fs::read(not_found_path).await {
                Ok(resource_content) => resource_content,
                Err(_) => return None,
            };

            let return_data = (Bytes::from(resource_content), None);
            Some(return_data)
        }
    }
}
