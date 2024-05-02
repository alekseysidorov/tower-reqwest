//! Adapters for various clients

mod reqwest;
#[cfg(feature = "reqwest-middleware")]
mod reqwest_middleware;
