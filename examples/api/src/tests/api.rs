// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// External Dependencies ------------------------------------------------------
extern crate noir;
use noir::{HttpApi, HttpEndpoint};


// Internal Dependencies ------------------------------------------------------
use server;


// Api Description used for testing the Server --------------------------------
#[derive(Copy, Clone, Default)]
pub struct Api;
impl HttpApi for Api {

    fn hostname(&self) -> &'static str {
        "localhost"
    }

    fn port(&self) -> u16 {
        4000
    }

    fn start(&self) {
        server::run(self.host().as_str());
    }

}


// A mocked endpoint for icanhazip.com ----------------------------------------
#[derive(Copy, Clone)]
pub struct IcanHazIp;
impl HttpEndpoint for IcanHazIp {

    fn hostname(&self) -> &'static str {
        "icanhazip.com"
    }

    fn port(&self) -> u16 {
        443
    }

}

