#[macro_use] extern crate json;
#[macro_use] extern crate noir;
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
<br>Response Failure: <bn>GET <by>request to \"<bn>http://localhost:4000<bn>/responses/query\" <by>returned <br>2 <by>error(s)

<bb> 1) <br>Request Failure: <bn>GET <by>response provided for \"<bn>https://example.com<bn>/two?key=value&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&foo=bar&single=item\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Expected <bg>a request <by>for the response, but got <br>none<by>.

<bb> 2) <br>Request Failure: <by>Unexpected <bn>GET <by>request to \"<bn>https://example.com<bn>/one?key=value&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&foo=bar&single=item\"<by>, no response was provided.


"#, actual);

}

