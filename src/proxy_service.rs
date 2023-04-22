use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::future::Future;
use std::io::Read;
use std::pin::Pin;
use std::sync::{mpsc, Arc, Mutex};
use std::task::{Context, Poll};

use futures::executor::block_on;
use hyper::body::HttpBody;
use hyper::service::Service;
use hyper::{Body, Client, Method, Request, Response, StatusCode};
use serde::Deserialize;
use tokio::sync::oneshot;
use tokio::time::Instant;
use tracing::{debug, error, info};

use crate::db::DBMessage;
use crate::db::DBMessage::Remove;
use crate::models::Tunnel;

pub struct ProxyService {
    pub tunnel_map: Arc<Mutex<HashMap<String, String>>>,
    pub tunnel_vec: Arc<Mutex<Vec<Tunnel>>>,
    pub db_sender: mpsc::Sender<DBMessage>,
}

impl ProxyService {
    async fn proxy(
        mut req: Request<Body>,
        host: String,
        tunnel_host: String,
    ) -> Result<Response<Body>, hyper::Error> {
        let uri = req.uri();

        let query = match uri.query() {
            None => "".to_string(),
            Some(query) => "?".to_string() + query,
        };

        let outgoing_uri = format!("http://{}{}{}", tunnel_host, uri.path(), query);

        info!("{}{} -> {}", host, uri.path(), outgoing_uri);

        *req.uri_mut() = outgoing_uri.parse().unwrap();

        let client = Client::new();

        match client.request(req).await {
            Ok(res) => Ok(res),
            Err(err) => {
                error!("{}", err.to_string().as_str());
                Err(err)
            }
        }
    }

    async fn http(
        db_sender: mpsc::Sender<DBMessage>,
        tunnel_vec: Arc<Mutex<Vec<Tunnel>>>,
        req: Request<Body>,
    ) -> Result<Response<Body>, hyper::Error> {
        debug!("{} - {}", req.method(), req.uri().path());

        match (req.method(), req.uri().path()) {
            (&Method::GET, "/api/") => {
                let tunnel_vec = tunnel_vec.lock().unwrap();

                let tunnels_string = serde_json::to_string(&*tunnel_vec).unwrap();

                let response = Response::builder()
                    .header("Access-Control-Allow-Origin", "*")
                    .body(Body::from(tunnels_string))
                    .expect("Could not create it");

                Ok(response)
            }

            (&Method::POST, "/api/delete/") => {
                let bytes = hyper::body::to_bytes(req.into_body()).await?;

                let bytes: Vec<u8> = bytes.iter().map(|byte| *byte).collect();

                let delete_req: DeleteReq = serde_json::from_slice(&bytes).unwrap();

                db_sender.send(Remove(delete_req.id)).unwrap();

                Ok(Response::builder()
                    .header("Access-Control-Allow-Origin", "*")
                    .body("Ok".into())
                    .unwrap())
            }

            (&Method::GET, mut path) => {
                if path == "/" {
                    path = "/index.html";
                }

                let file_location = format!("./client/dist{}", path);

                info!("loading path at {}", file_location);

                if let Ok(mut file) = File::open(&file_location) {
                    //todo better way

                    let metadata = fs::metadata(&file_location).expect("unable to read metadata");
                    let mut buffer = vec![0; metadata.len() as usize];
                    file.read(&mut buffer).expect("buffer overflow");

                    let body = Body::from(buffer);

                    let mime_type = match file_location.split('.').last().unwrap() {
                        "js" => "text/javascript",
                        "html" => "text/html",
                        "css" => "text/css",
                        _ => "",
                    };

                    let res = Response::builder()
                        .header("Content-Type", mime_type)
                        .body(body)
                        .unwrap();

                    return Ok(res);
                }

                ProxyService::not_found()
            }
            (_, _) => ProxyService::not_found(),
        }
    }

    fn not_found() -> Result<Response<Body>, hyper::Error> {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("not found".to_string()))
            .unwrap())
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

        let new_host = match self.tunnel_map.lock().unwrap().get(&host) {
            None => None,
            Some(host) => Some(host.to_string()),
        };

        let sender = self.db_sender.clone();
        let tunnel_vec = self.tunnel_vec.clone();

        Box::pin(async move {
            let res = match new_host {
                None => ProxyService::http(sender, tunnel_vec, req).await,
                Some(tunnel_host) => ProxyService::proxy(req, host.to_string(), tunnel_host).await,
            };

            info!("response took {:?}", now.elapsed());

            res
        })
    }
}

pub struct MakeProxyService {
    tunnel_map: Arc<Mutex<HashMap<String, String>>>,
    tunnel_vec: Arc<Mutex<Vec<Tunnel>>>,
    db_sender: mpsc::Sender<DBMessage>,
}

impl MakeProxyService {
    pub fn new(db_sender: mpsc::Sender<DBMessage>) -> MakeProxyService {
        let make = MakeProxyService {
            tunnel_map: Arc::new(Mutex::new(HashMap::new())),
            tunnel_vec: Arc::new(Mutex::new(vec![])),
            db_sender,
        };

        make.listen_db_updates();

        make
    }

    fn listen_db_updates(&self) {
        let (sender, receiver) = oneshot::channel();

        self.db_sender
            .send(DBMessage::Subscribe(sender))
            .expect("TODO: panic message");

        let tunnel_map = self.tunnel_map.clone();
        let tunnel_vec = self.tunnel_vec.clone();

        let db_sender = self.db_sender.clone();

        tokio::task::spawn_blocking(move || {
            let mut update_receiver = block_on(receiver).unwrap();

            while let Ok(()) = block_on(update_receiver.recv()) {
                let (sender, receiver) = oneshot::channel();
                db_sender.send(DBMessage::GetALl(sender)).unwrap();

                let tunnels = block_on(receiver).unwrap();

                let mut tunnel_map = tunnel_map.lock().unwrap();
                let mut tunnel_vec = tunnel_vec.lock().unwrap();

                tunnel_map.clear();
                tunnel_vec.clear();

                for tunnel in tunnels {
                    tunnel_map.insert(tunnel.domain_from.clone(), tunnel.domain_to.clone());
                    tunnel_vec.push(tunnel);
                }

                info!("updated tunnel-data");
            }

            debug!("wait");
        });
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
        let tunnel_map = self.tunnel_map.clone();
        let tunnel_vec = self.tunnel_vec.clone();
        let db_sender = self.db_sender.clone();

        Box::pin(async move {
            Ok(ProxyService {
                tunnel_map,
                tunnel_vec,
                db_sender,
            })
        })
    }
}

//todo extra file?
#[derive(Deserialize)]
struct DeleteReq {
    id: i32,
}
