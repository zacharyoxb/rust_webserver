use std::env;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Timelike, Utc};
use hyper::body::Bytes;
use hyper::Uri;
use tokio::fs;
use tokio::io;

// returns the resource, or an error
// TODO: Make this work for many resources, not just text
pub(crate) async fn retrieve_resource(uri: &Uri) -> Result<(Bytes, Option<SystemTime>), io::Error> {
    // check if file exists
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("html");

    if uri == "/" {
        path.push("index.html");
    } else {
        path.push(uri.to_string());
    }

    let path_exists = path.try_exists()?;
    match path_exists {
        true => {
            // read the content and get the last modified in SystemTime
            let http_content = fs::read_to_string(path.clone()).await?;
            let last_modified = fs::metadata(path.clone()).await?.modified()?;

            // convert SystemTime to DateTime then round/convert back
            let datetime_last_mod: DateTime<Utc> = DateTime::from(last_modified);
            let datetime_trunc = datetime_last_mod.with_nanosecond(0).unwrap();
            let last_modified_rounded =
                SystemTime::UNIX_EPOCH + Duration::new(datetime_trunc.timestamp() as u64, 0);

            // form tuple and send off
            let return_data = (Bytes::from(http_content), Some(last_modified_rounded));
            Ok(return_data)
        }
        false => {
            let mut not_found_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            not_found_path.push("html");
            not_found_path.push("404.html");
            let http_content = fs::read_to_string(not_found_path).await?;
            let return_data = (Bytes::from(http_content), None);
            Ok(return_data)
        }
    }
}
