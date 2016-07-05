#[macro_use]
mod base_test;
test!();


#[test]
fn test_responses_provided_with_query_string() {

    let actual = {
        API::get("/responses/query")
            .provide(responses![
                EXAMPLE.get("/one")
                       .with_query(query!{
                           "key" => "value",
                           "array[]" => vec!["item1", "item2", "item3"],
                           "foo" => "bar",
                           "single" => vec!["item"]
                       })
            ])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_responses_provided_with_query_string_mismatch() {

    let actual = {
        API::get("/responses/query")
            .provide(responses![
                EXAMPLE.get("/two")
                       .with_query(query!{
                           "key" => "value",
                           "array[]" => vec!["item1", "item2", "item3"],
                           "foo" => "bar",
                           "single" => vec!["item"]
                       })
            ])
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/responses/query\" <by>returned <br>2 <by>error(s)

<br> 1) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/two?key=value&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&foo=bar&single=item\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Expected <bg>a request <by>for the response, but got <br>none<by>.

<br> 2) <br>Request Failure: <by>Unexpected <bc>GET <by>request to \"<bc>https://example.com<bc>/one?key=value&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&foo=bar&single=item\"<by>, no response was provided.


"#, actual);

}

