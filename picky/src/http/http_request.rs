use http::header::HeaderName;
use snafu::Snafu;
use std::borrow::Cow;

#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum HttpRequestError {
    /// couldn't convert a http header value to string
    #[snafu(display("couldn't convert http header value to string for header key {}", key))]
    HeaderValueToStr { key: String },

    /// unexpected error occurred
    #[snafu(display("unexpected error: {}", reason))]
    Unexpected { reason: String },
}

pub trait HttpRequest {
    fn get_header_concatenated_values<'a>(&'a self, header_name: &HeaderName)
        -> Result<Cow<'a, str>, HttpRequestError>;
    fn get_lowercased_method(&self) -> Result<Cow<'_, str>, HttpRequestError>;
    fn get_target(&self) -> Result<Cow<'_, str>, HttpRequestError>;
}

impl HttpRequest for http::request::Parts {
    fn get_header_concatenated_values<'a>(
        &'a self,
        header_name: &HeaderName,
    ) -> Result<Cow<'a, str>, HttpRequestError> {
        let mut values = Vec::new();
        let all_values = self.headers.get_all(header_name);
        for value in all_values {
            let value_str = value.to_str().map_err(|_| HttpRequestError::HeaderValueToStr {
                key: header_name.as_str().to_owned(),
            })?;
            values.push(value_str.trim());
        }
        Ok(Cow::Owned(values.join(", ")))
    }

    fn get_lowercased_method(&self) -> Result<Cow<'_, str>, HttpRequestError> {
        Ok(Cow::Owned(self.method.as_str().to_lowercase()))
    }

    fn get_target(&self) -> Result<Cow<'_, str>, HttpRequestError> {
        Ok(Cow::Borrowed(self.uri.path()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{header, method::Method, request};

    #[test]
    fn http_request_parts() {
        let req = request::Builder::new()
            .method(Method::GET)
            .uri("/foo")
            .header("Host", "example.org")
            .header(header::DATE, "Tue, 07 Jun 2014 20:51:35 GMT")
            .header("X-Example", " Example header       with some whitespace.   ")
            .header("X-EmptyHeader", "")
            .header(header::CACHE_CONTROL, "max-age=60")
            .header(header::CACHE_CONTROL, "must-revalidate")
            .body(())
            .expect("couldn't build request");

        let (parts, _) = req.into_parts();

        assert_eq!(parts.get_target().expect("target"), "/foo");
        assert_eq!(parts.get_lowercased_method().expect("method"), "get");
        assert_eq!(
            parts.get_header_concatenated_values(&header::HOST).expect("host"),
            "example.org"
        );
        assert_eq!(
            parts.get_header_concatenated_values(&header::DATE).expect("date"),
            "Tue, 07 Jun 2014 20:51:35 GMT"
        );
        assert_eq!(
            parts
                .get_header_concatenated_values(&HeaderName::from_static("x-example"))
                .expect("example"),
            "Example header       with some whitespace."
        );
        assert_eq!(
            parts
                .get_header_concatenated_values(&HeaderName::from_static("x-emptyheader"))
                .expect("empty"),
            ""
        );
        assert_eq!(
            parts
                .get_header_concatenated_values(&header::CACHE_CONTROL)
                .expect("cache control"),
            "max-age=60, must-revalidate"
        );
    }
}