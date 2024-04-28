#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![warn(missing_docs)]

//! # Overview
//!
#![doc = include_utils::include_md!("README.md:description")]
//!

pub use service_ext::ServiceExt;
pub use tower::BoxError;

pub mod body_reader;
mod service_ext;
