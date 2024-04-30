//! Useful utilities for constructing HTTP requests.

use std::{any::Any, future::Future, marker::PhantomData};

use http::{Extensions, HeaderMap, HeaderName, HeaderValue, Method, Uri, Version};

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

    /// Appends a header to this request.
    ///
    /// This function will append the provided key/value as a header to the
    /// internal [`HeaderMap`] being constructed.  Essentially this is
    /// equivalent to calling [`HeaderMap::append`].
    #[must_use]
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        HeaderValue: TryFrom<V>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.header(key, value);
        self
    }

    /// Returns a mutable reference to headers of this request builder.
    ///
    /// If builder contains error returns `None`.
    pub fn headers_mut(&mut self) -> Option<&mut HeaderMap<HeaderValue>> {
        self.builder.headers_mut()
    }

    /// Adds an extension to this builder.
    #[must_use]
    pub fn extension<T>(mut self, extension: T) -> Self
    where
        T: Clone + Any + Send + Sync + 'static,
    {
        self.builder = self.builder.extension(extension);
        self
    }

    /// Returns a mutable reference to the extensions of this request builder.
    ///
    /// If builder contains error returns `None`.
    #[must_use]
    pub fn extensions_mut(&mut self) -> Option<&mut Extensions> {
        self.builder.extensions_mut()
    }

    /// Sets an HTTP body for this request.
    ///
    /// Unlike the [`http::request::Builder`] this function doesn't consume builder.
    /// This allows to override the request body.
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
    ///
    /// # Errors
    ///
    /// If erroneous data was passed during the query building process.
    pub fn build(self) -> Result<http::Request<ReqBody>, http::Error> {
        self.builder.body(self.body)
    }
}

impl<'a, C, Err, ReqBody, RespBody> ClientRequest<'a, C, Err, ReqBody, RespBody>
where
    C: ServiceExt<ReqBody, RespBody, Err>,
{
    /// Constructs the request and sends it to the target URI.
    pub fn send(
        self,
    ) -> Result<
        impl Future<Output = Result<http::Response<RespBody>, Err>> + Captures<&'a ()>,
        http::Error,
    > {
        let request = self.builder.body(self.body)?;
        Ok(self.client.execute(request))
    }
}
