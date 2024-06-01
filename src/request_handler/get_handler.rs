// Standard library imports
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use http_body_util::Full;
// External crate imports
use hyper::{Request, Response, StatusCode, Uri};
use hyper::body::Bytes;
use tokio::sync::RwLock;

// Internal modules
use crate::html_getters::{cache_accessor, dir_accessor};

// Handles get requests, returning either a get response packet or server error packet
pub(crate) async fn handle_get(req: Request<hyper::body::Incoming>, cache: Arc<RwLock<HashMap<Uri, String>>>) -> Result<Response<Full<Bytes>>, Infallible> {
    // clone arc instance
    let cache_clone = Arc::clone(&cache);
    
    // check cache for the page
    let http_content = cache_accessor::read_cache(cache, req.uri()).await;
    if http_content != "Null" {
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Full::new(Bytes::from(http_content)))
            .unwrap();
        return Ok(response)
    }

    // if not in cache, check if file exists
    let http_content_result = dir_accessor::retrieve_from_path(req.uri()).await;

    match http_content_result {
        // No error, page found
        Ok((http_content, false)) => {
            // cache content then send response
            cache_accessor::write_to_cache(cache_clone, req.uri(), &http_content).await;

            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(Full::new(Bytes::from(http_content)))
                .unwrap();
            Ok(response)
        }
        // No error, page not found
        Ok((http_content, true)) => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/html")
                .body(Full::new(Bytes::from(http_content)))
                .unwrap();
            Ok(response)
        }
        // Error.
        Err(_error) => {
            // Send back server error packet
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::new()))
                .unwrap();
            Ok(response)
        }
    }
}