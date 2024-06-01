use std::convert::Infallible;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode, Uri};
use hyper::body::Bytes;
use crate::{Cache, dir_accessor};

pub(crate) async fn handle_conn(req: Request<hyper::body::Incoming>, cache: &Cache) -> Result<Response<Full<Bytes>>, Infallible> {
    // define response to send to client
    // check request type
    if req.method() == hyper::Method::GET {
        let http_content = read_cache(cache, req.uri()).await;
        if http_content != "Null" {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", req.headers().get("Content-Type").unwrap())
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
                    write_to_cache(cache, req.uri(), &http_content);
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", req.headers().get("Content-Type").unwrap())
                        .body(Full::new(Bytes::from(http_content)))
                        .unwrap();
                    Ok(response)
                }
            }
            Err(Error) => {
                // TODO: Send back server error packet
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

async fn read_cache(cache: &Cache, uri: &Uri) -> String {
    let guard = cache.read().await;
    return match guard.get(uri) {
        Some(http_content) => http_content.clone(),
        _ => "Null".to_string()
    }
}

async fn write_to_cache(cache: &Cache, uri: &Uri, http_content: &String) {
    let mut guard = cache.write().await;
    guard.insert(uri.clone(), http_content.clone());
}
