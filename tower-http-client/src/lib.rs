#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![warn(missing_docs)]

//! # Overview
//!
#![doc = include_utils::include_md!("README.md:description")]

#[cfg(feature = "util")]
pub use service_ext::ServiceExt;
pub use tower::BoxError;

#[cfg(feature = "util")]
pub mod body_reader;
#[cfg(feature = "util")]
mod service_ext;
