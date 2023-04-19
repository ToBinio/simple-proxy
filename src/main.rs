use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server};
use tracing::subscriber::set_global_default;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let client = Client::new();

    let uri = req.uri();

    println!("{:?}", req.headers());

    let host = req.headers().get("host").unwrap().to_str().unwrap();
    let origin = match req.headers().get("origin") {
        None => "http",
        Some(origin) => origin.to_str().unwrap().split(':').next().unwrap(),
    };

    let host = host.split(':').next().unwrap();

    let outgoing = origin.to_string() + "://" + host + ":4200" + uri.path();

    info!("{}{} -> {}", host, uri.path(), outgoing);

    let res = client
        .get(outgoing.parse().unwrap())
        .await
        .expect("TODO: panic message");

    Ok(res)
}

#[tokio::main]
async fn main() {
    set_global_default(FmtSubscriber::builder().finish()).expect("could not set default tracer");

    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("server started");

    // Run this server for... forever!
    if let Err(e) = server.await {
        error!("server error: {}", e)
    }
}
