// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::any::Any;


// External Dependencies ------------------------------------------------------
use colored::*;
use httparse::Request;
use hyper::method::Method;
use hyper::header::Headers;


// Internal Dependencies ------------------------------------------------------
use mock::MockRequest;
use super::{HttpLike, HttpBody};
use super::body::http_body_from_parts;


// Noir Internal --------------------------------------------------------------
pub struct HttpRequest {
    pub hostname: String,
    pub port: u16,
    pub method: Method,
    pub path: String,
    headers: Headers,
    body: Option<HttpBody>
}

impl HttpRequest  {

    pub fn new(
        hostname: String,
        port: u16,
        req: Request,
        data: Vec<u8>

    ) -> HttpRequest {

        let headers = Headers::from_raw(req.headers).unwrap();
        let body = http_body_from_parts(data, &headers);

        HttpRequest {
            hostname: hostname,
            port: port,
            method: req.method.unwrap().parse().unwrap(),
            path: req.path.unwrap().to_string(),
            headers: headers,
            body: Some(body)
        }

    }

    fn host(&self) -> String {
        format!("{}:{}", self.hostname, self.port)
    }

    fn protocol(&self) -> &'static str {
        match self.port {
            443 => "https",
            _ => "http"
        }
    }

    fn url(&self) -> String {
        match self.port {
            443 | 80 => format!("{}://{}", self.protocol(), self.hostname),
            _ => format!("{}://{}", self.protocol(), self.host())
        }
    }

}

impl MockRequest for HttpRequest {

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn validate(&mut self) -> Option<String> {
        Some(format!(
            "{} {} {} {} \"{}{}\"{}",
            "Request Failure:".red().bold(),
            "Unexpected".yellow(),
            format!("{}", self.method).cyan(),
            "request to".yellow(),
            self.url().cyan(),
            self.path.cyan(),
            ", no response was provided.".yellow()
        ))
    }

}

impl HttpLike for HttpRequest {

    fn headers(&self) -> &Headers {
        &self.headers
    }

    fn into_http_body(&mut self) -> HttpBody where Self: Sized {
        self.body.take().unwrap()
    }

}

