#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![warn(missing_docs)]

//! # Overview
//!
#![doc = include_utils::include_md!("README.md:description")]
//!

use std::future::Future;

use http::{Method, Uri};
use request::ClientRequest;
pub use tower::BoxError;
use tower::Service;

#[cfg(feature = "reqwest")]
pub mod adapters;
pub mod body_reader;
pub mod request;

/// An extension trait for Tower HTTP services with the typical client methods.
///
/// Essentially, this trait adds methods similar to those in [`reqwest::Client`] one.
///
/// # Examples
///
/// Creating a client and reading the response body using this trait.
///
#[doc = include_utils::include_md!("README.md:example")]
///
/// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
pub trait ServiceExt<ReqBody, RespBody, Err>: Sized {
    /// Executes an HTTP request.
    fn execute<R>(
        &self,
        request: http::Request<R>,
    ) -> impl Future<Output = Result<http::Response<RespBody>, Err>>
    where
        ReqBody: From<R>;

    /// Starts building a request with the given method and URI.
    fn request<U>(&self, method: Method, uri: U) -> ClientRequest<'_, Self, Err, ReqBody, RespBody>
    where
        ReqBody: Default,
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<http::Error>,
    {
        ClientRequest::builder(self).method(method).uri(uri)
    }

    /// Convenience method to make a `GET` request to a given URL.
    fn get<U>(&self, uri: U) -> ClientRequest<'_, Self, Err, ReqBody, RespBody>
    where
        ReqBody: Default,
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<http::Error>,
    {
        self.request(Method::GET, uri)
    }

    /// Convenience method to make a `PUT` request to a given URL.
    fn put<U>(&self, uri: U) -> ClientRequest<'_, Self, Err, ReqBody, RespBody>
    where
        ReqBody: Default,
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<http::Error>,
    {
        self.request(Method::PUT, uri)
    }

    /// Convenience method to make a `POST` request to a given URL.
    fn post<U>(&self, uri: U) -> ClientRequest<'_, Self, Err, ReqBody, RespBody>
    where
        ReqBody: Default,
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<http::Error>,
    {
        self.request(Method::POST, uri)
    }

    /// Convenience method to make a `PATCH` request to a given URL.
    fn patch<U>(&self, uri: U) -> ClientRequest<'_, Self, Err, ReqBody, RespBody>
    where
        ReqBody: Default,
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<http::Error>,
    {
        self.request(Method::PATCH, uri)
    }

    /// Convenience method to make a `DELETE` request to a given URL.
    fn delete<U>(&self, uri: U) -> ClientRequest<'_, Self, Err, ReqBody, RespBody>
    where
        ReqBody: Default,
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<http::Error>,
    {
        self.request(Method::DELETE, uri)
    }

    /// Convenience method to make a `HEAD` request to a given URL.
    fn head<U>(&self, uri: U) -> ClientRequest<'_, Self, Err, ReqBody, RespBody>
    where
        ReqBody: Default,
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<http::Error>,
    {
        self.request(Method::HEAD, uri)
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

/// Extension trait for the [`http::Response`].
pub trait ResponseExt<T>: Sized {
    /// Consumes the response and returns a body reader wrapper.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tower_http_client::{ResponseExt as _, ServiceExt as _};
    /// use tower_reqwest::HttpClientService;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     // Create a new client
    ///     let client = HttpClientService::new(reqwest::Client::new());
    ///     // Execute request by using this service.
    ///     let response = client.get("http://ip.jsontest.com").send()?.await?;
    ///
    ///     let text = response.body_reader().utf8().await?;
    ///     println!("{text}");
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    fn body_reader(self) -> crate::body_reader::BodyReader<T>;
}

impl<T> ResponseExt<T> for http::Response<T> {
    fn body_reader(self) -> crate::body_reader::BodyReader<T> {
        crate::body_reader::BodyReader::new(self.into_body())
    }
}
