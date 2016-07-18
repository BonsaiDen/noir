// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::cmp;
use std::time::Duration;
use std::error::Error as ErrorTrait;
use std::net::{SocketAddr, Shutdown};
use std::io::{Error, ErrorKind, Write, Read};


// External Dependencies ------------------------------------------------------
use hyper;
use httparse;
use hyper::net::{NetworkConnector, NetworkStream};


// Internal Dependencies ------------------------------------------------------
use mock::MockResponseProvider;
use resource::http::HttpRequest;


/// A macro for intercepting `hyper::Client::new()` calls made during tests.
///
/// During testing (`#[cfg(test)]`) the macro will be replaced with
/// `hyper::Client::with_connector(...)`
///
/// Outside of testing the macro will be compiled out and reduce itself to a
/// normal `hyper::Client::new()` call.
///
/// # Example Usage
///
/// ```rust
/// # extern crate hyper;
/// # #[macro_use] extern crate noir;
///
/// # fn main() {
/// let client = hyper_client!();
/// let request = client.get(
///     "https://example.com/"
/// );
/// # }
/// ```
#[macro_export]
macro_rules! hyper_client {
    () => {
        if cfg!(test) {
            $crate::mock::http::mocked_hyper_client()

        } else {
            hyper::Client::new()
        }
    }
}


#[cfg_attr(feature = "clippy", allow(inline_always))]
#[inline(always)]
#[doc(hidden)]
pub fn mocked_hyper_client() -> hyper::Client {
    hyper::Client::with_connector(MockConnector)
}


// Noir Internal --------------------------------------------------------------
struct MockConnector;
impl NetworkConnector for MockConnector {

    type Stream = MockStream;

    fn connect(
        &self,
        host: &str,
        port: u16,
        _: &str

    ) -> Result<Self::Stream, hyper::Error> {
        Ok(MockStream::new(host, port))
    }

}

struct MockStream {
    host: String,
    port: u16,
    request: Vec<u8>,
    response: Result<Vec<u8>, Error>,
    response_index: usize
}

impl MockStream {
    pub fn new(host: &str, port: u16) -> MockStream {
        MockStream {
            host: host.to_string(),
            port: port,
            request: Vec::new(),
            response: Ok(Vec::new()),
            response_index: 0
        }
    }
}

impl Write for MockStream {

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.request.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Error> {

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);

        match req.parse(&self.request[..]) {
            Ok(httparse::Status::Complete(size)) => {

                let request = Box::new(HttpRequest::new(
                    self.host.to_string(),
                    self.port,
                    req,
                    self.request[size..].to_vec()
                ));

                match MockResponseProvider::response_from_request(request) {
                    Ok(response) => {
                        self.response = response;
                        Ok(())
                    },
                    Err(err) => Err(err)
                }

            },
            _ => unreachable!()
        }

    }

}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        match self.response.as_ref() {
            Ok(response) => {

                let bytes_available = response.len() - self.response_index;
                let bytes_to_read = buf.len();
                let bytes_read = cmp::min(bytes_to_read, bytes_available);

                let bytes = &response[
                    self.response_index..self.response_index + bytes_read
                ];

                for (index, b) in bytes.iter().enumerate() {
                    buf[index] = *b;
                }

                self.response_index += bytes_read;

                Ok(bytes_read)

            },
            Err(err) => Err(Error::new(err.kind(), err.description()))
        }
    }
}

impl NetworkStream for MockStream {

    fn peer_addr(&mut self) -> Result<SocketAddr, Error> {
        Err(Error::new(ErrorKind::NotConnected, "noir: Address not mocked."))
    }

    fn set_read_timeout(&self, _: Option<Duration>) -> Result<(), Error> {
        Ok(())
    }

    fn set_write_timeout(&self, _: Option<Duration>) -> Result<(), Error> {
        Ok(())
    }

    fn close(&mut self, _: Shutdown) -> Result<(), Error> {
        Ok(())
    }

}

