use tokio::fs;

// returns http contents and if the contents are a 404 page or not
pub(crate) async fn retrieve_from_path(path: &str) -> (String, bool) {
    // check if file exists

}