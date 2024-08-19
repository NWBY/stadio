use hyper::{server::conn::http1, service::service_fn, Response};
use hyper_util::rt::TokioIo;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

mod config;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load("stadio.yaml")?;
    let backends = Arc::new(Mutex::new(VecDeque::from(config.backends)));

    let in_addr: SocketAddr = ([127, 0, 0, 1], 3001).into();

    let listener = TcpListener::bind(in_addr).await?;

    println!("Listening on http://{}", in_addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let backends = Arc::clone(&backends);

        let service = service_fn(move |mut req| {
            let backends = Arc::clone(&backends);

            async move {
                let mut backends = backends.lock().await;
                let backend = backends.pop_front().unwrap();
                backends.push_back(backend.clone());

                let uri_string = format!(
                    "{}{}",
                    backend,
                    req.uri()
                        .path_and_query()
                        .map(|x| x.as_str())
                        .unwrap_or("/")
                );
                let uri = uri_string.parse().unwrap();
                *req.uri_mut() = uri;

                // Add appropriate headers
                let headers = req.headers_mut();
                headers.insert("X-Forwarded-Proto", "http".parse().unwrap());
                if let Some(host) = headers.get("Host").cloned() {
                    headers.insert("X-Forwarded-Host", host);
                }
                if let Some(for_header) = headers.get("X-Forwarded-For") {
                    let mut addr = for_header.to_str().unwrap().to_owned();
                    addr.push_str(", ");
                    addr.push_str(headers["x-real-ip"].to_str().unwrap());
                    headers.insert("X-Forwarded-For", addr.parse().unwrap());
                } else if let Some(real_ip) = headers.get("x-real-ip").cloned() {
                    headers.insert("X-Forwarded-For", real_ip);
                }

                let host = req.uri().host().expect("uri has no host");
                let port = req.uri().port_u16().unwrap_or(80);
                let addr = format!("{}:{}", host, port);

                let client_stream = TcpStream::connect(addr).await.unwrap();
                let io = TokioIo::new(client_stream);

                let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        println!("Connection failed: {:?}", err);
                    }
                });

                sender.send_request(req).await
            }
        });

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}
