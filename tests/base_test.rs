macro_rules! test { () => {

// Crates ---------------------------------------------------------------------
extern crate colored;
extern crate hyper;


// STD Dependencies -----------------------------------------------------------
use std::thread;
use std::io::Read;
use std::time::Duration;
use std::net::ToSocketAddrs;


// External Dependencies ------------------------------------------------------
use hyper::method::Method;
use hyper::status::StatusCode;
use hyper::header::{Accept, Connection, ContentType, Server, qitem};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
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

            (Method::Get, "/timeout") => {
                thread::sleep(Duration::from_millis(2000));
                "".to_string()
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

                // Handle form-data requests
                if let Some(&mut ContentType(Mime(_, _, ref mut attrs))) = req.headers.get_mut::<ContentType>() {

                    // Replace any form boundary value with a static string so we
                    // can test it
                    if let Some(&mut (Attr::Boundary, Value::Ext(ref mut b))) = attrs.get_mut(0) {

                        // Ignore plain boundaries from some of the tests
                        if b != "boundary" {
                            // TODO IW: Replace boundary without string conversion
                            let mut str_body = String::from_utf8(body).unwrap();
                            str_body = str_body.replace(b.as_str(), "boundary12345");

                            *b = "boundary12345".to_string();

                            body = str_body.into();
                        }

                    }

                }

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

            (Method::Post, "/form") => {

                let mut body = String::new();
                req.read_to_string(&mut body).unwrap();

                let boundary = if let Some(&ContentType(ref mime)) = req.headers.get::<ContentType>() {
                    if let &Mime(TopLevel::Application, SubLevel::FormData, ref attrs) = mime {
                        Some(attrs[0].1.as_str().to_string())

                    } else {
                        None
                    }

                } else {
                    None
                };

                if let Some(boundary) = boundary {
                    body = body.replace(boundary.as_str(), "<boundary>");
                }

                res.headers_mut().set(
                    ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
                );
                body

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

            // TODO IW: Support binary responses
            let mut body = String::new();
            r.read_to_string(&mut body).unwrap();
            *res.status_mut() = r.status;

            // Handle form-data responses
            if let Some(&mut ContentType(Mime(_, _, ref mut attrs))) = r.headers.get_mut::<ContentType>() {

                // Replace any form boundary value with a static string so we
                // can test it
                if let Some(&mut (Attr::Boundary, Value::Ext(ref mut b))) = attrs.get_mut(0) {
                    body = body.replace(b.as_str(), "boundary12345");
                    *b = "boundary12345".to_string();
                }

            }

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

