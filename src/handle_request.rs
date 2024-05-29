use tokio::net::TcpListener;

use std::io;

async fn process_socket<T>(socket: T) {
    //TODO
}

pub(crate) async fn start_server() -> io::Result<()> {
    // temp local host bind
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        process_socket(socket).await;
    }
}