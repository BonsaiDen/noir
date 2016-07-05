// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::any::Any;
use std::io::Error;


// Internal Dependencies ------------------------------------------------------
#[doc(hidden)]
pub mod http;
mod provider;


// Exports --------------------------------------------------------------------
pub use self::provider::ResponseProvider;


/// A trait for implementation of a response provided to a concrete type of
/// `MockRequest`.
pub trait MockResponse: Send {

    /// Compare the response against any type implementing `MockRequest` and
    /// return `true` if they belong together.
    ///
    /// Any implementation should try to downcast the boxed value to its concrete
    /// type and - if the downcast succeeds - compare the custom properties of
    /// the request and response.
    ///
    /// Requests will try to match all the responses in the order these were
    /// provided to the test and will call `MockResponse::respond()` for the
    /// first match.
    ///
    /// Responses which should only match a single request need to always return
    /// `false` once `MockResponse::respond()` was called.
    fn matches(&self, &Box<MockRequest>) -> bool;

    /// Called when the implementation of `MockResponse::matches` return `true`.
    ///
    /// Should return a `Result` with either the response body or a `std::io::Error`.
    fn respond(&mut self, Box<MockRequest>) -> MockRequestResponse;

    /// If the response has a matching `MockRequest`, compare the two and return
    /// a vector of error messages listing any differences.
    ///
    /// ### Test Failure
    ///
    /// If the return vector contains any error messages.
    fn validate(&mut self, response_index: usize, request_index: usize) -> Vec<String>;

    /// Return a header for use with the formatted error values returned by
    /// `MockResponse::validate()`.
    fn validate_header(&self, error_count: usize) -> String;

}

/// A trait for implementation of a request matched against concrete types of
/// `MockResponse`.
pub trait MockRequest: Send + Any {

    /// If the request has no matching `MockResponse` return a error message.
    ///
    /// ### Test Failure
    ///
    /// If `Some(String)` is returned.
    fn validate(&mut self) -> Option<String>;

    /// A helper for downcasting a `MockRequest` trait object into one of its
    /// concrete types.
    #[cfg_attr(feature = "clippy", allow(needless_lifetimes))]
    fn downcast_ref<'a>(
        request: &'a Box<MockRequest>

    ) -> Option<&'a Self> where Self: Any + Send + Sized + 'static {
        request.as_any().downcast_ref::<Self>()
    }

    /// Mutable version of `MockRequest::downcast_ref`.
    #[cfg_attr(feature = "clippy", allow(needless_lifetimes))]
    fn downcast_mut<'a>(
        request: &'a mut Box<MockRequest>

    ) -> Option<&'a mut Self> where Self: Any + Send + Sized + 'static {
        request.as_any_mut().downcast_mut::<Self>()
    }

    /// Method for casting the concrete implementation of `MockRequest` into a
    /// `&Any` for use with `MockRequest::downcast_ref()`.
    ///
    /// This must be implemented on the concrete type in order for the `&Any`
    /// to have the correct type id for downcasting.
    ///
    /// # Concrete Implementation
    ///
    /// ```rust
    /// # use std::any::Any;
    /// # struct Foo;
    /// # impl Foo {
    /// fn as_any(&self) -> &Any {
    ///     self
    /// }
    /// # }
    /// ```
    fn as_any(&self) -> &Any;

    /// Method for casting the concrete implementation of `MockRequest` into a
    /// `&mut Any` for use with `MockRequest::downcast_mut()`.
    ///
    /// This must be implemented on the concrete type in order for the `&mut Any`
    /// to have the correct type id for downcasting.
    ///
    /// # Concrete Implementation
    ///
    /// ```rust
    /// # use std::any::Any;
    /// # struct Foo;
    /// # impl Foo {
    /// fn as_any_mut(&mut self) -> &mut Any {
    ///     self
    /// }
    /// # }
    /// ```
    fn as_any_mut(&mut self) -> &mut Any;

}

/// A trait for implementation of a custom mock provider.
pub trait MockProvider: Send {

    /// Called before each individual `HttpRequest` in a test is send.
    fn setup(&mut self);

    /// Called after each individual `HttpRequest` in a test has completed.
    fn teardown(&mut self);

}

/// A response to a request made against a mocked endpoint.
///
/// Contains the raw response body data.
///
/// # Errors
///
/// If the response was configured to return a custom `std::io::Error`.
pub type MockRequestResponse = Result<Vec<u8>, Error>;

/// An interface for `MockRequest` trait implementations for consuming
/// matching `MockResponse` objects that were provided during the active test.
pub struct MockResponseProvider;
impl MockResponseProvider {

    /// Returns the first, not yet consumed `MockResponse` that matches the
    /// provided `request`.
    ///
    /// Matching is performed via `MockResponse::matches_request()`.
    ///
    /// # Errors
    ///
    /// When no matching response for the `request` exists.
    pub fn response_from_request(
        request: Box<MockRequest>

    ) -> Result<MockRequestResponse, Error> where Self: Sized {
        ResponseProvider::request(request)
    }

}

