//! Adapters for various client types

mod reqwest;
#[cfg(feature = "reqwest-middleware")]
mod reqwest_middleware;
