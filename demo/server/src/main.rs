#![deny(rust_2018_idioms)]

use vertigo_demo_server::build_server;

const HOST: &str = "127.0.0.1";
const PORT: u16 = 3333;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server start on {HOST}:{PORT} ...");

    build_server(HOST, PORT)?.await
}
