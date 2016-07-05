macro_rules! test { () => {

// Crates ---------------------------------------------------------------------
#[macro_use] extern crate json;
#[macro_use] extern crate noir;
extern crate colored;
extern crate hyper;


// STD Dependencies -----------------------------------------------------------
use std::io::Read;
use std::net::ToSocketAddrs;


// External Dependencies ------------------------------------------------------
use hyper::method::Method;
use hyper::status::StatusCode;
use hyper::header::{Accept, Connection, ContentType, Server, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::server::{Request, Response};
use hyper::uri::RequestUri;


// Noir Dependencies ----------------------------------------------------------
use noir::{HttpApi, HttpEndpoint};


// Test Server Api ------------------------------------------------------------
fn handle(mut req: Request, mut res: Response) {

    let body = if let RequestUri::AbsolutePath(path) = req.uri.clone() {
        match (req.method.clone(), path.as_str()) {

            (Method::Get, "/get/hello") => {
                res.headers_mut().set(Server("Servername".to_string()));
                res.headers_mut().set(
                    ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
                );
                "Hello World".to_string()
            },

            (m, "/echo/method") => {
                format!("{}", m)
            },

            (Method::Get, "/status/404") => {
                *res.status_mut() = StatusCode::NotFound;
                "".to_string()
            },

            (Method::Get, "/status/500") => {
                *res.status_mut() = StatusCode::InternalServerError;
                "".to_string()
            },

            (Method::Get, "/headers/echo") => {
                *res.headers_mut() = req.headers.clone();
                "".to_string()
            },

            (Method::Post, "/body/echo") => {
                let mut body = String::new();
                req.read_to_string(&mut body).unwrap();
                body
            },

            (Method::Post, "/echo") => {
                *res.headers_mut() = req.headers.clone();
                res.headers_mut().set_raw("Date", vec![b"".to_vec()]);
                let mut body = Vec::new();
                req.read_to_end(&mut body).unwrap();
                res.send(&body[..]).unwrap();
                return;
            },

            (Method::Post, "/response/forward") => {

                let mut body = Vec::new();
                req.read_to_end(&mut body).unwrap();

                let _ext = hyper_client!().post(
                    format!("https://example.com/forward").as_str()

                ).headers(req.headers.clone()).body(&body[..]).send();

                "".to_string()

            },

            (Method::Get, "/response/upgrade") => {

                let _ext = hyper_client!().request(
                    Method::Extension("UPGRADE".to_string()),
                    format!("https://example.com/upgrade").as_str()

                ).send();

                "".to_string()

            },

            (Method::Get, "/responses/none") => {
                "".to_string()
            },

            (Method::Get, "/responses/one") => {
                external_request(&mut res, "/one").to_string()
            },

            (Method::Get, "/responses/query") => {
                external_request(&mut res, "/one?key=value&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&foo=bar&single=item").to_string()
            },

            (Method::Get, "/responses/two") => {
                external_request(&mut res, "/one");
                external_request(&mut res, "/two");
                "".to_string()
            },

            (m, p) => {
                res.headers_mut().set(
                    ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
                );
                format!("Route not found: {} {}", m, p)
            }

        }

    } else {
        "".to_string()
    };

    res.send(body.as_bytes()).unwrap();

}

fn external_request(res: &mut Response, path: &'static str) -> String {

    let req = hyper_client!().get(
        format!("https://example.com{}", path).as_str()

    ).header(Connection::close()).header(Accept(vec![
         qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))

    ])).send();

    match req {
        Ok(mut r) => {
            let mut body = String::new();
            r.read_to_string(&mut body).unwrap();
            *res.status_mut() = r.status;
            *res.headers_mut() = r.headers.clone();
            body
        },
        Err(e) => {
            *res.status_mut() = StatusCode::InternalServerError;
            e.to_string()
        }
    }

}

fn test_server<T: ToSocketAddrs>(addr: T) {
    let _listening = hyper::Server::http(addr).unwrap().handle(handle);
}


// Test Configuration ---------------------------------------------------------
#[derive(Copy, Clone, Default)]
pub struct API;
impl HttpApi for API {

    fn hostname(&self) -> &'static str {
        "localhost"
    }

    fn port(&self) -> u16 {
        4000
    }

    fn start(&self) {
        test_server(self.host().as_str());
    }

}

#[derive(Copy, Clone)]
pub struct EXAMPLE;
impl HttpEndpoint for EXAMPLE {

    fn hostname(&self) -> &'static str {
        "example.com"
    }

    fn port(&self) -> u16 {
        443
    }

}

// Test Output Helper ---------------------------------------------------------
pub fn multiline(output: String) -> String {
    let multiline = output.split("\n").map(|s| {

        let mut raw_string = format!("{:?}", s);
        raw_string = raw_string.replace("\\u{1b}", "");
        raw_string = raw_string.replace("[1;31m", "<br>");
        raw_string = raw_string.replace("[1;32m", "<bg>");
        raw_string = raw_string.replace("[33m", "<by>");
        raw_string = raw_string.replace("[1;34m", "<bb>");
        raw_string = raw_string.replace("[1;35m", "<bp>");
        raw_string = raw_string.replace("[36m", "<bn>");
        raw_string = raw_string.replace("[1;36m", "<bc>");
        raw_string = raw_string.replace("[0m", "");
        raw_string = raw_string.replace("[1;42;37m", "<gbg>");
        raw_string = raw_string.replace("[1;41;37m", "<gbr>");

        (&raw_string[1..raw_string.len() - 1]).trim_right().to_string()

    }).collect::<Vec<String>>().join("\n");
    multiline
}

} }

macro_rules! assert_pass {
    ($result:expr) => {
        if let Err(err) = $result {
            use colored::*;
            println!(
                "\n{} {}{}{}",
                "Test Output:".yellow(),
                ">>>".blue().bold(),
                multiline(err).red().bold(),
                "<<<".blue().bold()
            );
            panic!("Noir test was expected to pass.");
        }
    }
}

macro_rules! assert_fail {
    ($expected:expr, $actual:expr) => {
        match $actual {
            Ok(()) => {
                panic!("Noir test was expected to fail.");
            },
            Err(err) => {
                println!("{}", err);

                use colored::*;

                let result = multiline(err);
                if result != $expected {
                    println!(
                        "\n{} {}{}{}",
                        "Expected Output:".yellow(),
                        ">>>".blue().bold(),
                        $expected.green().bold(),
                        "<<<".blue().bold()
                    );

                    println!(
                        "\n{} {}{}{}",
                        "Actual Output:".yellow(),
                        ">>>".blue().bold(),
                        result.red().bold(),
                        "<<<".blue().bold()
                    );

                    panic!("Noir test output does not match.");
                }
            }
        }
    }
}

