#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![warn(missing_docs)]

//! # Overview
//!
#![doc = include_utils::include_md!("README.md:description")]
//!

use std::future::Future;

use http::Uri;
use request::ClientRequest;
pub use tower::BoxError;
use tower::Service;

pub mod body_reader;
pub mod request;

/// An extension trait for Tower HTTP services with the typical client methods.
pub trait ServiceExt<ReqBody, RespBody, Err>: Sized {
    /// Executes an HTTP request.
    fn execute<R>(
        &self,
        request: http::Request<R>,
    ) -> impl Future<Output = Result<http::Response<RespBody>, Err>>
    where
        ReqBody: From<R>;

    /// Convenience method to make a `GET` request to a given URL.
    fn get<T>(&self, uri: T) -> ClientRequest<'_, Self, Err, ReqBody, RespBody>
    where
        ReqBody: Default,
        Uri: TryFrom<T>,
        <Uri as TryFrom<T>>::Error: Into<http::Error>,
    {
        ClientRequest::builder(self)
            .method(http::Method::GET)
            .uri(uri)
    }
}

impl<S, ReqBody, RespBody, Err> ServiceExt<ReqBody, RespBody, Err> for S
where
    S: Service<http::Request<ReqBody>, Response = http::Response<RespBody>, Error = Err>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: 'static,
{
    fn execute<R>(
        &self,
        request: http::Request<R>,
    ) -> impl Future<Output = Result<http::Response<RespBody>, Err>>
    where
        ReqBody: From<R>,
    {
        self.clone().call(request.map(ReqBody::from))
    }
}
