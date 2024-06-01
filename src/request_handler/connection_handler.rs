// External imports
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use http_body_util::Full;
use hyper::{Request, Response, StatusCode, Uri};
use hyper::body::Bytes;
use tokio::sync::RwLock;
use crate::html_getters;

// Internal imports
use crate::html_getters::*;
use crate::request_handler::*;

pub(crate) async fn handle_conn(req: Request<hyper::body::Incoming>, cache: Arc<RwLock<HashMap<Uri, String>>>) -> Result<Response<Full<Bytes>>, Infallible> {
    let cache_clone = Arc::clone(&cache);
    // define response to send to client
    // check request type
    if req.method() == hyper::Method::GET {
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
            Ok((http_content, is_404)) => {
                // if it's a 404 error, return that
                return if is_404 {
                    let response = Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header("Content-Type", "text/html")
                        .body(Full::new(Bytes::from(http_content)))
                        .unwrap();
                    Ok(response)
                } else {
                    // cache content then send response
                    cache_accessor::write_to_cache(cache_clone, req.uri(), &http_content).await;
                    
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/html")
                        .body(Full::new(Bytes::from(http_content)))
                        .unwrap();
                    Ok(response)
                }
            }
            Err(Error) => {
                // TODO: Send back server error packet
                eprintln!("{}", Error)
            }
        }
    }
    
    // Otherwise send bad request
    let response = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Full::new(Bytes::new()))
        .unwrap();
    return Ok(response)
}


