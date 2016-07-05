// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate noir;

use noir::{HttpApi, HttpEndpoint};
use hyper::status::StatusCode;
use hyper::header::Connection;
use super::api::{Api, IcanHazIp};

#[test]
fn test_ip_mocked() {
    Api::get("/api/ip")
        .with_headers(headers![
            Connection::close()
        ])
        .with_body("Some post data")
        .provide(responses![
            IcanHazIp.get("/")
                .with_body("Mocked")
                .expected_headers(headers![
                    Connection::close()
                ])
        ])
        .expected_status(StatusCode::Ok)
        .expected_body("Your current IP address is: Mocked");
}

