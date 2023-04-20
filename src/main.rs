use std::env;
use std::fmt::Debug;
use std::net::SocketAddr;

use diesel::{Connection, MysqlConnection, RunQueryDsl};
use dotenvy::dotenv;
use hyper::Server;
use tracing::subscriber::set_global_default;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

use crate::proxy_service::MakeProxyService;
use crate::schema::connections::dsl::connections;

mod models;
mod proxy_service;
mod schema;

#[tokio::main]
async fn main() {
    set_global_default(FmtSubscriber::builder().finish()).expect("could not set default tracer");

    load_data();

    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let server = Server::bind(&addr).serve(MakeProxyService {});

    info!("server started");

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

fn load_data() {
    let mut connetion = establish_connection();

    let results = connections
        .load::<models::Connection>(&mut connetion)
        .expect("Error loading connections");

    for x in results {
        println!("{}", x.id);
    }
}
