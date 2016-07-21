#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();


// Expected Header ------------------------------------------------------------
#[test]
fn test_headers_expected() {

    let actual = {
        API::get("/get/hello")
            .expected_header(Server("Servername".to_string()))
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_headers_expected_multiple() {

    let actual = {
        API::get("/get/hello")
            .expected_headers(headers![
                Server("Servername".to_string())
            ])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_headers_expected_multiple_trailing_comma() {

    let actual = {
        API::get("/get/hello")
            .expected_headers(headers![
                Server("Servername".to_string()),
            ])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_headers_expected_mismatch() {

    let actual = {
        API::get("/get/hello")
            .expected_header(Server("Servername Foo".to_string()))
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>GET <by>request to \"<bn>http://localhost:4000<bn>/get/hello\" <by>returned <br>1 <by>error(s)

<bb> 1) <by>Response <by>header \"<bb>Server\" <by>does not match, expected:

        \"<bg>Servername Foo\"

    <by>but got:

        \"<br>Servername\"

    <by>difference:

        \"Servername <gbg>Foo\"


"#, actual);

}


// With Headers ---------------------------------------------------------------
#[test]
fn test_headers_with_expected() {

    let actual = {
        API::get("/headers/echo")
            .with_header(Accept(vec![
                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
            ]))
            .expected_headers(headers![Accept(vec![
                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
            ])])
            .collect()
    };

    assert_pass!(actual);

}


// Unexpected Header ------------------------------------------------------------
#[test]
fn test_headers_unexpected() {

    let actual = {
        API::get("/echo/method")
            .unexpected_header::<ContentType>()
            .collect()
    };

    assert_pass!(actual);

}


#[test]
fn test_headers_unexpected_mismatch() {

    let actual = {
        API::get("/get/hello")
            .unexpected_header::<Server>()
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>GET <by>request to \"<bn>http://localhost:4000<bn>/get/hello\" <by>returned <br>1 <by>error(s)

<bb> 1) <by>Response <by>header \"<bb>Server\" <by>was expected <bg>to be absent<by>, but <br>is present<by>.


"#, actual);

}

