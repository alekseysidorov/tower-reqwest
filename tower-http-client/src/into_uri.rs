use http::{
    uri::{self},
    Uri,
};

pub trait IntoUri
where
    Uri: TryFrom<Self::Input>,
    <Uri as TryFrom<Self::Input>>::Error: Into<http::Error>,
{
    type Input;
    fn into_uri(self) -> Self::Input;
}

impl IntoUri for &Uri {
    type Input = Self;

    fn into_uri(self) -> Self::Input {
        self
    }
}

impl IntoUri for Uri {
    type Input = Self;

    fn into_uri(self) -> Self::Input {
        self
    }
}

impl IntoUri for String {
    type Input = Self;

    fn into_uri(self) -> Self::Input {
        self
    }
}

impl IntoUri for &String {
    type Input = Self;

    fn into_uri(self) -> Self::Input {
        self
    }
}

impl IntoUri for &str {
    type Input = Self;

    fn into_uri(self) -> Self::Input {
        self
    }
}

impl<'a> IntoUri for &'a Vec<u8> {
    type Input = &'a [u8];

    fn into_uri(self) -> Self::Input {
        self
    }
}

impl IntoUri for Vec<u8> {
    type Input = Self;

    fn into_uri(self) -> Self::Input {
        self
    }
}

impl IntoUri for &[u8] {
    type Input = Self;

    fn into_uri(self) -> Self::Input {
        self
    }
}

impl IntoUri for uri::Parts {
    type Input = Self;

    fn into_uri(self) -> Self::Input {
        self
    }
}

impl IntoUri for url::Url {
    type Input = String;

    fn into_uri(self) -> Self::Input {
        self.into()
    }
}

impl<'a> IntoUri for &'a url::Url {
    type Input = &'a str;

    fn into_uri(self) -> Self::Input {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use http::Uri;
    use url::Url;

    use super::IntoUri as _;

    #[test]
    fn test_url_to_uri_smoke() {
        let example =
            "abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1";

        let url = Url::parse(example).unwrap();
        let expected_uri = Uri::from_static(example);

        let actual_uri: Uri = url.into_uri().parse().expect("failed to convert url");
        assert_eq!(actual_uri, expected_uri);
    }
}
