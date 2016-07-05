// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::net::ToSocketAddrs;


// External Dependencies ------------------------------------------------------
use nickel;
use nickel::HttpRouter;


// Internal Dependencies ------------------------------------------------------
mod handler;


// Server startup -------------------------------------------------------------
pub fn run<T: ToSocketAddrs>(addr: T) {

    let mut server = nickel::Nickel::new();

    server.get("/api/ip", middleware! {|_, mut response|
        format!("Your current IP address is: {}", handler::get_ip(&mut response))
    });

    server.listen(addr);

}

