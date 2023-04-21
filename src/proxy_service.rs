use diesel::{Connection, MysqlConnection, RunQueryDsl};
use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use crate::models;
use crate::models::Tunnel;
use crate::schema::tunnels::dsl::tunnels;
use hyper::service::Service;
use hyper::{Body, Client, Request, Response};
use tokio::time::Instant;
use tracing::{error, info};

pub struct ProxyService {
    pub tunnel_map: Arc<Mutex<HashMap<String, String>>>,
}

impl ProxyService {
    fn proxy(
        mut req: Request<Body>,
        host: String,
        tunnel_host: String,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send>> {
        let uri = req.uri();

        let query = match uri.query() {
            None => "".to_string(),
            Some(query) => "?".to_string() + query,
        };

        let outgoing_uri = format!("http://{}{}{}", tunnel_host, uri.path(), query);

        info!("{}{} -> {}", host, uri.path(), outgoing_uri);

        *req.uri_mut() = outgoing_uri.parse().unwrap();

        let client = Client::new();

        Box::pin(async move {
            match client.request(req).await {
                Ok(res) => Ok(res),
                Err(err) => {
                    error!("{}", err.to_string().as_str());
                    Err(err)
                }
            }
        })
    }

    fn http(
        _req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send>> {
        Box::pin(async move { Ok(Response::new(Body::from("Servus!"))) })
    }
}

impl Service<Request<Body>> for ProxyService {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let now = Instant::now();

        let host = req
            .headers()
            .get("host")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let res = match self.tunnel_map.lock().unwrap().get(&host) {
            None => ProxyService::http(req),
            Some(tunnel_host) => ProxyService::proxy(req, host.to_string(), tunnel_host.clone()),
        };

        info!("response took {:?}", now.elapsed());

        res
    }
}

pub struct MakeProxyService {
    tunnel_map: Arc<Mutex<HashMap<String, String>>>,
}

impl MakeProxyService {
    pub fn new() -> MakeProxyService {
        let mut tunnel_map = HashMap::new();
        let tunnel_vec = load_data();

        info!("tunnel-data loaded");

        for tunnel in tunnel_vec {
            tunnel_map.insert(tunnel.domain_from, tunnel.domain_to);
        }

        MakeProxyService {
            tunnel_map: Arc::new(Mutex::new(tunnel_map)),
        }
    }
}

impl<T> Service<T> for MakeProxyService {
    type Response = ProxyService;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let tunnel_vec = self.tunnel_map.clone();

        Box::pin(async move {
            Ok(ProxyService {
                tunnel_map: tunnel_vec,
            })
        })
    }
}

pub fn establish_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn load_data() -> Vec<Tunnel> {
    let mut connection = establish_connection();

    tunnels
        .load::<Tunnel>(&mut connection)
        .expect("Error loading connections")
}
