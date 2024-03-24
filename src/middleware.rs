//! Compatibility layer for [`reqwest-middleware`] ecosystem.
//!
//! [`reqwest-middleware`]: https://crates.io/crates/reqwest-middleware

use std::{fmt::Debug, sync::Arc};

use reqwest::Request;
use task_local_extensions::Extensions;
use tower::Service;

type ReqwestMiddleware = Arc<dyn reqwest_middleware::Middleware>;
type RequestWithExtensions<'a> = (Request, &'a mut Extensions);

#[derive(Clone)]
pub struct TowerLayer {
    middleware: ReqwestMiddleware,
}

#[derive(Clone)]
pub struct TowerService {
    middleware: ReqwestMiddleware,
}

impl Debug for TowerLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TowerLayer").finish()
    }
}

impl Debug for TowerService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TowerService").finish()
    }
}

impl TowerLayer {
    pub fn new(middleware: impl Into<ReqwestMiddleware>) -> Self {
        Self {
            middleware: middleware.into(),
        }
    }
}

impl<'a> Service<RequestWithExtensions<'a>> for TowerService {
    type Response;
    type Error;
    type Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, req: Request) -> Self::Future {
        todo!()
    }
}
