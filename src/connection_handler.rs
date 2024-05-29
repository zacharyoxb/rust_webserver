use std::convert::Infallible;
use http_body_util::Full;
use hyper::{Request, Response};
use hyper::body::Bytes;
use crate::Cache;

pub(crate) async fn handle_conn(req: Request<hyper::body::Incoming>, cache: &Cache) -> Result<Response<Full<Bytes>>, Infallible> {
    // check if is valid http request
    if req.method() == hyper::Method::GET {

    }
}
