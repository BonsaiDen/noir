// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// External Dependencies ------------------------------------------------------
use hyper::method::Method;


// Internal Dependencies ------------------------------------------------------
use super::response::{HttpResponse, http_response};


/// A trait for the description of a HTTP based endpoint used to provided
/// mocked responses to a testable API.
///
/// # Example Implementation
///
/// ```rust
/// # extern crate noir;
/// use noir::HttpEndpoint;
///
/// #[derive(Copy, Clone)]
/// struct ExternalResource;
/// impl HttpEndpoint for ExternalResource {
///
///     fn hostname(&self) -> &'static str {
///         "rust-lang.org"
///     }
///
///     fn port(&self) -> u16 {
///         443
///     }
///
/// }
/// # fn main() {}
/// ```
pub trait HttpEndpoint: Send + Copy {

    /// Returns the hostname of the endpoint.
    fn hostname(&self) -> &'static str;

    /// Returns the port of the api.
    fn port(&self) -> u16;

    /// Returns the `hostname:port` combination of the endpoint.
    fn host(&self) -> String {
        format!("{}:{}", self.hostname(), self.port())
    }

    /// Returns the http protocol used by the endpoint.
    ///
    /// By default this is based on the port, returning `https` for port 443.
    fn protocol(&self) -> &'static str {
        match self.port() {
            443 => "https",
            _ => "http"
        }
    }

    /// Returns the fully qualified base url of the endpoint.
    fn url(&self) -> String {
        match self.port() {
            443 | 80 => format!("{}://{}", self.protocol(), self.hostname()),
            _ => format!("{}://{}", self.protocol(), self.host())
        }
    }

    /// Returns the fully qualified url of the endpoint with the specified `path`
    /// appended.
    fn url_with_path(&self, path: &str) -> String {
        format!("{}{}", self.url(), path)
    }

    /// Return a response to the next `OPTIONS` request made against the endpoint
    /// which matches the specified path.
    fn options(&self, path: &'static str) -> HttpResponse<Self> {
        http_response(*self, Method::Options, path)
    }

    /// Return a response to the next `GET` request made against the endpoint
    /// which matches the specified path.
    fn get(&self, path: &'static str) -> HttpResponse<Self> {
        http_response(*self, Method::Get, path)
    }

    /// Return a response to the next `POST` request made against the endpoint
    /// which matches the specified path.
    fn post(&self, path: &'static str) -> HttpResponse<Self> {
        http_response(*self, Method::Post, path)
    }

    /// Return a response to the next `PUT` request made against the endpoint
    /// which matches the specified path.
    fn put(&self, path: &'static str) -> HttpResponse<Self> {
        http_response(*self, Method::Put, path)
    }

    /// Return a response to the next `DELETE` request made against the endpoint
    /// which matches the specified path.
    fn delete(&self, path: &'static str) -> HttpResponse<Self> {
        http_response(*self, Method::Delete, path)
    }

    /// Return a response to the next `HEAD` request made against the endpoint
    /// which matches the specified path.
    fn head(&self, path: &'static str) -> HttpResponse<Self> {
        http_response(*self, Method::Head, path)
    }

    /// Return a response to the next `TRACE` request made against the endpoint
    /// which matches the specified path.
    fn trace(&self, path: &'static str) -> HttpResponse<Self> {
        http_response(*self, Method::Trace, path)
    }

    /// Return a response to the next `CONNECT` request made against the endpoint
    /// which matches the specified path.
    fn connect(&self, path: &'static str) -> HttpResponse<Self> {
        http_response(*self, Method::Connect, path)
    }

    /// Return a response to the next `PATCH` request made against the endpoint
    /// which matches the specified path.
    fn patch(&self, path: &'static str) -> HttpResponse<Self> {
        http_response(*self, Method::Patch, path)
    }

    /// Return a response to the next request made against the endpoint
    /// which matches the specified path and http verb extension.
    fn ext(&self, http_verb: &'static str, path: &'static str) -> HttpResponse<Self> {
        http_response(*self, Method::Extension(http_verb.to_string()), path)
    }

}

