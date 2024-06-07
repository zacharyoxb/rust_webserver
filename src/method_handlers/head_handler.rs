// Standard library imports
use http_body_util::Full;
use std::convert::Infallible;
// External crate imports
use chrono::format::StrftimeItems;
use chrono::offset;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use sysinfo::System;

// Handles option requests, returning either a option response packet or server error packet
pub(crate) async fn handle_head(
    _req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    // TODO: When adding handlers, change Allow header
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Date", get_time())
        .header("Server", get_server_version())
        .header("Allow", "GET, OPTIONS, HEAD")
        .header("Content-Type", "text/html; charset=UTF-8")
        .body(Full::new(Bytes::new()))
        .unwrap();
    Ok(response)
}
// TODO: there are 2 functions like this?
fn get_time() -> String {
    // Standard HTTP format: Date: Sun, 02 Jun 2024 12:00:00 UTC
    let date_time = offset::Utc::now();
    let date_format = StrftimeItems::new("%a, %d %b %Y %H:%M:%S UTC");
    date_time.format_with_items(date_format.clone()).to_string()
}

fn get_server_version() -> String {
    System::os_version().unwrap_or_else(|| "Unknown version".to_string())
}
