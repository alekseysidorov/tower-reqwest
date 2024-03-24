//! Compatibility layer for [`reqwest-middleware`] ecosystem.
//!
//! [`reqwest-middleware`]: https://crates.io/crates/reqwest-middleware

use std::{
    sync::Arc,
    task::{Context, Poll},
};

use futures_util::{future::BoxFuture, FutureExt};
use reqwest::{Client, Request, Response};
use reqwest_middleware::{ClientWithMiddleware, Middleware};
use tower::Service;

#[derive(Clone)]
pub struct ServiceWrapper(ClientWithMiddleware);

impl From<ClientWithMiddleware> for ServiceWrapper {
    fn from(value: ClientWithMiddleware) -> Self {
        Self(value)
    }
}

impl From<(Client, Vec<Arc<dyn Middleware>>)> for ServiceWrapper {
    fn from((client, middleware_stack): (Client, Vec<Arc<dyn Middleware>>)) -> Self {
        Self::from(ClientWithMiddleware::new(client, middleware_stack))
    }
}

impl Service<Request> for ServiceWrapper {
    type Response = Response;
    type Error = reqwest_middleware::Error;
    // TODO Rewrite without boxing.
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let client = self.0.clone();
        async move { client.execute(req).await }.boxed()
    }
}

impl Service<Request> for &ServiceWrapper {
    type Response = Response;
    type Error = reqwest_middleware::Error;
    // TODO Rewrite without boxing.
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let client = self.0.clone();
        async move { client.execute(req).await }.boxed()
    }
}
