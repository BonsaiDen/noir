#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();


// Cascacing Errors -----------------------------------------------------------
#[test]
fn test_cascading_missing_response_suppress() {

    let actual = {
        API::get("/responses/one")
            .expected_status(StatusCode::Ok)
            .expected_header(Server("Foo".to_string()))
            .expected_body("Hello World")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/responses/one\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <by>Unexpected <bc>GET <by>request to \"<bc>https://example.com<bc>/one\"<by>, no response was provided.

<bg>Note: <bbb>Suppressed <bb>3 <bbb>request error(s) that may have resulted from failed response expectations.


"#, actual);

}

#[test]
fn test_cascading_missing_response_show() {

    use noir::Options;
    let actual = {
        API::get("/responses/one")
            .with_options(Options {
                error_suppress_cascading: false,
                .. Default::default()
            })
            .expected_status(StatusCode::Ok)
            .expected_header(Server("Foo".to_string()))
            .expected_body("Hello World")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/responses/one\" <by>returned <br>4 <by>error(s)

<br> 1) <by>Response <by>status code does not match value, expected:

        \"<bg>200 OK\"

    <by>but got:

        \"<br>500 Internal Server Error\"

<br> 2) <by>Response <by>header \"<bb>Server\" <by>was expected <bg>to be present<by>, but <br>is missing<by>.

<br> 3) <by>Response <by>raw body data does not match, expected the following <bg>11 bytes<by>:

       [<bg>0x48, <bg>0x65, <bg>0x6C, <bg>0x6C, <bg>0x6F, <bg>0x20, <bg>0x57, <bg>0x6F, <bg>0x72, <bg>0x6C, <bg>0x64]

    <by>but got the following <br>35 bytes <by>instead:

       [<br>0x6E, <br>0x6F, <br>0x69, <br>0x72, <br>0x3A, <br>0x20, <br>0x4E, <br>0x6F, <br>0x20, <br>0x72, <br>0x65, <br>0x73, <br>0x70, <br>0x6F, <br>0x6E, <br>0x73,
        <br>0x65, <br>0x20, <br>0x70, <br>0x72, <br>0x6F, <br>0x76, <br>0x69, <br>0x64, <br>0x65, <br>0x64, <br>0x20, <br>0x69, <br>0x6E, <br>0x20, <br>0x74, <br>0x65,
        <br>0x73, <br>0x74, <br>0x2E]

<br> 4) <br>Request Failure: <by>Unexpected <bc>GET <by>request to \"<bc>https://example.com<bc>/one\"<by>, no response was provided.


"#, actual);

}

#[test]
fn test_cascading_mismatch_response_suppress() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_body("Foo").expected_body("Foo")
            ])
            .expected_status(StatusCode::Ok)
            .expected_header(Server("Foo".to_string()))
            .expected_body("Hello World")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/responses/one\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/one\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>raw body data does not match, expected the following <bg>3 bytes<by>:

             [<bg>0x46, <bg>0x6F, <bg>0x6F]

          <by>but got the following <br>0 bytes <by>instead:

             []

<bg>Note: <bbb>Suppressed <bb>2 <bbb>request error(s) that may have resulted from failed response expectations.


"#, actual);

}

#[test]
fn test_cascading_mismatch_response_show() {

    use noir::Options;
    let actual = {
        API::get("/responses/one")
            .with_options(Options {
                error_suppress_cascading: false,
                .. Default::default()
            })
            .provide(responses![
                EXAMPLE.get("/one").expected_body("Foo")
            ])
            .expected_status(StatusCode::Ok)
            .expected_header(Server("Foo".to_string()))
            .expected_body("Hello World")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/responses/one\" <by>returned <br>3 <by>error(s)

<br> 1) <by>Response <by>header \"<bb>Server\" <by>was expected <bg>to be present<by>, but <br>is missing<by>.

<br> 2) <by>Response <by>raw body data does not match, expected the following <bg>11 bytes<by>:

       [<bg>0x48, <bg>0x65, <bg>0x6C, <bg>0x6C, <bg>0x6F, <bg>0x20, <bg>0x57, <bg>0x6F, <bg>0x72, <bg>0x6C, <bg>0x64]

    <by>but got the following <br>0 bytes <by>instead:

       []

<br> 3) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/one\" <by>returned <br>1 <by>error(s)

    <br> 3.1) <by>Request <by>raw body data does not match, expected the following <bg>3 bytes<by>:

             [<bg>0x46, <bg>0x6F, <bg>0x6F]

          <by>but got the following <br>0 bytes <by>instead:

             []


"#, actual);

}

#[test]
fn test_cascading_out_of_order_responses_suppress() {

    let actual = {
        API::get("/responses/two")
            .provide(responses![
                EXAMPLE.get("/two"),
                EXAMPLE.get("/one")
            ])
            .expected_status(StatusCode::Ok)
            .expected_header(Server("Foo".to_string()))
            .expected_body("Hello World")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/responses/two\" <by>returned <br>2 <by>error(s)

<br> 1) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/two\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Response fetched out of order, <bg>provided for request <bb>1<by>, <br>fetched by request <bb>2<by>.

<br> 2) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/one\" <by>returned <br>1 <by>error(s)

    <br> 2.1) <by>Response fetched out of order, <bg>provided for request <bb>2<by>, <br>fetched by request <bb>1<by>.

<bg>Note: <bbb>Suppressed <bb>2 <bbb>request error(s) that may have resulted from failed response expectations.


"#, actual);

}

#[test]
fn test_cascading_out_of_order_responses_show() {

    use noir::Options;
    let actual = {
        API::get("/responses/two")
            .with_options(Options {
                error_suppress_cascading: false,
                .. Default::default()
            })
            .provide(responses![
                EXAMPLE.get("/two"),
                EXAMPLE.get("/one")
            ])
            .expected_status(StatusCode::Ok)
            .expected_header(Server("Foo".to_string()))
            .expected_body("Hello World")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/responses/two\" <by>returned <br>4 <by>error(s)

<br> 1) <by>Response <by>header \"<bb>Server\" <by>was expected <bg>to be present<by>, but <br>is missing<by>.

<br> 2) <by>Response <by>raw body data does not match, expected the following <bg>11 bytes<by>:

       [<bg>0x48, <bg>0x65, <bg>0x6C, <bg>0x6C, <bg>0x6F, <bg>0x20, <bg>0x57, <bg>0x6F, <bg>0x72, <bg>0x6C, <bg>0x64]

    <by>but got the following <br>0 bytes <by>instead:

       []

<br> 3) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/two\" <by>returned <br>1 <by>error(s)

    <br> 3.1) <by>Response fetched out of order, <bg>provided for request <bb>1<by>, <br>fetched by request <bb>2<by>.

<br> 4) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/one\" <by>returned <br>1 <by>error(s)

    <br> 4.1) <by>Response fetched out of order, <bg>provided for request <bb>2<by>, <br>fetched by request <bb>1<by>.


"#, actual);

}

