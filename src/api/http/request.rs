// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::sync::{Arc, Mutex};


// External Dependencies ------------------------------------------------------
use colored::*;
use hyper::{Client, Error};
use hyper::method::Method;
use hyper::client::Response;
use hyper::status::StatusCode;
use hyper::header::{Header, Headers, HeaderFormat, ContentType};


// Internal Dependencies ------------------------------------------------------
use HttpApi;
use Options;
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
    options: Options,

    api_timed_out: bool,
    dump_response: bool,

    provided_responses: Vec<Box<MockResponse + 'static>>,
    provided_mocks: Vec<Box<MockProvider + 'static>>,

    request_headers: Headers,
    request_body: Option<HttpBody>,

    expected_status: Option<StatusCode>,
    expected_headers: Headers,
    expected_body: Option<HttpBody>,
    compare_exact: bool,

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
        self.path = util::path_with_query(self.path.as_str(), query);
        self
    }

    /// Sets the request body.
    ///
    /// Also sets the `Content-Type` header of the request based on the type
    /// of the `body` argument.
    ///
    /// The set `Content-Type` can be overridden via `HttpRequest::with_header`.
    pub fn with_body<S: Into<HttpBody>>(mut self, body: S) -> Self {
        self.request_body = Some(body.into());
        self
    }

    /// Sets the request's configuration options.
    ///
    /// This allows to change or override the default request behaviour.
    pub fn with_options(mut self, options: Options) -> Self {
        self.options = options;
        self
    }

    /// Sets the expected response status for the request.
    ///
    /// ### Test Failure
    ///
    /// If the actual response status does not match the expected one.
    ///
    /// ### Test Failure Examples
    ///
    /// [expanded](terminal://headers_expected)
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
    ///
    /// ### Test Failure Examples
    ///
    /// [expanded](terminal://headers_unexpected)
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
    /// of the response.
    ///
    /// ##### text/*
    ///
    /// These Compared as strings, if no other character encoding is set in the
    /// response's MIME type, UTF-8 will be used as the default.
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
    ///
    /// ### Test Failure Examples
    ///
    /// [expanded](terminal://body_expected_raw_mismatch)
    /// [collapsed](terminal://body_text_mismatch_diff_added)
    /// [collapsed](terminal://body_with_expected_json_mismatch)
    pub fn expected_body<S: Into<HttpBody>>(mut self, body: S) -> Self {
        self.expected_body = Some(body.into());
        self.compare_exact = false;
        self
    }

    /// Sets the expected response body for the request (exact version).
    ///
    /// This method is based on `HttpRequest::expected_body()` but performs
    /// additional comparison based on the mime type of the response:
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
        self.compare_exact = true;
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
    ///
    /// ### Test Failure Examples
    ///
    /// [expanded](terminal://dump_response_with_raw_body)
    /// [collapsed](terminal://dump_response_with_text_body)
    /// [collapsed](terminal://dump_response_with_json_body)
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
                "\n{} {} \"{}\" {} {}{}\n\n",
                "API Failure:".red().bold(),
                "Server for".yellow(),
                self.api.url().as_str().cyan(),
                "did not respond within".yellow(),
                "1000ms".green().bold(),
                ".".yellow()
            ));
        }

        // Limit ourselves to one test at a time in order to ensure correct
        // handling of mocked requests and responses
        let (errors, error_count, suppressed_count) = if let Ok(_) = REQUEST_LOCK.lock() {
            self.send()

        } else {
            (vec![format!(
                "{} {}",
                "Internal Error:".red().bold(),
                "Request lock failed.".yellow()

            )], 1, 0)
        };

        if !errors.is_empty() {

            let mut report = String::new();

            // Title
            report.push_str(format!(
                "\n{} {} {} \"{}{}\" {} {} {}\n",
                "Response Failure:".red().bold(),
                format!("{}", self.method).cyan(),
                "request to".yellow(),
                self.api.url().cyan(),
                self.path.cyan(),
                "returned".yellow(),
                format!("{}", error_count).red().bold(),
                "error(s)".yellow()

            ).as_str());

            // Response and Request Errors
            for (index, e) in errors.iter().enumerate() {
                report.push_str(format!(
                    "\n{} {}\n", format!("{:2})", index + 1).blue().bold(),
                    e

                ).as_str());
            }

            // Suppressed error information
            if suppressed_count != 0 {
                report.push_str(format!(
                    "\n{} {} {} {}\n",
                    "Note:".green().bold(),
                    "Suppressed".black().bold(),
                    format!("{}", suppressed_count).blue().bold(),
                    "request error(s) that may have resulted from failed response expectations.".black().bold()

                ).as_str());
            }

            // Padding
            report.push_str("\n\n");

            Err(report)

        } else {
            Ok(())
        }

    }

    fn send(&mut self) -> (Vec<String>, usize, usize) {

        // Provide responses to request interceptors
        ResponseProvider::provide(
            self.provided_responses.drain(0..).collect()
        );

        for mock in &mut self.provided_mocks {
            mock.setup();
        }

        // Set up hyper client
        let mut client = Client::new();
        client.set_read_timeout(Some(self.options.api_request_timeout));
        client.set_write_timeout(Some(self.options.api_request_timeout));

        // Send request and validate response
        let errors = match self.http_request(&mut client) {
            Ok(response) => self.validate_response(response),
            Err(_) => (vec![format!(
                "{} {} {}{}",
                "API Failure:".red().bold(),
                "No response within".yellow(),
                "1000ms".green().bold(),
                ".".yellow()

            )], 1, 0)
        };

        for mock in &mut self.provided_mocks {
            mock.teardown();
        }

        errors

    }

    fn http_request(&mut self, client: &mut Client) -> Result<Response, Error> {

        let (content_mime, body) = if let Some(body) = self.request_body.take() {
            util::http_body_into_parts(body)

        } else {
            (None, None)
        };

        if let Some(content_mime) = content_mime {
            // Set Content-Type based on body data if:
            // A. The body has a Mime
            // B. No other Content-Type has been set on the request
            if !self.request_headers.has::<ContentType>() {
                self.request_headers.set(ContentType(content_mime));
            }
        }

        let request = client.request(
            self.method.clone(),
            self.api.url_with_path(self.path.as_str()).as_str()

        ).headers(
            self.request_headers.clone()
        );

        if let Some(body) = body.as_ref() {
            request.body(&body[..]).send()

        } else {
            request.send()
        }

    }

    fn validate_response(&mut self, mut response: Response) -> (Vec<String>, usize, usize) {

        // Request dumping
        let mut errors = Vec::new();
        if self.dump_response {
            util::dump_http_resource(
                &mut errors,
                &mut response,
                "Response"
            );
        }

        // Validate Response
        let status = response.status;
        errors.append(&mut util::validate_http_resource(
            "Response",
            self.expected_status,
            &self.expected_headers,
            &mut self.unexpected_headers,
            &self.expected_body,
            &mut response,
            Some(status),
            self.compare_exact,
            &self.options
        ));

        // Validate Resource Requests
        let (mut response_errors, total_error_count) = self.validate_requests(errors.len());

        // Suppress cascading response errors that may be the result from
        // previous resource request errors
        if !response_errors.is_empty() &&
            self.options.error_suppress_cascading {

            // Remove the suppressed errors
            let suppressed_count = errors.len();
            errors.clear();

            errors.append(&mut response_errors);
            (errors, total_error_count - suppressed_count, suppressed_count)

        } else {
            errors.append(&mut response_errors);
            (errors, total_error_count, 0)
        }

    }

    fn validate_requests(&self, response_error_count: usize) -> (Vec<String>, usize) {

        // Correct error index offset if response errors are suppressed
        let index_offset = if self.options.error_suppress_cascading {
            0

        } else {
            response_error_count
        };

        let mut total_error_count = response_error_count;
        let mut response_errors = Vec::new();

        // Validate requests to all provided and unprovided responses
        for (
            mut response,
            response_index,
            request_index

        ) in ResponseProvider::provided_responses() {

            let errors = response.validate(response_index, request_index);
            if !errors.is_empty() {
                let header = response.validate_header(errors.len());
                total_error_count += errors.len();
                response_errors.push(format_response_errors(
                    header,
                    index_offset + response_index + 1,
                    errors
                ));
            }

        }

        for mut request in ResponseProvider::additional_requests() {
            if let Some(error) = request.validate() {
                response_errors.push(error);
                total_error_count += 1;
            }
        }

        // Reset the global response provider for the next test
        ResponseProvider::reset();

        (response_errors, total_error_count)

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
fn format_response_errors(
    header: String, offset: usize, errors: Vec<String>

) -> String {

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

                        ).blue().bold(),
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
        options: Default::default(),

        api_timed_out: api_timed_out,
        dump_response: false,

        provided_responses: Vec::new(),
        provided_mocks: Vec::new(),

        request_headers: Headers::new(),
        request_body: None,

        expected_status: None,
        expected_headers: Headers::new(),
        expected_body: None,
        compare_exact: false,

        unexpected_headers: Vec::new(),

        run_on_drop: true
    }
}

lazy_static! {
    static ref REQUEST_LOCK: Arc<Mutex<()>> = {
        Arc::new(Mutex::new(()))
    };
}

