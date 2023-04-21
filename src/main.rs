use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::models::Tunnel;
use diesel::{Connection, MysqlConnection, RunQueryDsl};
use dotenvy::dotenv;
use hyper::Server;
use tracing::subscriber::set_global_default;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

use crate::proxy_service::MakeProxyService;

mod models;
mod proxy_service;
mod schema;

#[tokio::main]
async fn main() {
    set_global_default(FmtSubscriber::builder().finish()).expect("could not set default tracer");

    dotenv().ok();

    let addr = SocketAddr::from(([127, 0, 0, 1], 80));

    let server = Server::bind(&addr).serve(MakeProxyService::new());

    info!("server started");

    if let Err(e) = server.await {
        error!("server error: {}", e)
    }
}
