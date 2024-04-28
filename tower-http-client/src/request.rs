use std::{future::Future, marker::PhantomData};

use bytes::Bytes;
use http::{Method, Uri, Version};

use crate::ServiceExt;

#[doc(hidden)]
pub trait Captures<U> {}

impl<T: ?Sized, U> Captures<U> for T {}

/// An [`http::Request`] builder.
///
/// In general, this builder copies the behavior of the [`http::request::Builder`],
/// but unlike it, this builder contains a reference to the client and is able to send a
/// constructed request. Also, this builder borrows most useful methods from the [`reqwest`] one.
///
/// [`reqwest`]: https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html
#[derive(Debug)]
pub struct ClientRequest<'a, C, Err, ReqBody, RespBody> {
    client: &'a C,
    builder: http::request::Builder,
    body: ReqBody,
    _phantom: PhantomData<(Err, RespBody)>,
}

impl<'a, C, Err, ReqBody, RespBody> ClientRequest<'a, C, Err, ReqBody, RespBody>
where
    ReqBody: Default,
{
    /// Creates a client request builder.
    pub fn builder(client: &'a C) -> Self {
        Self {
            client,
            builder: http::Request::builder(),
            body: ReqBody::default(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, C, Err, ReqBody, RespBody> ClientRequest<'a, C, Err, ReqBody, RespBody> {
    /// Sets the HTTP method for this request.
    ///
    /// By default this is `GET`.
    #[must_use]
    pub fn method<T>(mut self, method: T) -> Self
    where
        Method: TryFrom<T>,
        <Method as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.method(method);
        self
    }

    /// Sets the URI for this request
    ///
    /// By default this is `/`.
    #[must_use]
    pub fn uri<T>(mut self, uri: T) -> Self
    where
        Uri: TryFrom<T>,
        <Uri as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.uri(uri);
        self
    }

    /// Set the HTTP version for this request.
    ///
    /// By default this is HTTP/1.1.
    #[must_use]
    pub fn version(mut self, version: Version) -> Self {
        self.builder = self.builder.version(version);
        self
    }

    /// Sets an HTTP body for this request.
    pub fn body<NewReqBody>(
        self,
        body: impl Into<NewReqBody>,
    ) -> ClientRequest<'a, C, Err, NewReqBody, RespBody> {
        ClientRequest {
            client: self.client,
            builder: self.builder,
            body: body.into(),
            _phantom: PhantomData,
        }
    }

    /// Consumes this builder and returns a constructed request.
    pub fn build(self) -> Result<http::Request<ReqBody>, http::Error> {
        self.builder.body(self.body)
    }

    /// Constructs the request and sends it to the target URI.
    pub fn send(
        self,
    ) -> Result<
        impl Future<Output = Result<http::Response<RespBody>, Err>> + Captures<&'a ()>,
        http::Error,
    >
    where
        C: ServiceExt<ReqBody, RespBody, Err>,
    {
        let request = self.builder.body(self.body)?;
        Ok(self.client.execute(request))
    }
}
