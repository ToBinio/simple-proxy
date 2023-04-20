use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::service::Service;
use hyper::{http, Body, Client, Request, Response};
use tracing::info;

pub struct ProxyService {}

impl Service<Request<Body>> for ProxyService {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
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

        Box::pin(async move {
            let res = client.request(req).await.expect("TODO: panic message");
            Ok(res)
        })
    }
}

pub struct MakeProxyService {}

impl<T> Service<T> for MakeProxyService {
    type Response = ProxyService;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let fut = async move { Ok(ProxyService {}) };
        Box::pin(fut)
    }
}
