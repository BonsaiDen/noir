// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::str;


// External Dependencies ------------------------------------------------------
use json;
use colored::*;
use hyper::header::{Headers, ContentType};
use hyper::status::StatusCode;


// Internal Dependencies ------------------------------------------------------
use util;
use Options;
use super::{HttpBody, HttpLike, HttpQueryString};
use super::form::{HttpFormDataField, http_form_into_fields};
use super::body::{
    ParsedHttpBody,
    parse_http_body,
    http_body_from_parts,
    validate_http_request_body
};
use super::header::validate_http_request_headers;


// Re-Exports -----------------------------------------------------------------
pub use super::header::http_header_into_tuple;
pub use super::body::http_body_into_parts;


// HTTP related utilities -----------------------------------------------------
pub fn path_with_query(path: &str, query: HttpQueryString) -> String {

    let new_query = query.to_string();

    // Ignore hash and old query string
    let (path, _) = path.split_at(path.find('#').unwrap_or(path.len()));
    let (path, _) = path.split_at(path.find('?').unwrap_or(path.len()));

    // Insert new query string
    if new_query.is_empty() {
        path.to_string()

    } else {
        format!("{}?{}", path, new_query)
    }

}

pub fn validate_http_request<T: HttpLike>(
    result: &mut T,
    options: &Options,
    context: &str,
    response_status: Option<StatusCode>,
    expected_status: Option<StatusCode>,
    expected_headers: &Headers,
    unexpected_headers: &mut Vec<String>,
    expected_body: &Option<HttpBody>,
    expected_exact_body: bool

) -> Vec<String> {

    let mut errors = Vec::new();

    if let Some(actual) = response_status {
        if let Some(expected) = expected_status {
            if actual != expected {
                errors.push(format!(
                    "{} {}\n\n        \"{}\"\n\n    {}\n\n        \"{}\"",
                    context.yellow(),
                    "status code does not match value, expected:".yellow(),
                    format!("{}", expected).green().bold(),
                    "but got:".yellow(),
                    format!("{}", actual).red().bold()
                ));
            }
        }
    }

    validate_http_request_headers(
        &mut errors, result, context, expected_headers, unexpected_headers
    );

    if let Some(ref expected_body) = expected_body.as_ref() {
        validate_http_request_body(
            &mut errors,
            options,
            result,
            context,
            expected_body,
            expected_exact_body
        );
    }

    errors

}

pub fn dump_http_like<T: HttpLike>(
    errors: &mut Vec<String>,
    result: &mut T,
    context: &str
) {

    let headers = format_headers(result);
    let body = result.into_http_body();

    errors.push(
        format!(
            "{} {}\n\n        {}\n\n    {} {}",
            context.yellow(),
            "headers dump:".yellow(),
            headers,
            context.yellow(),
            format_body(parse_http_body(&body))
        )
    )

}

fn format_headers<T: HttpLike>(result: &mut T) -> String {

    let mut headers = result.headers().iter().map(|header| {
        (header.name().to_string(), header.value_string().clone())

    }).collect::<Vec<(String, String)>>();

    headers.sort();

    let max_name_length = headers.iter().map(|h| h.0.len()).max().unwrap_or(0);

    headers.into_iter().map(|(name, value)| {
        format!("{: >2$}: {}", name.cyan(), value.purple().bold(), max_name_length)

    }).collect::<Vec<String>>().join("\n        ")

}

fn format_body(body: Result<ParsedHttpBody, String>) -> String {
    match body {
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
                                format!("{:2})", i + 1).cyan().bold(),
                                "Field".yellow(),
                                name.cyan(),
                                "dump:".yellow(),
                                &value[1..value.len() - 1].purple().bold()
                            )
                        },
                        HttpFormDataField::Array(name, values) => {
                            format!(
                                "{} {} \"{}\" ({}) {}\n\n              {}\n",
                                format!("{:2})", i + 1).cyan().bold(),
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
                            let body = format_body(
                                parse_http_body(&body)

                            ).split("\n").map(|line| {
                                format!("      {}", line)

                            }).collect::<Vec<String>>().join("\n");

                            format!(
                                "{} {} \"{}\" (\"{}\", {}) {}\n",
                                format!("{:2})", i + 1).cyan().bold(),
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

