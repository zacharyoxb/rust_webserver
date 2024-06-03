// Standard library imports
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use http_body_util::Full;
// External crate imports
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use hyper::{HeaderMap, Request, Response, StatusCode, Uri};
use hyper::body::Bytes;
use hyper::header::{HeaderName, RANGE};

// Internal modules
mod html_getters;
mod request_handler;

// Internal crates
use crate::request_handler::*;

// type alias
type Cache = Arc<RwLock<HashMap<Uri, (String, SystemTime)>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // def address/port and bind them
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    // define cache to store http contents without file accesses
    let hashmap: HashMap<Uri, (String, SystemTime)> = HashMap::new();
    let cache: Cache = Arc::new(RwLock::new(hashmap));

    // connection accepting loop
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let cache_clone = Arc::clone(&cache);

        // spawns tokio task for concurrent handling
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|req|handle_conn(req, Arc::clone(&cache_clone))))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }

    async fn handle_conn(req: Request<hyper::body::Incoming>, cache: Cache) -> Result<Response<Full<Bytes>>, Infallible> {
        // check request type
        return match req.method() {
            &hyper::Method::OPTIONS => options_handler::handle_option(req).await,
            &hyper::Method::GET => {
                return if last_modified_header(&req) {
                    get_handlers::last_modified_handler::handle_last_modified(req, cache).await
                } else if match_header(&req) {
                    get_handlers::match_handler::handle_match(req, cache).await
                } else {
                    get_handlers::get_handler::handle_get(req, cache).await
                }
            }
            &hyper::Method::HEAD => head_handler::handle_head(req).await,
            &hyper::Method::POST => post_handler::handle_post(req).await,
            &hyper::Method::PUT => put_handler::handle_put(req).await,
            &hyper::Method::DELETE => delete_handler::handle_delete(req).await,
            &hyper::Method::TRACE => trace_handler::handle_trace(req).await,
            &hyper::Method::CONNECT => connect_handler::handle_connect(req).await,
            _ => {
                server_error_handler::send_not_implemented_packet()
            }
        }
    }

    fn last_modified_header(req: &Request<hyper::body::Incoming>) -> bool {
        return if req.headers().get("If-Modified-Since").is_some() ||
            req.headers().get("If-Unmodified-Since").is_some() {
            true
        } else {
            false
        }
    }
    
    fn match_header(req: &Request<hyper::body::Incoming>) -> bool {
        return if req.headers().get("If-Match").is_some() ||
            req.headers().get("If-None-Match").is_some() {
            true
        } else {
            false
        }
    }
}

