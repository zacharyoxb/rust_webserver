// Standard library imports
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

// External crate imports
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use hyper::{Uri};

// Internal modules
mod html_getters;
mod request_handler;


// type alias cos I'm not writing that crap again
type Cache = Arc<RwLock<HashMap<Uri, String>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // def address/port and bind them
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    // define cache to store http contents without file accesses
    let hashmap: HashMap<Uri, String> = HashMap::new();
    let cache: Cache = Arc::new(RwLock::new(hashmap));

    // connection accepting loop
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let cache_clone = Arc::clone(&cache);

        // spawns tokio task for concurrent handling
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|req|request_handler::connection_handler::handle_conn(req, Arc::clone(&cache_clone))))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

