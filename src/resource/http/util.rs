// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::str;


// External Dependencies ------------------------------------------------------
use colored::*;
use hyper::mime::Mime;
use hyper::status::StatusCode;
use hyper::header::{Headers, ContentType};


// Internal Dependencies ------------------------------------------------------
use Options;
use super::{HttpBody, HttpResource, HttpQueryString};
use super::body::{http_body_from_parts, format_http_body, validate_http_body};
use super::header::{validate_http_headers, format_http_headers};


// Re-Exports -----------------------------------------------------------------
pub use super::body::http_body_into_parts;
pub use super::header::http_header_into_tuple;


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

pub fn validate_http_resource<T: HttpResource>(
    context: &str,
    expected_status: Option<StatusCode>,
    expected_headers: &Headers,
    unexpected_headers: &mut Vec<String>,
    expected_body: &Option<HttpBody>,
    actual: &mut T,
    actual_status: Option<StatusCode>,
    compare_exact: bool,
    options: &Options

) -> Vec<String> {

    let mut errors = Vec::new();

    if let Some(actual) = actual_status {
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

    validate_http_headers(
        &mut errors,
        context,
        expected_headers,
        unexpected_headers,
        actual.headers()
    );

    if let Some(expected_body) = expected_body.as_ref() {
        let body = actual.into_http_body();
        validate_http_body(
            &mut errors,
            context,
            expected_body,
            body,
            compare_exact,
            options
        );
    }

    errors

}

pub fn dump_http_resource<T: HttpResource>(
    errors: &mut Vec<String>,
    actual: &mut T,
    context: &str
) {

    let headers = format_http_headers(actual.headers());
    let body = actual.into_http_body();

    errors.push(
        format!(
            "{} {}\n\n        {}\n\n    {} {}",
            context.yellow(),
            "headers dump:".yellow(),
            headers,
            context.yellow(),
            format_http_body(&body)
        )
    )

}

pub fn validate_http_multipart_body(
    expected_body: &[u8],
    expected_mime: &Mime,
    actual_body: &[u8],
    actual_mime: &Mime,
    compare_exact: bool,
    options: &Options

) -> Vec<String> {

    let mut headers = Headers::new();
    headers.set(ContentType(expected_mime.clone()));

    let expected_body = http_body_from_parts(expected_body.to_vec(), &headers);
    headers.set(ContentType(actual_mime.clone()));

    let actual_body = http_body_from_parts(actual_body.to_vec(), &headers);

    let mut errors = Vec::new();
    validate_http_body(
        &mut errors,
        "",
        &expected_body,
        actual_body,
        compare_exact,
        options
    );

    errors

}

