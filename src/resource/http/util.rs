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
use hyper::status::StatusCode;
use hyper::header::{Headers, HeaderView};


// Internal Dependencies ------------------------------------------------------
use util::json as json_util;
use super::{
    HttpHeader, HttpBody, HttpLike, ParsedHttpBody,
    parse_http_body, parse_json
};


// HTTP Utilities -------------------------------------------------------------
pub fn http_body_into_vec(body: HttpBody) -> Vec<u8> {
    body.data
}

pub fn http_header_into_tuple(header: HttpHeader) -> (String, Vec<u8>) {
    (header.name, header.value)
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

fn validate_http_request_headers<T: HttpLike>(
    errors: &mut Vec<String>,
    result: &mut T,
    context: &str,
    expected_headers: &Headers,
    unexpected_headers: &mut Vec<String>
) {

    // Sort for stable error ordering
    let mut headers = expected_headers.iter().collect::<Vec<HeaderView>>();
    headers.sort_by(|a, b| {
        a.name().cmp(b.name())
    });

    for header in headers {
        if let Some(expected_value) = result.headers().get_raw(header.name()) {
            let actual_value = header.value_string();
            if expected_value[0].as_slice() != actual_value.as_bytes() {
                let expected_value = String::from_utf8(expected_value[0].clone()).unwrap();
                errors.push(format!(
                    "{} {} \"{}\" {}\n\n        \"{}\"\n\n    {}\n\n        \"{}\"",
                    context.yellow(),
                    "header".yellow(),
                    header.name().blue().bold(),
                    "does not match, expected:".yellow(),
                    actual_value.green().bold(),
                    "but got:".yellow(),
                    expected_value.red().bold()
                ));
            }

        } else {
            errors.push(format!(
                "{} {} \"{}\" {} {}{} {}{}",
                context.yellow(),
                "header".yellow(),
                header.name().blue().bold(),
                "was expected".yellow(),
                "to be present".green().bold(),
                ", but".yellow(),
                "is missing".red().bold(),
                ".".yellow()
            ));
        }
    }

    // Sort for stable error ordering
    unexpected_headers.sort();

    for header in unexpected_headers {
        if let Some(_) = result.headers().get_raw(header) {
            errors.push(format!(
                "{} {} \"{}\" {} {}{} {}{}",
                context.yellow(),
                "header".yellow(),
                header.blue().bold(),
                "was expected".yellow(),
                "to be absent".green().bold(),
                ", but".yellow(),
                "is present".red().bold(),
                ".".yellow()
            ));
        }
    }

}

fn validate_http_request_body<T: HttpLike>(
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
    let body = parse_http_body(result.headers(), &body);
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

                        let errors = json_util::compare(
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
                            json_util::format(errors.unwrap_err())
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

fn diff_text(context:&str, expected: &str, actual: &str) -> String {

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

