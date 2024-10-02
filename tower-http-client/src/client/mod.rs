//! Extensions for Tower services that provide HTTP clients implementation.

pub use self::{
    body_reader::BodyReader, into_uri::IntoUri, request_builder::ClientRequest,
    response_ext::ResponseExt, service_ext::ServiceExt,
};

pub mod body_reader;
pub mod request_builder;

mod into_uri;
mod response_ext;
mod service_ext;
