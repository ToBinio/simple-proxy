use std::net::SocketAddr;

use diesel::{Connection, RunQueryDsl};
use dotenvy::dotenv;
use hyper::Server;
use tracing::metadata::LevelFilter;
use tracing::subscriber::set_global_default;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

use crate::db::DB;
use crate::proxy_service::MakeProxyService;

mod db;
mod models;
mod proxy_service;
mod schema;

#[tokio::main]
async fn main() {
    set_global_default(
        FmtSubscriber::builder()
            .with_max_level(LevelFilter::DEBUG)
            .finish(),
    )
    .expect("could not set default tracer");

    dotenv().ok();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let db_sender = DB::start();

    let server = Server::bind(&addr).serve(MakeProxyService::new(db_sender));

    info!("server started");

    if let Err(e) = server.await {
        error!("server error: {}", e)
    }
}
