#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();


// Response Status ------------------------------------------------------------
#[test]
fn test_get_with_expected_status() {

    let actual = {
        API::get("/status/500")
            .expected_status(StatusCode::InternalServerError).collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_get_with_expected_status_mismatch() {

    let actual = {
        API::get("/status/404")
            .expected_status(StatusCode::Ok).collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/status/404\" <by>returned <br>1 <by>error(s)

<bb> 1) <by>Response <by>status code does not match value, expected:

        \"<bg>200 OK\"

    <by>but got:

        \"<br>404 Not Found\"


"#, actual);

}

