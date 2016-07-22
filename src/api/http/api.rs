// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::thread;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};


// External Dependencies ------------------------------------------------------
use hyper;
use hyper::Client;
use hyper::method::Method;


// Internal Dependencies ------------------------------------------------------
use super::request::HttpRequest;


/// A trait for the description of a testable, HTTP based API.
///
/// # Example Implementation
///
/// ```rust
/// # extern crate noir;
/// use noir::HttpApi;
///
/// #[derive(Copy, Clone, Default)]
/// struct Api;
/// impl HttpApi for Api {
///
///     fn hostname(&self) -> &'static str {
///         "localhost"
///     }
///
///     fn port(&self) -> u16 {
///         8080
///     }
///
///     fn start(&self) {
///         // Start the HTTP server...
///     }
///
/// }
/// # fn main() {}
/// ```
///
/// # Test Failure Examples
///
/// [expanded](terminal://api_start_timeout)
/// [expanded](terminal://api_request_timeout)
pub trait HttpApi: Send + Copy + Default {

    /// A blocking callback for provision of the API.
    ///
    /// The callback is executed in a background thread and should serve the
    /// API at the specified host.
    fn start(&self);

    /// Returns the hostname of the API.
    fn hostname(&self) -> &'static str;

    /// Returns the port of the API.
    fn port(&self) -> u16;

    /// Returns the `hostname:port` combination of the API used for making
    /// requests against the API.
    fn host(&self) -> String {
        format!("{}:{}", self.hostname(), self.port())
    }

    /// Returns the HTTP protocol used by the API.
    ///
    /// By default this is based on the port, returning `https` for port 443.
    fn protocol(&self) -> &'static str {
        match self.port() {
            443 => "https",
            _ => "http"
        }
    }

    /// Returns the fully qualified base URL of the API.
    fn url(&self) -> String {
        match self.port() {
            443 | 80 => format!("{}://{}", self.protocol(), self.hostname()),
            _ => format!("{}://{}", self.protocol(), self.host())
        }
    }

    /// Returns the fully qualified URL of the API with the specified `path`
    /// appended.
    fn url_with_path(&self, path: &str) -> String {
        format!("{}{}", self.url(), path)
    }

    /// Should return the maximum duration to wait for the API to become
    /// available when a test is started.
    ///
    /// Defaults to `1000ms`.
    fn timeout(&self) -> Duration {
        Duration::from_millis(1000)
    }

    /// Returns a `OPTIONS` request that will be performed against the API.
    fn options(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Options, path)
    }

    /// Returns a `GET` request that will be performed against the API.
    fn get(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Get, path)
    }

    /// Returns a `POST` request that will be performed against the API.
    fn post(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Post, path)
    }

    /// Returns a `PUT` request that will be performed against the API.
    fn put(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Put, path)
    }

    /// Returns a `DELETE` request that will be performed against the API.
    fn delete(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Delete, path)
    }

    /// Returns a `HEAD` request that will be performed against the API.
    fn head(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Head, path)
    }

    /// Returns a `TRACE` request that will be performed against the API.
    fn trace(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Trace, path)
    }

    /// Returns a `CONNECT` request that will be performed against the API.
    fn connect(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Connect, path)
    }

    /// Returns a `PATCH` request that will be performed against the API.
    fn patch(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Patch, path)
    }

    /// Returns a request for the specified HTTP verb extension that will be
    /// performed against the API.
    fn ext(http_verb: &'static str, path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Extension(http_verb.to_string()), path)
    }

}


// Statics --------------------------------------------------------------------
fn request<A: HttpApi + 'static>(
    api: A,
    method: Method,
    path: &'static str

) -> HttpRequest<A> {

    let mut api_timed_out = false;
    if let Ok(started) = API_THREADS_STARTED.lock() {

        let host = api.host();
        let mut thread_map = started.borrow_mut();
        if !thread_map.contains_key(&host) {

            // Start server in the background
            thread::spawn(move || {
                api.start();
            });

            let now = Instant::now();
            let timeout = api.timeout();

            // Wait for API server to be available
            while now.elapsed() < timeout {

                let mut client = Client::new();

                // Windows has rather huge timeouts configured here by default
                // so we want to avoid stalling the tests by reducing these
                client.set_read_timeout(Some(Duration::from_millis(50)));
                client.set_write_timeout(Some(Duration::from_millis(50)));

                match client.head(api.url().as_str()).send() {
                    Err(hyper::Error::Io(_)) => { /* waiting for server */ },
                    _ => break
                }

                thread::sleep(Duration::from_millis(10));

            }

            // API server didn't start in time
            if now.elapsed() >= timeout {
                api_timed_out = true;

            // Insert into map
            } else {
                thread_map.insert(host, true);
            }

        }

    }

    super::request::http_request(api, method, path, api_timed_out)

}

lazy_static! {
    static ref API_THREADS_STARTED: Arc<Mutex<RefCell<HashMap<String, bool>>>> = {
        Arc::new(Mutex::new(RefCell::new(HashMap::new())))
    };
}

