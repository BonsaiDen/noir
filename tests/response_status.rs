#[macro_use]
mod base_test;
test!();

#[test]
fn test_responses_provided_with_status() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_status(StatusCode::Forbidden)
            ])
            .expected_status(StatusCode::Forbidden)
            .collect()
    };

    assert_pass!(actual);

}

