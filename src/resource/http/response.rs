// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::io::Error;


// External Dependencies ------------------------------------------------------
use colored::*;
use hyper::method::Method;
use hyper::server::Response as ServerResponse;
use hyper::status::StatusCode;
use hyper::header::{Header, Headers, HeaderFormat, ContentType};


// Internal Dependencies ------------------------------------------------------
use Options;
use super::request::HttpRequest;
use super::endpoint::HttpEndpoint;
use mock::{MockRequest, MockResponse};
use resource::http::util;
use resource::http::{HttpHeader, HttpBody, HttpQueryString};


/// A mocked HTTP response that is being provided to a testable API.
pub struct HttpResponse<E: HttpEndpoint> {
    endpoint: E,

    method: Method,
    path: String,
    options: Options,

    dump_request: bool,

    response_status: Option<StatusCode>,
    response_headers: Headers,
    response_body: Option<HttpBody>,
    response_error: Option<Error>,

    expected_headers: Headers,
    unexpected_headers: Vec<String>,
    expected_body: Option<HttpBody>,
    expected_exact_body: bool,

    request: Option<Box<MockRequest>>
}

impl<E: HttpEndpoint> HttpResponse<E> {

    /// Sets the response status.
    pub fn with_status(mut self, status_code: StatusCode) -> Self {
        self.response_status = Some(status_code);
        self
    }

    /// Sets additional headers to be send with the response.
    ///
    /// Use the `headers![...]` macro to easily create a vector containing
    /// concrete types of the `hyper::Header` trait for use with this method.
    pub fn with_headers(mut self, headers: Vec<HttpHeader>) -> Self {
        for header in headers {
            let (name, value) = util::http_header_into_tuple(header);
            self.response_headers.set_raw(name, vec![value]);
        }
        self
    }

    /// Sets one additional header to be send with the response.
    pub fn with_header<H: Header + HeaderFormat>(mut self, header: H) -> Self {
        self.response_headers.set(header);
        self
    }

    /// Sets the query string of the response's path.
    ///
    /// This will override any existing query string previously set or derived
    /// from the request's path.
    pub fn with_query(mut self, query: HttpQueryString) -> Self {
        self.path = util::path_with_query(self.path.as_str(), query);
        self
    }

    /// Sets the response body.
    ///
    /// Also sets the `Content-Type` header of the response based on the type
    /// of the `body` argument.
    ///
    /// The set `Content-Type` can be overridden via `HttpResponse::with_header`.
    pub fn with_body<S: Into<HttpBody>>(mut self, body: S) -> Self {
        self.response_body = Some(body.into());
        self
    }

    /// Sets a low level IO error to be returned once the response is read.
    pub fn with_error(mut self, error: Error) -> Self {
        self.response_error = Some(error);
        self
    }

    /// Sets the response's configuration options.
    ///
    /// This allows to change or override the default response behaviour.
    pub fn with_options(mut self, options: Options) -> Self {
        self.options = options;
        self
    }

    /// Sets one additional header that should be present on the request to the
    /// response.
    ///
    /// ### Test Failure
    ///
    /// If the header is either missing from the request or its value does
    /// not match the expected one.
    pub fn expected_header<H: Header + HeaderFormat>(mut self, header: H) -> Self {
        self.expected_headers.set(header);
        self
    }

    /// Sets one additional header that should be absent from the request made
    /// to the response.
    ///
    /// ### Test Failure
    ///
    /// If the header is present on the request.
    pub fn unexpected_header<H: Header + HeaderFormat>(mut self) -> Self {
        self.unexpected_headers.push(<H>::header_name().to_string());
        self
    }

    /// Sets additional headers that should be present on the request to the
    /// response.
    ///
    /// Use the `headers![...]` macro to easily create a vector containing
    /// concrete types of the `hyper::Header` trait for use with this method.
    ///
    /// ### Test Failure
    ///
    /// If one or more of the headers are either missing from the request
    /// or their values differ from the expected ones.
    pub fn expected_headers(mut self, headers: Vec<HttpHeader>) -> Self {
        for header in headers {
            let (name, value) = util::http_header_into_tuple(header);
            self.expected_headers.set_raw(name, vec![value]);
        }
        self
    }

    /// Sets the expected request body for the response.
    ///
    /// The expected and the actual body are compared based on the MIME type
    /// of the request.
    ///
    /// ##### text/*
    ///
    /// These Compared as strings, if no other character encoding is set in the
    /// request's MIME type, UTF-8 will be used as the default.
    ///
    /// ##### application/json
    ///
    /// JSON objects are deep compared, but __additional keys on request objects
    /// are ignored__.
    ///
    /// This allows for simpler and more fine grained assertions against JSON
    /// requests.
    ///
    /// ##### All other mime types
    ///
    /// These are compared on a byte by byte basis.
    ///
    /// ### Test Failure
    ///
    /// If the actual request body does not match the expected one.
    pub fn expected_body<S: Into<HttpBody>>(mut self, body: S) -> Self {
        self.expected_body = Some(body.into());
        self.expected_exact_body = false;
        self
    }

    /// Sets the expected request body for the response, exact version.
    ///
    /// This method is based on `HttpRequest::expected_body()` but performs
    /// additional comparison based on the mime type of the request:
    ///
    /// ##### application/json
    ///
    /// In contrast to `HttpResponse::expected_body()` __additional keys on
    /// request objects are compared and will fail the test__.
    ///
    /// ##### All other mime types
    ///
    /// See `HttpResponse::expected_body()`.
    ///
    /// ### Test Failure
    ///
    /// If the actual request body does not match the expected one.
    pub fn expected_exact_body<S: Into<HttpBody>>(mut self, body: S) -> Self {
        self.expected_body = Some(body.into());
        self.expected_exact_body = true;
        self
    }

    /// Dumps the request headers and body for this response.
    ///
    /// ### Test Failure
    ///
    /// Always.
    pub fn dump(mut self) -> Self {
        self.dump_request = true;
        self
    }

}

impl<E: HttpEndpoint> MockResponse for HttpResponse<E> {

    fn matches(&self, request: &Box<MockRequest>) -> bool {
        if let Some(request) = HttpRequest::downcast_ref(request) {
            self.request.is_none()
                && self.method == request.method
                && self.endpoint.hostname() == request.hostname
                && self.endpoint.port() == request.port
                && self.path == request.path

        } else {
            false
        }
    }

    fn respond(
        &mut self,
        request: Box<MockRequest>

    ) -> Result<Vec<u8>, Error> {

        self.request = Some(request);

        if let Some(err) = self.response_error.take() {
            Err(err)

        } else {
            let mut data: Vec<u8> = Vec::new();
            {

                // Convert body into Mime and Vec<u8>
                let (content_mime, mut body) = if let Some(body) = self.response_body.take() {
                    util::http_body_into_parts(body)

                } else {
                    (None, None)
                };

                // Set Content-Type based on body data if:
                // A. The body has a Mime
                // B. No other Content-Type has been set on the request
                if let Some(content_mime) = content_mime {
                    if !self.response_headers.has::<ContentType>() {
                        self.response_headers.set(ContentType(content_mime));
                    }
                }

                // Create Response
                let mut res = ServerResponse::new(
                    &mut data, &mut self.response_headers
                );

                // Set status code if specified
                if let Some(status) = self.response_status {
                    *res.status_mut() = status;
                }

                // Send body if specified
                if let Some(body) = body.take() {
                    res.send(&body[..]).ok();

                // Empty body
                } else {
                    res.start().unwrap().end().ok();
                }
            }

            Ok(data)

        }

    }

    fn validate(
        &mut self,
        response_index: usize,
        request_index: usize

    ) -> Vec<String> {

        if let Some(request) = self.request.as_mut() {

            let mut errors = Vec::new();

            let request = HttpRequest::downcast_mut(request).unwrap();
            if self.dump_request {
                util::dump_http_like(
                    &mut errors, request, "Request"
                );
            }

            if response_index != request_index {
                errors.push(format!(
                    "{} {} {}{} {} {}{}",
                    "Response fetched out of order,".yellow(),
                    "provided for request".green().bold(),
                    format!("{}", response_index + 1).blue().bold(),
                    ",".yellow(),
                    "fetched by request".red().bold(),
                    format!("{}", request_index + 1).blue().bold(),
                    ".".yellow()
                ));
            }

            errors.append(&mut util::validate_http_request(
                request,
                &self.options,
                "Request",
                None,
                None,
                &self.expected_headers,
                &mut self.unexpected_headers,
                &self.expected_body,
                self.expected_exact_body
            ));

            errors

        } else {
            vec![format!(
                "{} {} {} {}{}",
                "Expected".yellow(),
                "a request".green().bold(),
                "for the response, but got".yellow(),
                "none".red().bold(),
                ".".yellow()
            )]
        }

    }


    fn validate_header(
        &self,
        error_count: usize

    ) -> String {
        format!(
            "{} {} \"{}{}\" {} {} {}",
            format!("{}", self.method).cyan().bold(),
            "response provided for".yellow(),
            self.endpoint.url().cyan().bold(),
            self.path.cyan().bold(),
            "returned".yellow(),
            format!("{}", error_count).red().bold(),
            "error(s)".yellow()
        )
    }

}

// Internal -------------------------------------------------------------------
pub fn http_response<E: HttpEndpoint>(
    endpoint: E,
    method: Method,
    path: &'static str

) -> HttpResponse<E> {
    HttpResponse {
        endpoint: endpoint,

        method: method,
        path: path.to_string(),
        options: Default::default(),

        dump_request: false,

        response_status: None,
        response_headers: Headers::new(),
        response_body: None,
        response_error: None,

        unexpected_headers: Vec::new(),
        expected_headers: Headers::new(),
        expected_body: None,
        expected_exact_body: false,

        request: None
    }
}

