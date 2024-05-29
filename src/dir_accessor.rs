use tokio::fs;

pub(crate) async fn page_exists(path: &str) -> bool {
    // check if file exists
    match fs::metadata(path).await {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    }
}