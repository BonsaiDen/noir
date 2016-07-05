// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::time::Duration;
use std::sync::{Arc, Mutex};


// External Dependencies ------------------------------------------------------
use url::Url;
use colored::*;
use hyper::Client;
use hyper::method::Method;
use hyper::client::Response;
use hyper::status::StatusCode;
use hyper::header::{Header, Headers, HeaderFormat};


// Internal Dependencies ------------------------------------------------------
use HttpApi;
use mock::{MockResponse, MockProvider, ResponseProvider};
use resource::http::util;
use resource::http::{HttpHeader, HttpBody, HttpQueryString};

/// A HTTP request for API testing.
///
/// The request is automatically started once the instance goes out of scope
/// and any set up expectations are asserted and logged to `stdout` in case of
/// failure.
///
/// `HttpRequest::collect()` can be used to manually start the request.
///
/// # Example Usage
///
/// ```rust
/// # #[macro_use] extern crate noir;
/// # extern crate hyper;
/// # use hyper::status::StatusCode;
/// # use hyper::header::{Accept, Connection, qitem};
/// # use hyper::mime::{Mime, TopLevel, SubLevel};
/// # use noir::{HttpApi, HttpEndpoint, HttpRequest};
/// # #[derive(Copy, Clone, Default)]
/// # struct Api;
/// # impl HttpApi for Api {
/// #     fn hostname(&self) -> &'static str {
/// #         "localhost"
/// #     }
/// #     fn port(&self) -> u16 {
/// #         8080
/// #     }
/// #     fn start(&self) {
/// #         // Start the HTTP server...
/// #     }
/// # }
/// # #[derive(Copy, Clone)]
/// # struct ExternalResource;
/// # impl HttpEndpoint for ExternalResource {
/// #     fn hostname(&self) -> &'static str {
/// #         "rust-lang.org"
/// #     }
/// #     fn port(&self) -> u16 {
/// #         443
/// #     }
/// # }
/// # fn test_request() -> HttpRequest<Api> {
/// Api::get("/")
///     .with_headers(headers![
///         Connection::close(),
///         Accept(vec![
///             qitem(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
///         ])
///     ])
///     .provide(responses![
///         ExternalResource.get("/")
///                         .with_status(StatusCode::Ok)
///                         .with_body("Hello World")
///                         .expected_header(Connection::close())
///     ])
///     .expected_status(StatusCode::Ok)
///     .expected_body("Hello World")
/// # }
/// # fn main() {
/// # test_request().collect().unwrap_err();
/// # }
/// ```
///
/// # Panics
///
/// If any of the set up expectations fail.
pub struct HttpRequest<A: HttpApi> {
    api: A,
    method: Method,
    path: String,

    api_timed_out: bool,
    dump_response: bool,

    provided_responses: Vec<Box<MockResponse + 'static>>,
    provided_mocks: Vec<Box<MockProvider + 'static>>,

    request_headers: Headers,
    request_body: Option<HttpBody>,

    expected_status: Option<StatusCode>,
    expected_headers: Headers,
    expected_body: Option<HttpBody>,
    expected_exact_body: bool,

    unexpected_headers: Vec<String>,

    run_on_drop: bool
}

impl<A: HttpApi> HttpRequest<A> {

    /// Sets additional headers to be send with the request.
    ///
    /// Use the `headers![...]` macro to easily create a vector containing
    /// concrete types of the `hyper::Header` trait for use with this method.
    pub fn with_headers(mut self, headers: Vec<HttpHeader>) -> Self {
        for header in headers {
            let (name, value) = util::http_header_into_tuple(header);
            self.request_headers.set_raw(name, vec![value]);
        }
        self
    }

    /// Sets one additional header to be send with the request.
    pub fn with_header<H: Header + HeaderFormat>(mut self, header: H) -> Self {
        self.request_headers.set(header);
        self
    }

    /// Sets the request's query string.
    ///
    /// This will override any existing query string previously set or derived
    /// from the request's path.
    pub fn with_query(mut self, query: HttpQueryString) -> Self {

        // Parse existing URL
        let url = self.api.url_with_path(self.path.as_str());
        let mut uri = Url::parse(url.as_str()).unwrap();

        // Set new query string
        let query: Option<String> = query.into();
        match query {
            Some(query) => uri.set_query(Some(query.as_str())),
            None => uri.set_query(None)
        }

        // Adjust path with new query string
        self.path = if let Some(query) = uri.query() {
            format!("{}?{}", uri.path(), query)

        } else {
            uri.path().to_string()
        };

        self

    }

    /// Sets the request body.
    pub fn with_body<S: Into<HttpBody>>(mut self, body: S) -> Self {
        self.request_body = Some(body.into());
        self
    }

    /// Sets the expected response status for the request.
    ///
    /// ### Test Failure
    ///
    /// If the actual response status does not match the expected one.
    pub fn expected_status(mut self, status_code: StatusCode) -> Self {
        self.expected_status = Some(status_code);
        self
    }

    /// Sets one additional header that should be present on the response.
    ///
    /// ### Test Failure
    ///
    /// If the header is either missing from the response or its value does
    /// not match the expected one.
    pub fn expected_header<H: Header + HeaderFormat>(mut self, header: H) -> Self {
        self.expected_headers.set(header);
        self
    }

    /// Sets one additional header that should be absent from the response.
    ///
    /// ### Test Failure
    ///
    /// If the header is present on the response.
    pub fn unexpected_header<H: Header + HeaderFormat>(mut self) -> Self {
        self.unexpected_headers.push(<H>::header_name().to_string());
        self
    }

    /// Sets additional headers that should be present on the response.
    ///
    /// Use the `headers![...]` macro to easily create a vector containing
    /// concrete types of the `hyper::Header` trait for use with this method.
    ///
    /// ### Test Failure
    ///
    /// If one or more of the headers are either missing from the response
    /// or their values differ from the expected ones.
    pub fn expected_headers(mut self, headers: Vec<HttpHeader>) -> Self {
        for header in headers {
            let (name, value) = util::http_header_into_tuple(header);
            self.expected_headers.set_raw(name, vec![value]);
        }
        self
    }

    /// Sets the expected response body for the request.
    ///
    /// The expected and the actual body are compared based on the MIME type
    /// of the reponse.
    ///
    /// ##### text/*
    ///
    /// These Compared as strings, if no other charset is set in the response
    /// MIME type, UTF-8 will be used as the default encoding.
    ///
    /// ##### application/json
    ///
    /// JSON objects are deep compared, but __additional keys on response objects
    /// are ignored__.
    ///
    /// This allows for simpler and more fine grained assertions against JSON
    /// responses.
    ///
    /// ##### All other mime types
    ///
    /// These are compared on a byte by byte basis.
    ///
    /// ### Test Failure
    ///
    /// If the actual response body does not match the expected one.
    pub fn expected_body<S: Into<HttpBody>>(mut self, body: S) -> Self {
        self.expected_body = Some(body.into());
        self.expected_exact_body = false;
        self
    }

    /// Sets the expected response body for the request (exact version).
    ///
    /// This method is based on `HttpRequest::expected_body()` but performs
    /// additional comparison based on the mime type of the reponse:
    ///
    /// ##### application/json
    ///
    /// In contrast to `HttpResponse::expected_body()` __additional keys on
    /// response objects are compared and will fail the test__.
    ///
    /// ##### All other mime types
    ///
    /// See `HttpResponse::expected_body()`.
    ///
    /// ### Test Failure
    ///
    /// If the actual response body does not match the expected one.
    pub fn expected_exact_body<S: Into<HttpBody>>(mut self, body: S) -> Self {
        self.expected_body = Some(body.into());
        self.expected_exact_body = true;
        self
    }

    /// Provides additional mocked responses from endpoints for the time of the
    /// currently executing request.
    ///
    /// Use the `responses![...]` macro to easily create a vector containing
    /// concrete types of the `MockResponse` trait for use with this method.
    ///
    /// ### Test Failure
    ///
    /// If one or more of the provided responses is not called or does not
    /// match its expectations.
    pub fn provide(mut self, mut resources: Vec<Box<MockResponse>>) -> Self {
        self.provided_responses.append(&mut resources);
        self
    }

    /// Dumps the response headers and body this request.
    ///
    /// ### Test Failure
    ///
    /// Always.
    pub fn dump(mut self) -> Self {
        self.dump_response = true;
        self
    }

    /// Provides additional mocks which will be active for the time of the
    /// currently executing request.
    ///
    /// Use the `mocks![...]` macro to easily create a vector containing
    /// concrete types of the `MockProvider` trait for use with this method.
    pub fn mocks<T: MockProvider + 'static>(mut self, mocks: Vec<T>) -> Self {
        for res in mocks {
            self.provided_mocks.push(Box::new(res));
        }
        self
    }

    /// Directly execute the test request and collect any error message output.
    pub fn collect(mut self) -> Result<(), String> {
        self.run_on_drop = false;
        self.run()
    }


    // Internal ---------------------------------------------------------------
    fn run(&mut self) -> Result<(), String> {

        // Handle cases where the API test server did not start in time
        if self.api_timed_out {
            return Err(format!(
                "\n{}\n\n    {} \"{}\" {} {}{}\n\n",
                "Noir Api Failure:".red().bold(),
                "Specified server to test at".yellow(),
                self.api.url().as_str().blue().bold(),
                "did not start within".yellow(),
                "1000ms".cyan(),
                ".".yellow()
            ));
        }

        // Limit ourselves to one test at a time in order to ensure correct
        // handling of mocked requests and responses
        let (errors, error_count) = if let Ok(_) = REQUEST_LOCK.lock() {
            self.request()

        } else {
            (vec![format!(
                "{} {}",
                "Internal Noir Error:".red().bold(),
                "Request lock failed.".yellow()

            )], 1)
        };

        if !errors.is_empty() {

            let mut report = String::new();

            // Title
            report.push_str(format!(
                "\n{} {} {} \"{}{}\" {} {} {}\n",
                "Response Failure:".red().bold(),
                format!("{}", self.method).cyan().bold(),
                "request to".yellow(),
                self.api.url().cyan().bold(),
                self.path.cyan().bold(),
                "returned".yellow(),
                format!("{}", error_count).red().bold(),
                "error(s)".yellow()

            ).as_str());

            // Response and Request Errors
            for (index, e) in errors.iter().enumerate() {
                report.push_str(format!(
                    "\n{} {}\n", format!("{:2})", index + 1).red().bold(),
                    e

                ).as_str());
            }

            // Padding
            report.push_str("\n\n");

            Err(report)

        } else {
            Ok(())
        }

    }

    fn request(&mut self) -> (Vec<String>, usize) {

        // Convert body into Vec<u8>
        let body = if let Some(body) = self.request_body.take() {
            util::http_body_into_vec(body)

        } else {
            Vec::new()
        };

        // Setup request
        let mut client = Client::new();
        client.set_read_timeout(Some(Duration::from_millis(1000)));
        client.set_write_timeout(Some(Duration::from_millis(1000)));

        let request = client.request(
            self.method.clone(),
            self.api.url_with_path(self.path.as_str()).as_str()

        ).headers(
            self.request_headers.clone()

        ).body(
            &body[..]
        );

        // Provide responses to request interceptors
        ResponseProvider::provide(
            self.provided_responses.drain(0..).collect()
        );

        for mock in &mut self.provided_mocks {
            mock.setup();
        }

        // Send response and validate
        let errors = match request.send() {
            Ok(response) => {
                self.validate(response)
            },
            Err(err) => {
                // TODO IW: Explictly handle timeouts?
                (vec![format!(
                    "{} {}",
                    "Internal Noir Error:".red().bold(),
                    err.to_string().yellow()

                )], 1)
            }
        };

        for mock in &mut self.provided_mocks {
            mock.teardown();
        }

        errors

    }

    fn validate(&mut self, mut response: Response) -> (Vec<String>, usize) {

        let mut errors = Vec::new();
        if self.dump_response {
            util::dump_http_like(
                &mut errors,
                &mut response,
                "Response"
            );
        }

        let status = response.status;
        errors.append(&mut util::validate_http_request(
            &mut response,
            "Response",
            Some(status),
            self.expected_status,
            &self.expected_headers,
            &mut self.unexpected_headers,
            &self.expected_body,
            self.expected_exact_body
        ));

        let mut error_count = errors.len();
        let index_offset = error_count;

        for (
            mut response,
            response_index,
            request_index

        ) in ResponseProvider::provided_responses() {

            let response_errors = response.validate(response_index, request_index);
            if !response_errors.is_empty() {
                let header = response.validate_header(response_errors.len());
                error_count += response_errors.len();
                errors.push(format_response_errors(
                    header,
                    index_offset + response_index + 1,
                    response_errors
                ));
            }

        }

        for mut request in ResponseProvider::additional_requests() {
            if let Some(error) = request.validate() {
                errors.push(error);
                error_count += 1;
            }
        }

        ResponseProvider::reset();

        (errors, error_count)

    }

}

/// Automatically starts the request and logs any errors from set up expectations
/// to `stdout`.
///
/// # Panics
///
/// If any of the set up expectations fail.
impl<A: HttpApi> Drop for HttpRequest<A> {
    fn drop(&mut self) {
        if self.run_on_drop {
            if let Err(report) = self.run() {
                print!("{}", report);
                panic!("Request failed, see above for details.");
            }
        }
    }
}


// Helper ---------------------------------------------------------------------
fn format_response_errors(header: String, offset: usize, errors: Vec<String>) -> String {

    let mut formatted = format!(
        "{} {}",
        "Request Failure:".red().bold(),
        header
    );

    for (index, e) in errors.iter().enumerate() {
        formatted.push_str("\n\n");
        formatted.push_str(
            e.lines()
             .enumerate()
             .map(|(i, line)| {
                if i == 0 {
                    format!(
                        "    {} {}",
                        format!(
                            "{:2}.{})",
                            offset,
                            index + 1

                        ).red().bold(),
                        line
                    )

                } else {
                    format!("      {}", line)
                }

            }).collect::<Vec<String>>().join("\n").as_str()
        );
    }

    formatted

}


// Internal -------------------------------------------------------------------
pub fn http_request<A: HttpApi>(
    api: A,
    method: Method,
    path: &'static str,
    api_timed_out: bool

) -> HttpRequest<A> {
    HttpRequest {
        api: api,
        method: method,
        path: path.to_string(),

        api_timed_out: api_timed_out,
        dump_response: false,

        provided_responses: Vec::new(),
        provided_mocks: Vec::new(),

        request_headers: Headers::new(),
        request_body: None,

        expected_status: None,
        expected_headers: Headers::new(),
        expected_body: None,
        expected_exact_body: false,

        unexpected_headers: Vec::new(),

        run_on_drop: true
    }
}

lazy_static! {
    static ref REQUEST_LOCK: Arc<Mutex<()>> = {
        Arc::new(Mutex::new(()))
    };
}

