#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();

#[test]
fn test_responses_provided_out_of_order() {

    let actual = {
        API::get("/responses/two")
            .provide(responses![
                EXAMPLE.get("/two"),
                EXAMPLE.get("/one")
            ])
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/responses/two\" <by>returned <br>2 <by>error(s)

<bb> 1) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/two\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Response fetched out of order, <bg>provided for request <bb>1<by>, <br>fetched by request <bb>2<by>.

<bb> 2) <br>Request Failure: <bc>GET <by>response provided for \"<bc>https://example.com<bc>/one\" <by>returned <br>1 <by>error(s)

    <bb> 2.1) <by>Response fetched out of order, <bg>provided for request <bb>2<by>, <br>fetched by request <bb>1<by>.


"#, actual);

}

