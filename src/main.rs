use tokio;

#[tokio::main]
async fn main() {
    if let Err(e) = handle_request::start_server().await {
        eprintln!("Server error: {}", e);
    }
}

// Assuming your module is named `my_module`:
mod handle_request;