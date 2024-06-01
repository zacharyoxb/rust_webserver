// Standard library imports
use std::env;
use std::path::PathBuf;
// External crate imports
use hyper::Uri;
use tokio::fs;
use tokio::io;

// returns http contents and if the contents are a 404 page or not
pub(crate) async fn retrieve_from_path(uri: &Uri) -> Result<(String, bool), io::Error> {
    // check if file exists
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("html");
    
    if uri.to_string() == "/" {
        path.push("index.html");
    } else {
        path.push(uri.to_string());
    }
    
    let path_exists = path.try_exists()?;
    return match path_exists {
        true => {
            let http_content = (fs::read_to_string(path).await?, false);
            Ok(http_content)
        }
        false => {
            let mut not_found_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            not_found_path.push("html");
            not_found_path.push("404.html");
            let http_content = (fs::read_to_string(not_found_path).await?, true);
            Ok(http_content)
        }
    }
}