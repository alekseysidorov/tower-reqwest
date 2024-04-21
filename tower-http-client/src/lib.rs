#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

//! # Overview
//!
#![doc = include_utils::include_md!("README.md:description")]

#[cfg(feature = "util")]
#[cfg_attr(docsrs, doc(cfg(feature = "util")))]
pub mod util;
