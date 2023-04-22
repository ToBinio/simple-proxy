use std::env;
use std::net::SocketAddr;

use diesel::{Connection, RunQueryDsl};
use dotenvy::dotenv;
use futures::TryStreamExt;
use hyper::Server;
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
    set_global_default(FmtSubscriber::builder().finish()).expect("could not set default tracer");

    dotenv().ok();

    let addr: [u8;4] = env::var("ADDR").expect("ADDR must be set").split(".").into_iter().map(|part| part.parse().unwrap()).collect::<Vec<u8>>().try_into().unwrap();

    let addr = SocketAddr::from((
        addr,
        env::var("PORT").expect("PORT must be set").parse().unwrap(),
    ));

    let db_sender = DB::start();

    let server = Server::bind(&addr).serve(MakeProxyService::new(db_sender));

    info!("server started");

    if let Err(e) = server.await {
        error!("server error: {}", e)
    }
}
