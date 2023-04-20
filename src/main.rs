use std::convert::Infallible;
use std::env;
use std::fmt::Debug;
use std::net::SocketAddr;

use diesel::{Connection, MysqlConnection, RunQueryDsl};
use dotenvy::dotenv;
use hyper::{Body, Client, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use tracing::{error, info};
use tracing::subscriber::set_global_default;
use tracing_subscriber::FmtSubscriber;

use crate::schema::connections::dsl::connections;

mod models;
mod schema;

async fn hello_world(mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let client = Client::new();

    let uri = req.uri();

    let host = req.headers().get("host").unwrap().to_str().unwrap();
    let host = host.split(':').next().unwrap();

    let query = match uri.query() {
        None => "".to_string(),
        Some(query) => "?".to_string() + query,
    };

    let outgoing = "http://".to_string() + host + ":4200" + uri.path() + query.as_str();

    info!("{}{} -> {}", host, uri.path(), outgoing);

    *req.uri_mut() = outgoing.parse().unwrap();

    let res = client.request(req).await.expect("TODO: panic message");

    Ok(res)
}

#[tokio::main]
async fn main() {
    set_global_default(FmtSubscriber::builder().finish()).expect("could not set default tracer");

    load_data();

    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

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

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn load_data(){

    let mut connetion =  establish_connection();

    let resulst = connections.load::<models::Connection>(&mut connetion).expect("Error loading connections");

    for x in resulst {
        println!("{}", x.id);
    }

}
