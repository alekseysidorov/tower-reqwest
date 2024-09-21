#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![warn(missing_docs)]

//! # Overview
//!
#![doc = include_utils::include_md!("README.md:description")]
//!
//! An example of multi-threaded concurrent requests sending routine with the requests rate limit.
//!
//! ```rust
#![doc = include_str!("../examples/rate_limiter.rs")]
//! ```

pub use into_uri::IntoUri;
pub use response_ext::ResponseExt;
pub use service_ext::ServiceExt;

#[cfg(feature = "reqwest")]
pub mod adapters;
pub mod body_reader;
pub mod request_builder;
#[cfg(feature = "util")]
pub mod util;

mod into_uri;
mod response_ext;
mod service_ext;
