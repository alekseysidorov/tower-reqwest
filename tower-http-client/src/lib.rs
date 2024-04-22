#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

//! # Overview
//!
#![doc = include_utils::include_md!("README.md:description")]

#[cfg(feature = "util")]
#[doc(inline)]
pub use service_ext::ServiceExt;

#[cfg(feature = "util")]
mod service_ext;
