// Standard library imports
use hyper::body::Bytes;
use std::env;
use std::path::PathBuf;
use std::time::SystemTime;
// External crate imports
use hyper::Uri;
use tokio::fs;
use tokio::io;

// returns http contents, and last modified (if 404 not found, no date is returned)
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
            let http_content = fs::read_to_string(path.clone()).await?;
            let last_modified = fs::metadata(path.clone()).await?.modified()?;
            let return_data = (Bytes::from(http_content), Some(last_modified));
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
