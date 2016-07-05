// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};


// External Dependencies ------------------------------------------------------
use hyper;
use hyper::Client;
use hyper::method::Method;


// Internal Dependencies ------------------------------------------------------
use super::request::HttpRequest;


/// A trait for the description of a testable, HTTP based api.
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
pub trait HttpApi: Send + Copy + Default {

    /// A blocking callback for provision of the api.
    ///
    /// The callback is executed in a background thread and should serve the
    /// api at the specified host.
    fn start(&self);

    /// Returns the hostname of the api.
    fn hostname(&self) -> &'static str;

    /// Returns the port of the api.
    fn port(&self) -> u16;

    /// Returns the `hostname:port` combination of the api used for making
    /// requests against the api.
    fn host(&self) -> String {
        format!("{}:{}", self.hostname(), self.port())
    }

    /// Returns the http protocol used by the api.
    ///
    /// By default this is based on the port, returning `https` for port 443.
    fn protocol(&self) -> &'static str {
        match self.port() {
            443 => "https",
            _ => "http"
        }
    }

    /// Returns the fully qualified base url of the api.
    fn url(&self) -> String {
        match self.port() {
            443 | 80 => format!("{}://{}", self.protocol(), self.hostname()),
            _ => format!("{}://{}", self.protocol(), self.host())
        }
    }

    /// Returns the fully qualified url of the api with the specified `path`
    /// appended.
    fn url_with_path(&self, path: &str) -> String {
        format!("{}{}", self.url(), path)
    }

    /// Returns a `OPTIONS` request that will be performed against the api.
    fn options(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Options, path)
    }

    /// Returns a `GET` request that will be performed against the api.
    fn get(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Get, path)
    }

    /// Returns a `POST` request that will be performed against the api.
    fn post(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Post, path)
    }

    /// Returns a `PUT` request that will be performed against the api.
    fn put(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Put, path)
    }

    /// Returns a `DELETE` request that will be performed against the api.
    fn delete(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Delete, path)
    }

    /// Returns a `HEAD` request that will be performed against the api.
    fn head(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Head, path)
    }

    /// Returns a `TRACE` request that will be performed against the api.
    fn trace(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Trace, path)
    }

    /// Returns a `CONNECT` request that will be performed against the api.
    fn connect(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Connect, path)
    }

    /// Returns a `PATCH` request that will be performed against the api.
    fn patch(path: &'static str) -> HttpRequest<Self> where Self: 'static {
        request(Self::default(), Method::Patch, path)
    }

    /// Returns a request for the specified http verb extension that will be
    /// performed against the api.
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

    // TODO IW: Support multiple APIs?
    let mut api_timed_out = false;
    if let Ok(started) = API_THREAD_STARTED.lock() {

        // Start server in background if required
        if !started.load(Ordering::Relaxed) {

            started.store(true, Ordering::Relaxed);

            thread::spawn(move || {
                api.start();
            });

            // Wait for API server to be started
            let mut ticks = 0;
            while ticks < 100 {
                let client = Client::new();
                match client.head(api.url().as_str()).send() {
                    Err(hyper::Error::Io(_)) => { /* waiting for server */ },
                    _ => break
                }
                thread::sleep(Duration::from_millis(10));
                ticks += 1;
            }

            // API server didn't start in time
            if ticks == 100 {
                api_timed_out = true;
            }

        }

    }

    super::request::http_request(api, method, path, api_timed_out)

}

lazy_static! {
    static ref API_THREAD_STARTED: Arc<Mutex<AtomicBool>> = {
        Arc::new(Mutex::new(AtomicBool::new(false)))
    };
}

