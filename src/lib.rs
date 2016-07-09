// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! **noir** is a request driven, black box testing library for HTTP based APIs.
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![deny(
    missing_docs,
    trivial_casts, trivial_numeric_casts,
    unsafe_code,
    unused_import_braces, unused_qualifications
)]


// Crates ---------------------------------------------------------------------
extern crate url;
#[macro_use]
extern crate json;
extern crate rand;
extern crate hyper;
extern crate colored;
extern crate httparse;
#[macro_use]
extern crate lazy_static;
extern crate difference;


// Modules --------------------------------------------------------------------
mod api;
#[doc(hidden)]
pub mod mock;
mod resource;
mod util;


// Exports --------------------------------------------------------------------
pub use api::http::{HttpApi, HttpRequest};
pub use mock::{
    MockResponse, MockRequest, MockProvider,
    MockResponseProvider, MockRequestResponse
};
pub use resource::http::{
    HttpEndpoint,
    HttpResponse,
    HttpHeader,
    HttpBody,
    HttpQueryString,
    HttpFormData
};


/// A convenience macro for creating a vector of `Box<MockResponse>` items.
#[macro_export]
macro_rules! responses {
    ( $( $x:expr ),* ) => (
        {
            let mut temp_vec = Vec::<Box<$crate::MockResponse>>::new();
            $(
                temp_vec.push(Box::new($x));
            )*
            temp_vec
        }
    );
    // TODO fix macro name and test
    ( $( $x:expr, )* ) => ( res![ $($x ),* ] )
}

/// A convenience macro for creating a vector of `Box<MockProvider>` items.
#[macro_export]
macro_rules! mocks {
    ( $( $x:expr ),* ) => (
        {
            let mut temp_vec = Vec::<Box<$crate::MockProvider>>::new();
            $(
                temp_vec.push(Box::new($x));
            )*
            temp_vec
        }
    );
    // TODO fix macro name and test
    ( $( $x:expr, )* ) => ( res![ $($x ),* ] )
}

/// A convenience macro for creating a vector of `HttpHeader` items.
#[macro_export]
macro_rules! headers {
    ( $( $x:expr ),* ) => (
        {
            let mut temp_vec = Vec::<$crate::HttpHeader>::new();
            $(
                temp_vec.push($x.into());
            )*
            temp_vec
        }
    );
    // TODO fix macro name and test
    ( $( $x:expr, )* ) => ( res![ $($x ),* ] )
}

/// A macro for creating a `HttpQueryString` instance.
#[macro_export]
macro_rules! query {
    // TODO IW: Support trailing comma
    {} => ($crate::HttpQueryString::new(vec![]));

    { $( $key:expr => $value:expr ),* } => ({

        let mut items = Vec::new();

        $(
            items.push(($key, $value).into());
        )*

        $crate::HttpQueryString::new(items)
    })
}

/// A macro for creating a `HttpFormData ` instance.
#[macro_export]
macro_rules! form {
    // TODO IW: Support trailing comma
    {} => ($crate::HttpFormData::new(vec![]));

    { $( $key:expr => $value:expr ),* } => ({

        let mut fields = Vec::new();

        $(
            fields.push(($key, $value).into());
        )*

        $crate::HttpFormData::new(fields)
    })
}

