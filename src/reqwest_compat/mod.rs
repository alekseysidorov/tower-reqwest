pub use response::{into_http_response, response_body};

mod response;

fn reqwest_status_code_to_http(code: reqwest::StatusCode) -> http::StatusCode {
    let raw = code.as_u16();
    http::StatusCode::from_u16(raw).unwrap()
}

fn reqwest_version_to_http(version: reqwest::Version) -> http::Version {
    if version == reqwest::Version::HTTP_09 {
        return http::Version::HTTP_09;
    } else if version == reqwest::Version::HTTP_10 {
        return http::Version::HTTP_10;
    } else if version == reqwest::Version::HTTP_11 {
        return http::Version::HTTP_11;
    } else if version == reqwest::Version::HTTP_2 {
        return http::Version::HTTP_2;
    } else if version == reqwest::Version::HTTP_3 {
        return http::Version::HTTP_3;
    }

    unreachable!()
}

fn reqwest_header_name_to_http(
    name: &reqwest::header::HeaderName,
) -> Result<http::header::HeaderName, http::header::InvalidHeaderName> {
    let raw = name.as_str().as_bytes();
    http::header::HeaderName::from_bytes(raw)
}

fn reqwest_header_value_to_http(
    value: &reqwest::header::HeaderValue,
) -> Result<http::header::HeaderValue, http::header::InvalidHeaderValue> {
    let raw = value.as_bytes();
    http::header::HeaderValue::from_bytes(raw)
}
