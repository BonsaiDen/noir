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
use url::Url;
use colored::*;
use hyper::client::Response;
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
    data: Vec<u8>
}

#[doc(hidden)]
impl<'a> From<&'a mut Response> for HttpBody {
    fn from(res: &mut Response) -> HttpBody {
        let mut body: Vec<u8> = Vec::new();
        res.read_to_end(&mut body).unwrap();
        HttpBody {
            data: body
        }
    }
}

impl From<Vec<u8>> for HttpBody {
    /// Creates a HTTP body out of a byte vector.
    fn from(vec: Vec<u8>) -> HttpBody {
        HttpBody {
            data: vec
        }
    }
}

impl From<&'static str> for HttpBody {
    /// Creates a HTTP body out of a string slice.
    fn from(string: &'static str) -> HttpBody {
        HttpBody {
            data: string.into()
        }
    }
}

impl From<String> for HttpBody {
    /// Creates a HTTP body out of `String`.
    fn from(string: String) -> HttpBody {
        HttpBody {
            data: string.into()
        }
    }
}

impl From<json::JsonValue> for HttpBody {
    /// Creates a HTTP body out of JSON value.
    fn from(json: json::JsonValue) -> HttpBody {
        HttpBody {
            data: json::stringify(json).into()
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
/// let qs: Option<String> = (query! {
///     "key" => "value",
///     "array[]" => vec!["item1", "item2","item3"],
///     "number" => 42,
///     "valid" => true
///
/// }).into();
///
/// // ...will result in the following query string:
/// assert_eq!(
///     qs.unwrap(),
///     "key=value&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&number=42&valid=true"
/// );
/// # }
/// ```
pub struct HttpQueryString {
    items: Vec<HttpQueryStringItem>
}

#[doc(hidden)]
impl HttpQueryString {
    pub fn new(items: Vec<HttpQueryStringItem>) -> HttpQueryString {
        HttpQueryString {
            items: items
        }
    }
}

#[doc(hidden)]
impl Into<Option<String>> for HttpQueryString {
    fn into(self) -> Option<String> {

        let mut uri = Url::parse("http://query.string/").unwrap();

        for item in self.items {
            match item {
                HttpQueryStringItem::Value(key, value) => {
                    uri.query_pairs_mut().append_pair(
                        key.as_str(),
                        value.as_str()
                    );
                },
                HttpQueryStringItem::Array(key, values) => {
                    for value in values {
                        uri.query_pairs_mut().append_pair(
                            key.as_str(),
                            value.as_str()
                        );
                    }
                }
            }
        }

        match uri.query() {
            Some(query) => Some(query.to_string()),
            None => None
        }

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

