//! Adapters for various clients

pub mod reqwest;
#[cfg(feature = "reqwest-middleware")]
mod reqwest_middleware;
