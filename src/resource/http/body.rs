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
use Options;
use super::HttpFormData;
use super::form::{
    HttpFormDataField,
    http_form_into_body_parts,
    http_form_into_fields,
    parse_form_data
};


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
        // If the res body is empty, this will fail so we simply ignore it
        res.read_to_end(&mut body).ok();
        http_body_from_parts(body, &res.headers)
    }
}

impl From<Vec<u8>> for HttpBody {
    /// Creates a HTTP body from a byte vector.
    ///
    /// # Test Failure Examples
    ///
    /// [expanded](terminal://body_expected_raw_mismatch)
    fn from(vec: Vec<u8>) -> HttpBody {
        HttpBody {
            data: vec,
            mime: Some(Mime(TopLevel::Application, SubLevel::OctetStream, vec![]))
        }
    }
}

impl From<&'static str> for HttpBody {
    /// Creates a HTTP body from a string slice.
    ///
    /// # Test Failure Examples
    ///
    /// [expanded](terminal://body_text_mismatch_diff_added)
    fn from(string: &'static str) -> HttpBody {
        HttpBody {
            data: string.into(),
            mime: Some(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
        }
    }
}

impl From<String> for HttpBody {
    /// Creates a HTTP body from a `String`.
    ///
    /// # Test Failure Examples
    ///
    /// [expanded](terminal://body_text_mismatch_diff_removed)
    fn from(string: String) -> HttpBody {
        HttpBody {
            data: string.into(),
            mime: Some(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
        }
    }
}

impl From<json::JsonValue> for HttpBody {
    /// Creates a HTTP body from a JSON value.
    ///
    /// # Test Failure Examples
    ///
    /// [expanded](terminal://body_with_expected_json_mismatch)
    fn from(json: json::JsonValue) -> HttpBody {
        HttpBody {
            data: json::stringify(json).into(),
            mime: Some(Mime(TopLevel::Application, SubLevel::Json, vec![]))
        }
    }
}

impl From<HttpFormData> for HttpBody {
    /// Creates a HTTP body from form data.
    ///
    /// # Test Failure Examples
    ///
    /// [expanded](terminal://provided_response_with_expected_body_form_mismatch)
    fn from(form: HttpFormData) -> HttpBody {
        let (mime_type, body) = http_form_into_body_parts(form);
        HttpBody {
            data: body,
            mime: Some(mime_type)
        }
    }
}


// Parsing --------------------------------------------------------------------
enum ParsedHttpBody<'a> {
    Text(&'a str),
    Json(json::JsonValue),
    Form(HttpFormData),
    Raw(&'a [u8])
}

fn parse_http_body(body: &HttpBody) -> Result<ParsedHttpBody, String> {

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
                match util::json::parse(body.data.as_slice(), "body JSON") {
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
                    Err(err) => Err(format!(
                        "{}\n\n        {}",
                        "form body could not be parsed:".yellow(),
                         err.red().bold()
                    ))
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


// Formatting -----------------------------------------------------------------
pub fn format_http_body(body: &HttpBody) -> String {
    match parse_http_body(body) {
        Ok(actual) => match actual {
            ParsedHttpBody::Text(text) => {
                let text = format!("{:?}", text);
                format!(
                    "{}\n\n        \"{}\"",
                    "body dump:".yellow(),
                    &text[1..text.len() - 1].purple().bold()
                )
            },

            ParsedHttpBody::Json(json) => {
                format!(
                    "{}\n\n        {}",
                    "body dump:".yellow(),
                    // TODO IW: Support JSON highlighting
                    json::stringify_pretty(json, 4)
                        .lines()
                        .collect::<Vec<&str>>()
                        .join("\n        ").cyan()
                )
            },

            ParsedHttpBody::Form(form) => {

                let fields = http_form_into_fields(form);
                let field_count = fields.len();

                let fields = fields.into_iter().enumerate().map(|(i, field)| {
                    match field {
                        HttpFormDataField::Field(name, value) => {
                            let value = format!("{:?}", value);
                            format!(
                                "{} {} \"{}\" {}\n\n              \"{}\"\n",
                                format!("{:2})", i + 1).blue().bold(),
                                "Field".yellow(),
                                name.cyan(),
                                "dump:".yellow(),
                                &value[1..value.len() - 1].purple().bold()
                            )
                        },
                        HttpFormDataField::Array(name, values) => {
                            format!(
                                "{} {} \"{}\" ({}) {}\n\n              {}\n",
                                format!("{:2})", i + 1).blue().bold(),
                                "Array".yellow(),
                                name.cyan(),
                                format!("{} items", values.len()).purple().bold(),
                                "dump:".yellow(),
                                values.into_iter().map(|value| {
                                    let value = format!("{:?}", value);
                                    format!("\"{}\"", &value[1..value.len() - 1].purple().bold())

                                }).collect::<Vec<String>>().join(", ")
                            )
                        },
                        HttpFormDataField::FileVec(name, filename, mime, data) => {

                            // Parse file data into a HttpBody
                            let mut headers = Headers::new();
                            headers.set(ContentType(mime.clone()));
                            let body = http_body_from_parts(data, &headers);

                            // Format the body
                            let body = format_http_body(&body).split('\n').map(|line| {
                                format!("      {}", line)

                            }).collect::<Vec<String>>().join("\n");

                            format!(
                                "{} {} \"{}\" (\"{}\", {}) {}\n",
                                format!("{:2})", i + 1).blue().bold(),
                                "File".yellow(),
                                name.cyan(),
                                filename.purple().bold(),
                                format!("{}", mime).purple().bold(),
                                body.trim_left()
                            )

                        },
                        _ => unreachable!()
                    }

                }).collect::<Vec<String>>().join("\n        ");

                format!(
                    "{}\n\n        {}",
                    format!(
                        "{} {}{}",
                        "form dump with".yellow(),
                        format!("{} fields", field_count).cyan(),
                        ":".yellow()
                    ),
                    fields
                )

            },

            // Format as 16 column wide rows of hexadecimal numbers
            ParsedHttpBody::Raw(data) => {
                format!(
                    "{}\n\n       [{}]",
                    format!(
                        "{} {}{}",
                        "raw body dump of".yellow(),
                        format!("{} bytes", data.len()).cyan(),
                        ":".yellow()
                    ),
                    util::raw::format_purple(data)
                )
            }
        },
        Err(err) => err
    }
}


// Validation -----------------------------------------------------------------
pub fn validate_http_body(
    errors: &mut Vec<String>,
    context: &str,
    expected_body: &HttpBody,
    actual_body: HttpBody,
    compare_exact: bool,
    options: &Options
) {

    // Quick check before we perform heavy weight parsing
    // This might cause false negative for JSON, but we'll perform a deep
    // compare anyway.
    if actual_body.data == expected_body.data {
        return;
    }

    // Parse and compare different body types
    let mut error = match parse_http_body(&actual_body) {
        Ok(actual) => match actual {
            ParsedHttpBody::Text(actual) => compare_text_body(
                context, expected_body, actual
            ),
            ParsedHttpBody::Json(actual) => compare_json_body(
                context, expected_body, actual,
                compare_exact,
                options.json_compare_depth
            ),
            ParsedHttpBody::Form(actual) => compare_form_body(
                context, expected_body, actual,
                compare_exact,
                options
            ),
            ParsedHttpBody::Raw(actual) => compare_raw_body(
                context, expected_body, actual
            )
        },
        Err(err) => Some(format!("{} {}", context.yellow(), err))
    };

    if let Some(error) = error.take() {
        errors.push(error);
    }

}

fn compare_text_body(
    context: &str,
    expected: &HttpBody,
    actual: &str

) -> Option<String> {
    match str::from_utf8(expected.data.as_slice()) {
        Ok(expected) => {

            let (expected, actual, diff) = util::diff::text(
                expected,
                actual
            );

            Some(format!(
                "{} {}\n\n        \"{}\"\n\n    {}\n\n        \"{}\"\n\n    {}\n\n        \"{}\"",
                context.yellow(),
                "text body does not match, expected:".yellow(),
                expected.green().bold(),
                "but got:".yellow(),
                actual.red().bold(),
                "difference:".yellow(),
                diff
            ))

        },
        Err(err) => Some(format!(
            "{} {}\n\n        {}",
            context.yellow(),
            "body, expected text provided by test contains invalid UTF-8:".yellow(),
            format!("{:?}", err).red().bold()
        ))
    }
}

fn compare_json_body(
    context: &str,
    expected: &HttpBody,
    actual: json::JsonValue,
    compare_exact: bool,
    compare_depth: usize

) -> Option<String> {

    let expected_json = util::json::parse(
        expected.data.as_slice(),
        "body JSON provided by test"
    );

    match expected_json {
        Ok(expected) => {

            let errors = util::json::compare(
                &expected,
                &actual,
                compare_depth,
                compare_exact
            );

            // Exit early when there are no errors
            if errors.is_ok() {
                None

            } else {
                Some(format!(
                    "{} {}\n\n        {}",
                    context.yellow(),
                    "body JSON does not match:".yellow(),
                    util::json::format(errors.unwrap_err())
                ))
            }

        },
        Err(err) => Some(format!(
            "{} {} {}",
            context.yellow(),
            "body, expected".yellow(),
            err
        ))
    }

}

fn compare_form_body(
    context: &str,
    expected: &HttpBody,
    actual: HttpFormData,
    compare_exact: bool,
    options: &Options

) -> Option<String> {
    match parse_http_body(expected) {
        Ok(ParsedHttpBody::Form(expected)) => {

            let expected = http_form_into_fields(expected);
            let actual = http_form_into_fields(actual);
            let errors = util::form::compare(
                &expected,
                &actual,
                compare_exact,
                options
            );

            // Exit early when there are no errors
            if errors.is_ok() {
                None

            } else {
                Some(format!(
                    "{} {}\n\n        {}",
                    context.yellow(),
                    "body form data does not match:".yellow(),
                    util::form::format(errors.unwrap_err())
                ))
            }


        },
        _ => unreachable!()
    }
}

fn compare_raw_body(
    context: &str,
    expected: &HttpBody,
    actual: &[u8]

) -> Option<String> {
    Some(format!(
        "{} {}\n\n       [{}]\n\n    {}\n\n       [{}]",
        context.yellow(),
        format!(
            "{} {}{}",
            "raw body data does not match, expected the following".yellow(),
            format!("{} bytes", expected.data.len()).green().bold(),
            ":".yellow()
        ),
        util::raw::format_green(expected.data.as_slice()),
        format!(
            "{} {} {}",
            "but got the following".yellow(),
            format!("{} bytes", actual.len()).red().bold(),
            "instead:".yellow()
        ),
        util::raw::format_red(actual),
    ))
}

