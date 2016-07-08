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
use difference;
use hyper::header::Headers;
use hyper::status::StatusCode;


// Internal Dependencies ------------------------------------------------------
use super::{HttpBody, HttpLike, HttpQueryString};
use super::body::{ParsedHttpBody, parse_http_body, validate_http_request_body};
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

pub fn parse_json(data: &[u8]) -> Result<json::JsonValue, String> {
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

pub fn diff_text(context:&str, expected: &str, actual: &str) -> String {

    let diff = difference::diff(expected, actual, " ").1.into_iter().map(|diff| {
        match diff {
            difference::Difference::Same(s) => s,
            difference::Difference::Rem(s) => s.white().on_red().bold().to_string(),
            difference::Difference::Add(s) => s.white().on_green().bold().to_string()
        }

    }).collect::<Vec<String>>().join(" ");

    // TODO escape new lines and other characters?
    format!(
        "{} {}\n\n        {}\n\n    {}\n\n        {}\n\n    {}\n\n        {}",
        context.yellow(),
        "does not match, expected:".yellow(),
        format!("\"{}\"", expected).green().bold(),
        "but got:".yellow(),
        format!("\"{}\"", actual).red().bold(),
        "difference:".yellow(),
        format!("\"{}\"", diff)
    )
}

pub fn validate_http_request<T: HttpLike>(
    result: &mut T,
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
            &mut errors, result, context, expected_body, expected_exact_body
        );
    }

    errors

}

pub fn dump_http_like<T: HttpLike>(
    errors: &mut Vec<String>,
    result: &mut T,
    context: &str
) {

    // Format Headers
    let mut headers = result.headers().iter().map(|header| {
        (header.name().to_string(), header.value_string().clone())

    }).collect::<Vec<(String, String)>>();

    headers.sort();

    let max_name_length = headers.iter().map(|h| h.0.len()).max().unwrap_or(0);
    let headers = headers.into_iter().map(|(name, value)| {
        format!("{: >2$}: {}", name.cyan(), value.purple().bold(), max_name_length)

    }).collect::<Vec<String>>().join("\n        ");

    // Format Body
    let body = result.into_http_body();
    let body = parse_http_body(result.headers(), &body);
    errors.push(match body {
        Ok(actual) => match actual {
            ParsedHttpBody::Text(text) => {
                let text = format!("{:?}", text);
                format!(
                    "{} {}\n\n        {}\n\n    {} {}\n\n        \"{}\"",
                    context.yellow(),
                    "headers dump:".yellow(),
                    headers,
                    context.yellow(),
                    "body dump:".yellow(),
                    &text[1..text.len() - 1].purple().bold()
                )
            },

            ParsedHttpBody::Json(json) => {
                format!(
                    "{} {}\n\n        {}\n\n    {} {}\n\n        {}",
                    context.yellow(),
                    "headers dump:".yellow(),
                    headers,
                    context.yellow(),
                    "body dump:".yellow(),
                    // TODO highlighting
                    json::stringify_pretty(json, 4)
                        .lines()
                        .collect::<Vec<&str>>()
                        .join("\n        ").cyan()
                )
            },

            // Format as 16 columns rows of hexadecimal numbers
            ParsedHttpBody::Raw(data) => {
                format!(
                    "{} {}\n\n        {}\n\n    {} {}\n\n       [{}]",
                    context.yellow(),
                    "headers dump:".yellow(),
                    headers,
                    context.yellow(),
                    format!(
                        "{} {}{}",
                        "raw body dump of".yellow(),
                        format!("{} bytes", data.len()).cyan(),
                        ":".yellow()
                    ),
                    // TODO move out and use when comparing raw bodies
                    //  color must be made configurable (how?)
                    data.chunks(16).map(|c| {
                        c.iter().map(|d| {
                            format!("{}", format!("0x{:0>2X}", d).purple().bold())

                        }).collect::<Vec<String>>().join(", ")

                    }).collect::<Vec<String>>().join(",\n        ")
                )
            }
        },
        Err(err) => {
            format!(
                "{} {}\n\n        {}\n\n    {} {}",
                context.yellow(),
                "headers dump:".yellow(),
                headers,
                context.yellow(),
                err
            )
        }
    })
}

