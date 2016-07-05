#[macro_use]
mod base_test;
test!();


// With Query -----------------------------------------------------------------
#[test]
fn test_with_query_set() {

    let actual = {
        API::get("/query")
            .with_query(query!{
                "key" => "value",
                "array[]" => vec!["item1", "item2", "item3"],
                "foo" => "bar",
                "single" => vec!["item"]
            })
            .expected_body("Route not found: GET /query?key=value&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&foo=bar&single=item")
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_with_query_set_empty() {

    let actual = {
        API::get("/query")
            .with_query(query!{})
            .expected_body("Route not found: GET /query")
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_with_query_replace_existing() {

    let actual = {
        API::get("/query?existing=querystring")
            .with_query(query!{
                "key" => "value",
                "array[]" => vec!["item1", "item2", "item3"],
                "foo" => "bar",
                "single" => vec!["item"]
            })
            .expected_body("Route not found: GET /query?key=value&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&foo=bar&single=item")
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_with_query_replace_existing_with_empty() {

    let actual = {
        API::get("/query?existing=querystring")
            .with_query(query!{})
            .expected_body("Route not found: GET /query")
            .collect()
    };

    assert_pass!(actual);

}


#[test]
fn test_with_query_none_string_types() {

    let actual = {
        API::get("/query")
            .with_query(query!{
                "key" => 2,
                "array[]" => vec!["item1", "item2", "item3"],
                "foo" => 54.2,
                "single" => true
            })
            .expected_body("Route not found: GET /query?key=2&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&foo=54.2&single=true")
            .collect()
    };

    assert_pass!(actual);

}

