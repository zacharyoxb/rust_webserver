use std::path::PathBuf;
use hyper::Uri;
use tokio::fs;

// returns http contents and if the contents are a 404 page or not
pub(crate) async fn retrieve_from_path(uri: &Uri) -> (String, bool) {
    // check if file exists
    let path;
    if uri.to_string() == "/" {
        path = "../html/index.html".to_string()
    } else {
        path = format!("../html{}", uri.to_string());
    }

    let file_path = PathBuf::from(path);
    if file_path.exists() {
        return (fs::read_to_string(file_path).await?, false);
    } else {
        let not_found_path = PathBuf::from("../html/404.html");
        return(fs::read_to_string(not_found_path).await?, true);
    }
}