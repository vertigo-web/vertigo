pub async fn is_http_server_listening(port: u16) -> Result<(), ()> {
    use tokio::net::TcpStream;
    use tokio::io::AsyncWriteExt;
    use tokio::io::AsyncReadExt;

    match TcpStream::connect(("127.0.0.1", port)).await {
        Ok(mut stream) => {
            // Send a HEAD request to the HTTP server
            match stream.write_all(b"HEAD / HTTP/1.0\r\n\r\n").await {
                Ok(_) => {},
                Err(err) => {
                    // Network error while sending request
                    println!("Error = {err}");
                    return Err(());
                }
            }

            // Wait for a response
            let mut buf = [0; 1024];
            match stream.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    // If we received a response, the HTTP server is running
                    Ok(())
                },
                Ok(_) => {
                    // HTTP server is running but probably failed to configure properly
                    Err(())
                },
                Err(err) => {
                    // Network error while reading response
                    println!("Error = {err}");
                    Err(())
                },
            }
        },
        Err(err) => {
            // Http server has not opened the socket yet
            println!("Error = {err}");
            Err(())
        },
    }
}
