#[macro_use]
mod base_test;
test!();

use std::io::{Error, ErrorKind};


#[test]
fn test_responses_provided() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
            ])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_responses_provided_missing() {

    let actual = {
        API::get("/responses/none")
            .provide(responses![
                EXAMPLE.get("/one")
            ])
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/responses/none\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/one\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Expected <bg>a request <by>for the response, but got <br>none<by>.


"#, actual);

}

#[test]
fn test_responses_provided_ext_method() {

    let actual = {
        API::get("/response/upgrade")
            .provide(responses![
                EXAMPLE.ext("UPGRADE", "/upgrade")
            ])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_responses_provided_with_error() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_error(Error::new(ErrorKind::ConnectionRefused, "Mocked Error"))
            ])
            .expected_body("Mocked Error")
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_responses_provided_multiple_errors() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                    .expected_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
                    .expected_header(Server("Foo".to_string()))
                    .expected_body("Hello World")
            ])
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/responses/one\" <by>returned <br>3 <by>error(s)

<br> 1) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/one\" <by>returned <br>3 <by>error(s)

    <br> 1.1) <by>Request <by>header \"<bb>Content-Type\" <by>was expected <bg>to be present<by>, but <br>is missing<by>.

    <br> 1.2) <by>Request <by>header \"<bb>Server\" <by>was expected <bg>to be present<by>, but <br>is missing<by>.

    <br> 1.3) <by>Request <by>raw body data does not match, expected the following <bg>11 bytes<by>:

             [<bg>0x48, <bg>0x65, <bg>0x6C, <bg>0x6C, <bg>0x6F, <bg>0x20, <bg>0x57, <bg>0x6F, <bg>0x72, <bg>0x6C, <bg>0x64]

          <by>but got the following <br>0 bytes <by>instead:

             []


"#, actual);

}

