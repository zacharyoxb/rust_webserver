use std::path::PathBuf;
use hyper::Uri;
use tokio::fs;
use tokio::io;

// returns http contents and if the contents are a 404 page or not
pub(crate) async fn retrieve_from_path(uri: &Uri) -> Result<(String, bool), io::Error> {
    // check if file exists
    let path;
    if uri.to_string() == "/" {
        path = "../html/index.html".to_string()
    } else {
        path = format!("../html{}", uri.to_string());
    }

    let file_path = PathBuf::from(path);
    return if file_path.exists() {
        let http_content = (fs::read_to_string(file_path).await?, false);
        Ok(http_content)
    } else {
        let not_found_path = PathBuf::from("../html/404.html");
        let http_content = (fs::read_to_string(not_found_path).await?, true);
        Ok(http_content)
    }
}