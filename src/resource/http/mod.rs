// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::str;
use std::io::Read;


// External Dependencies ------------------------------------------------------
use json;
use colored::*;
use hyper::client::Response;
use url::form_urlencoded::Serializer;
use hyper::header::{Header, Headers, HeaderFormat, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};


// Modules --------------------------------------------------------------------
mod endpoint;
mod request;
mod response;
pub mod util;


// Exports --------------------------------------------------------------------
pub use self::request::HttpRequest;
pub use self::endpoint::HttpEndpoint;
pub use self::response::HttpResponse;

/// An abstraction over different data types used for HTTP request bodies.
pub struct HttpBody {
    data: Vec<u8>,
    mime: Option<Mime>
}

#[doc(hidden)]
impl<'a> From<&'a mut Response> for HttpBody {
    fn from(res: &mut Response) -> HttpBody {
        let mut body: Vec<u8> = Vec::new();
        res.read_to_end(&mut body).unwrap();
        HttpBody {
            data: body,
            mime: None
        }
    }
}

impl From<Vec<u8>> for HttpBody {
    /// Creates a HTTP body out of a byte vector.
    fn from(vec: Vec<u8>) -> HttpBody {
        HttpBody {
            data: vec,
            mime: Some(Mime(TopLevel::Application, SubLevel::OctetStream, vec![]))
        }
    }
}

impl From<&'static str> for HttpBody {
    /// Creates a HTTP body out of a string slice.
    fn from(string: &'static str) -> HttpBody {
        HttpBody {
            data: string.into(),
            mime: Some(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
        }
    }
}

impl From<String> for HttpBody {
    /// Creates a HTTP body out of `String`.
    fn from(string: String) -> HttpBody {
        HttpBody {
            data: string.into(),
            mime: Some(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
        }
    }
}

impl From<json::JsonValue> for HttpBody {
    /// Creates a HTTP body out of JSON value.
    fn from(json: json::JsonValue) -> HttpBody {
        HttpBody {
            data: json::stringify(json).into(),
            mime: Some(Mime(TopLevel::Application, SubLevel::Json, vec![]))
        }
    }
}

/// An abstraction over different `hyper::Header` implementations.
///
/// Used by the `headers![...]` macro to easily create a vector containing
/// different types that implement the `hyper::Header` trait.
pub struct HttpHeader {
    name: String,
    value: Vec<u8>
}

impl<H: Header + HeaderFormat> From<H> for HttpHeader {

    /// Converts a implementation of the `hyper::Header` trait into a abstract
    /// representation suitable for use within a `Vec`.
    fn from(header: H) -> HttpHeader {

        let mut headers = Headers::new();
        headers.set(header);

        let name = {
            headers.iter().next().unwrap().name()
        };

        HttpHeader {
            name: name.to_string(),
            value: headers.get_raw(name).unwrap()[0].clone()
        }

    }
}

/// An abstraction over a HTTP query string.
///
/// Created by the `query!{...}` macro:
///
/// ```rust
/// # #[macro_use] extern crate noir;
/// # fn main() {
/// // The query object...
/// let qs = query! {
///     "key" => "value",
///     "array[]" => vec!["item1", "item2","item3"],
///     "number" => 42,
///     "valid" => true
///
/// };
///
/// // ...will result in the following query string:
/// assert_eq!(
///     qs.to_string(),
///     "key=value&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&number=42&valid=true"
/// );
/// # }
/// ```
pub struct HttpQueryString {
    fields: Vec<HttpQueryStringItem>
}

#[doc(hidden)]
impl HttpQueryString {
    pub fn new(fields: Vec<HttpQueryStringItem>) -> HttpQueryString {
        HttpQueryString {
            fields: fields
        }
    }
}

#[doc(hidden)]
impl ToString for HttpQueryString {
    fn to_string(&self) -> String {

        let mut query = Serializer::new(String::new());

        for item in &self.fields {
            match item {
                &HttpQueryStringItem::Value(ref key, ref value) => {
                    query.append_pair(
                        key.as_str(),
                        value.as_str()
                    );
                },
                &HttpQueryStringItem::Array(ref key, ref values) => {
                    for value in values {
                        query.append_pair(
                            key.as_str(),
                            value.as_str()
                        );
                    }
                }
            }
        }

        query.finish()

    }
}

pub enum HttpQueryStringItem {
    Value(String, String),
    Array(String, Vec<String>)
}

macro_rules! impl_query_string_item_type {
    ($T:ty) => (

        impl From<(&'static str, $T)> for HttpQueryStringItem {
            fn from(item: (&'static str, $T)) -> HttpQueryStringItem {
                HttpQueryStringItem::Value(
                    item.0.to_string(),
                    item.1.to_string()
                )
            }
        }

        impl From<(&'static str, Vec<$T>)> for HttpQueryStringItem {
            fn from(item: (&'static str, Vec<$T>)) -> HttpQueryStringItem {
                HttpQueryStringItem::Array(
                    item.0.to_string(),
                    item.1.iter().map(|s| s.to_string()).collect()
                )
            }
        }

    )

}

// TODO IW: Clean up, once impl specialization is stable
impl_query_string_item_type!(&'static str);
impl_query_string_item_type!(String);
impl_query_string_item_type!(bool);
impl_query_string_item_type!(f64);
impl_query_string_item_type!(i64);
impl_query_string_item_type!(u64);
impl_query_string_item_type!(i32);
impl_query_string_item_type!(u32);


// Internal -------------------------------------------------------------------
pub trait HttpLike {
    fn headers(&self) -> &Headers;
    fn into_http_body(&mut self) -> HttpBody where Self: Sized;
}

impl HttpLike for Response {

    fn headers(&self) -> &Headers {
        &self.headers
    }

    fn into_http_body(&mut self) -> HttpBody where Self: Sized {
        self.into()
    }

}

enum ParsedHttpBody<'a> {
    Text(&'a str),
    Json(json::JsonValue),
    Raw(&'a [u8])
}

fn parse_http_body<'a>(headers: &Headers, body: &'a HttpBody) -> Result<ParsedHttpBody<'a>, String> {

    let content_type = headers.get::<ContentType>().and_then(|content_type| {
        Some(content_type.clone())

    }).unwrap_or_else(|| {
        ContentType(Mime(TopLevel::Application, SubLevel::OctetStream, vec![]))
    });

    match content_type.0 {
        Mime(TopLevel::Text, _, _) => {
            match str::from_utf8(body.data.as_slice()) {
                Ok(text) => Ok(ParsedHttpBody::Text(text)),
                Err(err) => Err(format!(
                    "{}\n\n        {}",
                    "text body contains invalid UTF-8:".yellow(),
                    format!("{:?}", err).red().bold()
                ))
            }
        },
        Mime(TopLevel::Application, SubLevel::Json, _) => {
            match parse_json(body.data.as_slice()) {
                Ok(json) => Ok(ParsedHttpBody::Json(json)),
                Err(err) => Err(err)
            }
        },
        _ => {
            Ok(ParsedHttpBody::Raw(&body.data[..]))
        }
    }

}

fn parse_json(data: &[u8]) -> Result<json::JsonValue, String> {
    match str::from_utf8(data) {
        Ok(text) => match json::parse(text) {
            Ok(value) => Ok(value),
            Err(err) => Err(format!(
                "{}\n\n        {}",
                "body contains invalid json:".yellow(),
                format!("{:?}", err).red().bold()
            ))
        },
        Err(err) => Err(format!(
            "{}\n\n        {}",
            "json body contains invalid UTF-8:".yellow(),
            format!("{:?}", err).red().bold()
        ))
    }
}

