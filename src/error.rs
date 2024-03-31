//! When something went wrong.

pub use tower::BoxError;

/// This type represent all possible errors that can occurs during the request processing.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred in the underlying client.
    #[error("Client error: {0}")]
    Client(reqwest::Error),
    /// An error occurred while processing a middleware.
    #[error("Middleware error: {0}")]
    Middleware(BoxError),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Client(value)
    }
}

#[cfg(feature = "reqwest-middleware")]
impl From<reqwest_middleware::Error> for Error {
    fn from(value: reqwest_middleware::Error) -> Self {
        match value {
            reqwest_middleware::Error::Middleware(err) => Self::Middleware(err.into()),
            reqwest_middleware::Error::Reqwest(err) => Self::Client(err),
        }
    }
}
