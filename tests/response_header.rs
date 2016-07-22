#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();

#[test]
fn test_responses_provided_with_header() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                       .with_header(Accept(vec![
                            qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
                       ]))
            ])
            .expected_header(Accept(vec![
                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
            ]))
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_responses_provided_with_expected_header() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                       .expected_header(Accept(vec![
                            qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
                       ]))
            ])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_responses_provided_with_expected_header_multiple() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                       .expected_headers(headers![
                            Accept(vec![
                                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
                            ])
                       ])
            ])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_responses_provided_with_expected_header_mismatch() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                       .expected_header(Accept(vec![
                            qitem(Mime(TopLevel::Text, SubLevel::Plain, vec![]))
                       ]))
            ])
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>GET <by>request to \"<bn>http://localhost:4000<bn>/responses/one\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>GET <by>response provided for \"<bn>https://example.com<bn>/one\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>header \"<bb>Accept\" <by>does not match, expected:

              \"<bg>text/plain\"

          <by>but got:

              \"<br>application/json\"

          <by>difference:

              \"<gbr>application/json <gbg>text/plain\"


"#, actual);

}

#[test]
fn test_responses_provided_with_unexpected_header() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                       .unexpected_header::<ContentType>()
            ])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_responses_provided_with_unexpected_header_mismatch() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                       .unexpected_header::<Accept>()
            ])
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>GET <by>request to \"<bn>http://localhost:4000<bn>/responses/one\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>GET <by>response provided for \"<bn>https://example.com<bn>/one\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>header \"<bb>Accept\" <by>was expected <bg>to be absent<by>, but <br>is present<by>.


"#, actual);

}

