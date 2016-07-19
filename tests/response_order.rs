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
<br>Response Failure: <bn>GET <by>request to \"<bn>http://localhost:4000<bn>/responses/two\" <by>returned <br>2 <by>error(s)

<bb> 1) <br>Request Failure: <bn>GET <by>response provided for \"<bn>https://example.com<bn>/two\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Response fetched out of order, <bg>provided for request <bb>1<by>, <br>fetched by request <bb>2<by>.

<bb> 2) <br>Request Failure: <bn>GET <by>response provided for \"<bn>https://example.com<bn>/one\" <by>returned <br>1 <by>error(s)

    <bb> 2.1) <by>Response fetched out of order, <bg>provided for request <bb>2<by>, <br>fetched by request <bb>1<by>.


"#, actual);

}

