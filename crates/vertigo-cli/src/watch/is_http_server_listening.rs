pub async fn is_http_server_listening(port: u16) -> bool {
    use tokio::net::TcpStream;
    use tokio::io::AsyncWriteExt;
    use tokio::io::AsyncReadExt;

    match TcpStream::connect(("127.0.0.1", port)).await {
        Ok(mut stream) => {
            // Send a HEAD request to the HTTP server
            if let Err(_) = stream.write_all(b"HEAD / HTTP/1.0\r\n\r\n").await {
                return false;
            }
            // Wait for a response
            let mut buf = [0; 1024];
            match stream.read(&mut buf).await {
                Ok(n) if n > 0 => true, // If we received a response, the HTTP server is running
                _ => false, // Otherwise, the HTTP server is not running
            }
        },
        Err(_) => false, // Connection error means that the HTTP server is not running
    }
}
