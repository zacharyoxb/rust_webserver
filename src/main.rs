use std::collections::HashMap;
use std::hash::Hash;
use std::net::SocketAddr;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use hyper::{Uri};

// type alias cos I'm not writing that crap again
type Cache = RwLock<HashMap<Uri, String>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // def address/port and bind them
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    // define cache to store http contents without file accesses
    let mut hashmap: HashMap<Uri, String> = HashMap::new();
    let cache: Cache = RwLock::new(hashmap);
    // create reference to avoid ownership problems
    let cache_ref = &cache;

    // connection accepting loop
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        // spawns tokio task for concurrent handling
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|req|connection_handler::handle_conn(req, cache_ref)))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        })
    }
}

mod connection_handler;
mod dir_accessor;