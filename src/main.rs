// Standard library imports
use http_body_util::Full;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
// External crate imports
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

// Internal modules
mod cache;
mod html_getters;
mod method_handlers;

// Internal crates
use crate::cache::cache_impl::Cache;
use crate::method_handlers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // def address/port and bind them
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    // define cache to store http contents without file accesses
    let cache = Cache::new();

    // connection accepting loop
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let cache_clone = Arc::clone(&cache);

        // spawns tokio task for concurrent handling
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|req| handle_conn(req, Arc::clone(&cache_clone))),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }

    async fn handle_conn(
        req: Request<hyper::body::Incoming>,
        cache_ref: Arc<Cache>,
    ) -> Result<Response<Full<Bytes>>, Infallible> {
        // check request type
        return match req.method() {
            &hyper::Method::OPTIONS => options_handler::handle_option(req).await,
            &hyper::Method::GET => get_handler::handle_get(req, Arc::clone(&cache_ref)).await,
            &hyper::Method::HEAD => head_handler::handle_head(req).await,
            &hyper::Method::POST => post_handler::handle_post(req).await,
            &hyper::Method::PUT => put_handler::handle_put(req).await,
            &hyper::Method::DELETE => delete_handler::handle_delete(req).await,
            &hyper::Method::TRACE => trace_handler::handle_trace(req).await,
            &hyper::Method::CONNECT => connect_handler::handle_connect(req).await,
            _ => handler_utils::packet_templates::send_not_implemented_packet(),
        };
    }
}
