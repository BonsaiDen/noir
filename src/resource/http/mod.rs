// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// External Dependencies ------------------------------------------------------
use hyper::header::Headers;
use hyper::client::Response;


// Modules --------------------------------------------------------------------
mod body;
mod header;
mod query;
mod form;

mod endpoint;
mod request;
mod response;
pub mod util;


// Re-Exports -----------------------------------------------------------------
pub use self::body::HttpBody;
pub use self::header::HttpHeader;
pub use self::query::HttpQueryString;
pub use self::form::{HttpFormData, HttpFormDataField};

pub use self::request::HttpRequest;
pub use self::endpoint::HttpEndpoint;
pub use self::response::HttpResponse;


// Abstraction for HTTP Responses/Requests ------------------------------------
pub trait HttpResource {
    fn headers(&self) -> &Headers;
    fn into_http_body(&mut self) -> HttpBody where Self: Sized;
}

impl HttpResource for Response {

    fn headers(&self) -> &Headers {
        &self.headers
    }

    fn into_http_body(&mut self) -> HttpBody where Self: Sized {
        self.into()
    }

}

