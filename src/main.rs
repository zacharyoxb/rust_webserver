use tokio;

#[tokio::main]
async fn main() {
    if let Err(e) = socket_listener::start_server().await {
        eprintln!("Server error: {}", e);
    }
}

mod socket_listener;