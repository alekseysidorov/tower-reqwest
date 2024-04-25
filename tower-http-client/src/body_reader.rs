//! Convenient wrapper for reading [`Body`] content.

use std::string::FromUtf8Error;

use bytes::{Buf, Bytes};
use http_body::Body;
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use thiserror::Error;

/// Convenient wrapper for reading [`Body`] content.
#[derive(Debug, Clone)]
pub struct BodyReader<B>(B);

/// Read body errors.
#[derive(Debug, Error)]
#[error(transparent)]
pub enum BodyReaderError<E, D> {
    /// An error occurred while reading the body.
    Read(E),
    /// An error occured while decoding the body content.
    Decode(D),
}

impl<B> BodyReader<B> {
    /// Creates a new reader instance for the given body.
    pub const fn new(body: B) -> Self {
        Self(body)
    }

    /// Reads the full response body as [`Bytes`].
    ///
    /// # Example
    ///
    /// ```
    #[doc = include_str!("../examples/body_reader_bytes.rs")]
    /// ```
    pub async fn bytes(self) -> Result<Bytes, B::Error>
    where
        B: Body,
        B::Data: Buf,
    {
        let body_bytes = self.0.collect().await?.to_bytes();
        Ok(body_bytes)
    }

    /// Reads the full response text.
    ///
    /// # Note
    ///
    /// The method will only attempt to decode the response as `UTF-8`, regardless of the
    /// `Content-Type` header.
    ///
    /// # Example
    ///
    /// ```
    #[doc = include_str!("../examples/body_reader_utf8.rs")]
    /// ```
    pub async fn utf8(self) -> Result<String, BodyReaderError<B::Error, FromUtf8Error>>
    where
        B: Body,
        B::Data: Buf,
    {
        let bytes = self.bytes().await.map_err(BodyReaderError::Read)?;
        String::from_utf8(bytes.into()).map_err(BodyReaderError::Decode)
    }

    /// Deserializes the response body as JSON.
    /// 
    /// # Examples
    /// 
    /// ```
    #[doc = include_str!("../examples/body_reader_json.rs")]
    /// ```
    pub async fn json<T>(self) -> Result<T, BodyReaderError<B::Error, serde_json::Error>>
    where
        T: DeserializeOwned,
        B: Body,
        B::Data: Buf,
    {
        let bytes = self.bytes().await.map_err(BodyReaderError::Read)?;
        serde_json::from_slice(&bytes).map_err(BodyReaderError::Decode)
    }
}
