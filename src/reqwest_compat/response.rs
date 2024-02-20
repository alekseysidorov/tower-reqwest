use futures_util::{StreamExt, TryStreamExt};
use http_body_util::StreamBody;
use hyper::body::{Body, Bytes, Frame};

use super::{
    reqwest_header_name_to_http, reqwest_header_value_to_http, reqwest_status_code_to_http,
    reqwest_version_to_http,
};

pub fn response_body(
    response: reqwest::Response,
) -> impl Body<Data = Bytes, Error = reqwest::Error> {
    let stream = response.bytes_stream().map_ok(Frame::data);
    StreamBody::new(stream)
}

pub fn into_http_response(
    response: reqwest::Response,
) -> http::Result<http::Response<impl Body<Data = Bytes, Error = reqwest::Error>>> {
    // Create response builder.
    let mut builder = http::Response::builder()
        .status(reqwest_status_code_to_http(response.status()))
        .version(reqwest_version_to_http(response.version()));
    
    // Convert headers.
    for (name, value) in response.headers() {
        let name = reqwest_header_name_to_http(name)?;
        let value = reqwest_header_value_to_http(value)?;
        builder = builder.header(name, value);
    }
    // FIXME: There is no way to take the reqwest extensions and move them into http::Extensions.
    // So we just ignore them for now.

    // And finaly consume the request body.
    builder.body(response_body(response))
}
