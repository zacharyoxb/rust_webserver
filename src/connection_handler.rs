use std::convert::Infallible;
use http_body_util::Full;
use hyper::{Request, Response, StatusCode, Uri};
use hyper::body::Body;
use hyper::body::Bytes;
use crate::{Cache, dir_accessor};

pub(crate) async fn handle_conn(req: Request<hyper::body::Incoming>, cache: &Cache) -> Result<Response<Full<Bytes>>, Infallible> {
    // define response to send to client
    // check if is valid http request (check cache first)
    if req.method() == hyper::Method::GET {
        let http_content = read_cache(cache, req.uri());
        if http_content != "Null" {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", req.headers().get("Content-Type"))
                .body(Body::from(http_content))
                .unwrap();
            return Ok(response)
        }

        // if not in cache, check if file exists
        let (http_content, is_404) = dir_accessor::retrieve_from_path(req.uri());
        // if it's a 404 error, return that
        if is_404 {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/html")
                .body(Body::from(http_content))
                .unwrap();
            return Ok(response)
        } else {
            // cache content then send response
            write_to_cache(cache, req.uri(), http_content);
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", req.headers().get("Content-Type"))
                .body(Body::from(http_content))
                .unwrap();
            return Ok(response)
        }
    }
    // Otherwise send bad request
    let response = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())
        .unwrap();
    return Ok(response)
}

fn read_cache(cache: &Cache, uri: &Uri) -> String {
    let guard = cache.read().unwrap();
    return match guard.get(uri) {
        Some(http_content) => http_content.clone(),
        _ => "Null".to_string()
    }
}

fn write_to_cache(cache: &Cache, uri: &Uri, http_content: String) {
    let guard = cache.write().unwrap();
    guard.insert(uri, http_content);
}
