// pub trait Layer<S> {
// type Service;

// fn layer(&self, inner: S) -> Self::Service;
// }

use std::net::SocketAddr;

use hyper::{service::make_service_fn, Body, Error, Request, Server};
use tower::{Layer, Service, ServiceBuilder};

pub struct LoggerLayer {
    target: &'static str,
}

impl<S> Layer<S> for LoggerLayer {
    type Service = LoggerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggerService {
            target: "test",
            service: inner,
        }
    }
}

pub struct LoggerService<S> {
    target: &'static str,
    service: S,
}

impl<S, Request> Service<Request> for LoggerService<S>
where
    S: Service<Request>,
    Request: std::fmt::Debug,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        println!("[LoggerService] send req: {:?}", req);
        self.service.call(req)
    }
}

#[tokio::main]
async fn main() {
    let mut client = ServiceBuilder::new()
        .layer(LoggerLayer { target: "test" })
        .service(hyper::Client::new());

    let request = Request::builder()
        .uri("http://captive.apple.com:80")
        .body(Body::empty())
        .unwrap();

    let response = client.call(request).await.unwrap();
    println!("response: {:?}", response);
}
