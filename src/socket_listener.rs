use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io;

async fn process_socket(mut socket: tokio::net::TcpStream) {
    let mut buf = [0; 1024];

    // check if it's a http request
    match socket.read(&mut buf).await {
        Ok(0) => return,
        Ok(n) => { //connection ended
            let response = get_response(buf);
            if let Err(e) = socket.write_all(response.as_bytes()).await {
                eprintln!("Failed to write to socket: {}", e);
            }
        }
        _ => {
            panic!("Match problem.")
        }
    }
}

fn get_response(mut buf: [u8; 1024]) -> String {
    // check if it's a http request
    if buf.starts_with(b"GET ") {
        // Check directories for http file
    } else {
        // send bad request response
        return "HTTP/1.1 400 Bad Request\r\nContent-Length: 11\r\n\r\nBad Request".to_string();
    }
}

pub(crate) async fn start_server() -> io::Result<()> {
    // temp local host bind
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            process_socket(socket).await;
        });
    }
}