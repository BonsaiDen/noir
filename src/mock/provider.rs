// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::io::{Error, ErrorKind};


// Internal Dependencies ------------------------------------------------------
use mock::{MockRequest, MockResponse};


// Global Mocked Response Provider --------------------------------------------
pub struct ResponseProvider {
    response_index: usize,
    provided_responses: Vec<(Box<MockResponse + 'static>, usize, usize)>,
    request_index: usize,
    additional_requests: Vec<Box<MockRequest>>
}

impl ResponseProvider {

    pub fn reset() {
        let handler = PROVIDER_INSTANCE.clone();
        match handler.lock() {
            Ok(handler) => {
                let mut provider = handler.borrow_mut();
                provider.response_index = 0;
                provider.request_index = 0;
            },
            _ => unreachable!()
        };
    }

    pub fn provide(resources: Vec<Box<MockResponse + 'static>>) {
        let handler = PROVIDER_INSTANCE.clone();
        match handler.lock() {
            Ok(handler) => {
                let mut provider = handler.borrow_mut();
                for resource in resources {
                    let index = provider.response_index;
                    provider.provided_responses.push((resource, index, 0));
                    provider.response_index += 1;
                }
            },
            _ => unreachable!()
        };
    }

    #[cfg_attr(feature = "clippy", allow(needless_return))]
    pub fn provided_responses() -> Vec<(Box<MockResponse + 'static>, usize, usize)> {
        let handler = PROVIDER_INSTANCE.clone();
        return match handler.lock() {
            Ok(handler) => {
                let mut provider = handler.borrow_mut();
                return provider.provided_responses.drain(0..).collect();
            },
            _ => Vec::new()
        };
    }

    #[cfg_attr(feature = "clippy", allow(needless_return))]
    pub fn additional_requests() -> Vec<Box<MockRequest + 'static>> {
        let handler = PROVIDER_INSTANCE.clone();
        return match handler.lock() {
            Ok(handler) => {
                let mut provider = handler.borrow_mut();
                return provider.additional_requests.drain(0..).collect();
            },
            _ => Vec::new()
        };
    }

    #[cfg_attr(feature = "clippy", allow(needless_return))]
    pub fn request(
        request: Box<MockRequest>

    ) -> Result<Result<Vec<u8>, Error>, Error> {

        let handler = PROVIDER_INSTANCE.clone();
        return match handler.lock() {
            Ok(handler) => {

                let mut provider = handler.borrow_mut();

                // Increase internal request counter for order validation
                let index = provider.request_index;
                provider.request_index += 1;

                // Check all responses for a potential match
                for &mut(
                    ref mut response,
                    _,
                    ref mut request_index

                ) in &mut provider.provided_responses {
                    if response.matches(&request) {
                        *request_index = index;
                        return Ok(response.respond(request));
                    }
                }

                // Track requests which are missing their response
                provider.additional_requests.push(request);

                Err(Error::new(
                    ErrorKind::ConnectionRefused,
                    "noir: No response provided in test."
                ))

            },
            _ => Err(Error::new(
                ErrorKind::ConnectionReset,
                "noir: Handler lock failed."
            ))
        };

    }

}


// Statics --------------------------------------------------------------------
lazy_static! {
    static ref PROVIDER_INSTANCE: Arc<Mutex<RefCell<ResponseProvider>>> = {
        Arc::new(Mutex::new(RefCell::new(ResponseProvider {
            response_index: 0,
            provided_responses: Vec::new(),
            request_index: 0,
            additional_requests: Vec::new()
        })))
    };
}

