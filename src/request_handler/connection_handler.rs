// Standard library imports
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
// External crate imports
use http_body_util::Full;
use hyper::{Request, Response, StatusCode, Uri};
use hyper::body::Bytes;
use tokio::sync::RwLock;

// Internal modules
use crate::request_handler::*;

pub(crate) async fn handle_conn(req: Request<hyper::body::Incoming>, cache: Arc<RwLock<HashMap<Uri, String>>>) -> Result<Response<Full<Bytes>>, Infallible> {
    // define response to send to client
    // check request type
    if req.method() == hyper::Method::GET {
        return get_handler::handle_get(req, cache).await;
    }
    
    // Otherwise send bad request
    let response = Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Full::new(Bytes::new()))
        .unwrap();
    return Ok(response)
}


