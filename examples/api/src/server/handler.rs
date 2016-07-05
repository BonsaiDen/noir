// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::io::Read;


// External Dependencies ------------------------------------------------------
use hyper;
use hyper::header::Connection;
use nickel;
use nickel::status::StatusCode;


// Route Handlers -------------------------------------------------------------
pub fn get_ip(response: &mut nickel::Response) -> String {

    // Allow noir to intercept this call when compiled in test
    let request = hyper_client!().get(
        "https://icanhazip.com/"

    ).header(Connection::close()).send();

    match request {
        Ok(mut res) => {
            let mut body = String::new();
            res.read_to_string(&mut body).unwrap();
            body
        },
        Err(e) => {
            *response.status_mut() = StatusCode::InternalServerError;
            e.to_string()
        }
    }

}

