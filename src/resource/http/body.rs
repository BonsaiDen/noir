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
use hyper::header::{Headers, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};


// Internal Dependencies ------------------------------------------------------
use util;
use super::{HttpLike, HttpFormData};
use super::form::{http_form_into_body_parts, parse_form_data};
use super::util::{parse_json, diff_text};


/// An abstraction over different data types used for HTTP request bodies.
pub struct HttpBody {
    data: Vec<u8>,
    mime: Option<Mime>
}


// Internal -------------------------------------------------------------------
pub fn http_body_from_parts(data: Vec<u8>, headers: &Headers) -> HttpBody {

    let mime = if let Some(&ContentType(ref mime)) = headers.get::<ContentType>() {
        mime.clone()

    } else {
        Mime(TopLevel::Application, SubLevel::OctetStream, vec![])
    };

    HttpBody {
        data: data,
        mime: Some(mime)
    }

}

pub fn http_body_into_parts(body: HttpBody) -> (Option<Mime>, Option<Vec<u8>>) {
    (body.mime, Some(body.data))
}

#[doc(hidden)]
impl<'a> From<&'a mut Response> for HttpBody {
    fn from(res: &mut Response) -> HttpBody {
        let mut body: Vec<u8> = Vec::new();
        res.read_to_end(&mut body).unwrap();
        http_body_from_parts(body, &res.headers)
    }
}

impl From<Vec<u8>> for HttpBody {
    /// Creates a HTTP body from a byte vector.
    fn from(vec: Vec<u8>) -> HttpBody {
        HttpBody {
            data: vec,
            mime: Some(Mime(TopLevel::Application, SubLevel::OctetStream, vec![]))
        }
    }
}

impl From<&'static str> for HttpBody {
    /// Creates a HTTP body from a string slice.
    fn from(string: &'static str) -> HttpBody {
        HttpBody {
            data: string.into(),
            mime: Some(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
        }
    }
}

impl From<String> for HttpBody {
    /// Creates a HTTP body from a `String`.
    fn from(string: String) -> HttpBody {
        HttpBody {
            data: string.into(),
            mime: Some(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
        }
    }
}

impl From<json::JsonValue> for HttpBody {
    /// Creates a HTTP body from a JSON value.
    fn from(json: json::JsonValue) -> HttpBody {
        HttpBody {
            data: json::stringify(json).into(),
            mime: Some(Mime(TopLevel::Application, SubLevel::Json, vec![]))
        }
    }
}

impl From<HttpFormData> for HttpBody {
    /// Creates a HTTP body from form data.
    fn from(form: HttpFormData) -> HttpBody {
        let (mime_type, body) = http_form_into_body_parts(form);
        HttpBody {
            data: body,
            mime: Some(mime_type)
        }
    }
}

pub fn validate_http_request_body<T: HttpLike>(
    errors: &mut Vec<String>,
    result: &mut T,
    context: &str,
    expected_body: &&HttpBody,
    expected_exact_body: bool
) {

    // Quick check before we perform heavy weight parsing
    // This might cause false negative for JSON, but we'll perform a deep
    // compare anyway.
    let body = result.into_http_body();
    if body.data == expected_body.data {
        return;
    }

    // Parse and compare different body types
    let body = parse_http_body(&body);
    errors.push(match body {

        Ok(actual) => match actual {
            ParsedHttpBody::Text(text) => {
                diff_text(
                    format!("{} body text", context).as_str(),
                    str::from_utf8(expected_body.data.as_slice()).unwrap(),
                    text
                )
            },
            ParsedHttpBody::Json(actual) => {
                let expected_json = parse_json(expected_body.data.as_slice());
                match expected_json {
                    Ok(expected) => {

                        let errors = util::json::compare(
                            &expected,
                            &actual,
                            4096, // TODO configure max depth
                            expected_exact_body
                        );

                        // Exit early when there are no errors
                        if errors.is_ok() {
                            return;
                        }

                        format!(
                            "{} {}\n\n        {}",
                            context.yellow(),
                            "body json does not match, expected:".yellow(),
                            util::json::format(errors.unwrap_err())
                        )

                    },
                    Err(err) => {
                        format!(
                            "{} {}\n\n        {}",
                            context.yellow(),
                            "body contains invalid json data:".yellow(),
                            err
                        )
                    }

                }

            },
            ParsedHttpBody::Form(form) => {
                // TODO IW: implement form field diffing
                format!(
                    "{} {}\n\n        {}",
                    context.yellow(),
                    "body contains form data:".yellow(),
                    "Form data diffing is not yet implemented.".red().bold()
                )
            },
            ParsedHttpBody::Raw(data) => {
                format!(
                    "{} {}\n\n       [{}]\n\n    {}\n\n       [{}]",

                    context.yellow(),

                    format!(
                        "{} {}{}",
                        "raw body data does not match, expected the following".yellow(),
                        format!("{} bytes", expected_body.data.len()).green().bold(),
                        ":".yellow()
                    ),

                    // TODO dry or better diff
                    expected_body.data.chunks(16).map(|c| {
                        c.iter().map(|d| {
                            format!("{}", format!("0x{:0>2X}", d).green().bold())

                        }).collect::<Vec<String>>().join(", ")

                    }).collect::<Vec<String>>().join(",\n        "),

                    format!(
                        "{} {} {}",
                        "but got the following".yellow(),
                        format!("{} bytes", data.len()).red().bold(),
                        "instead:".yellow()
                    ),

                    // TODO dry or better diff
                    data.chunks(16).map(|c| {
                        c.iter().map(|d| {
                            format!("{}", format!("0x{:0>2X}", d).red().bold())

                        }).collect::<Vec<String>>().join(", ")

                    }).collect::<Vec<String>>().join(",\n        ")

                )
            }
        },

        Err(err) => {
            format!("{} {}{}", context.yellow(), err, ".".to_string())
        }

    });

}

pub enum ParsedHttpBody<'a> {
    Text(&'a str),
    Json(json::JsonValue),
    Form(HttpFormData),
    Raw(&'a [u8])
}

pub fn parse_http_body<'a>(body: &'a HttpBody) -> Result<ParsedHttpBody<'a>, String> {

    if let Some(mime) = body.mime.as_ref() {
        match mime.clone() {
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
            Mime(TopLevel::Application, SubLevel::FormData, attrs) |
            Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, attrs) => {

                // Get form-data boundary, if present
                let boundary = attrs.get(0).map(|b| {
                    b.1.as_str().to_string()
                });

                match parse_form_data(body.data.as_slice(), boundary) {
                    Ok(data) => Ok(ParsedHttpBody::Form(data)),
                    Err(err) => Err(err)
                }

            },
            _ => {
                Ok(ParsedHttpBody::Raw(&body.data[..]))
            }
        }

    } else {
        Ok(ParsedHttpBody::Raw(&body.data[..]))
    }
}
