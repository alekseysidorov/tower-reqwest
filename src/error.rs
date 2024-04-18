//! When something went wrong.

pub use tower::BoxError;

/// This type represent all possible errors that can occurs during the request processing.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred in the underlying client.
    #[error("Client error: {0}")]
    Client(ClientError),
    /// An error occurred while processing a middleware.
    #[error("Middleware error: {0}")]
    Middleware(BoxError),
}

/// An error that can occur while handling HTTP requests.
#[derive(Debug, thiserror::Error)]
#[error("{inner}")]
pub struct ClientError {
    inner: BoxError,
    kind: ClientErrorKind,
}

impl ClientError {
    /// Returns true if the error was caused by a timeout.
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        matches!(self.kind, ClientErrorKind::Timeout)
    }

    /// Returns true if the error is related to connect
    #[must_use]
    pub fn is_connection(&self) -> bool {
        matches!(self.kind, ClientErrorKind::Timeout)
    }

    /// Returns true if the error is related to the request or response body.
    #[must_use]
    pub fn is_body(&self) -> bool {
        matches!(self.kind, ClientErrorKind::Body)
    }
}

#[derive(Debug, Clone, Copy)]
enum ClientErrorKind {
    Timeout,
    Connection,
    Body,
    Other,
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        let kind = if value.is_timeout() {
            ClientErrorKind::Timeout
        } else if value.is_connect() {
            ClientErrorKind::Connection
        } else if value.is_body() {
            ClientErrorKind::Body
        } else {
            ClientErrorKind::Other
        };

        Self {
            inner: Box::new(value),
            kind,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Client(value.into())
    }
}
