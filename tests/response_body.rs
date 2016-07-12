#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();

use json::JsonValue;


// Provided response bodies for request ---------------------------------------
#[test]
fn test_provided_response_with_response_body_text() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_body("Response Body")
            ])
            .expected_body("Response Body")
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_with_response_body_raw() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_body(vec![1, 2, 3, 4, 5])
            ])
            .expected_body(vec![1, 2, 3, 4, 5])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_with_response_body_json() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_body(object!{
                    "key" => "value"
                })
            ])
            .expected_body(object!{
                "key" => "value"
            })
            .collect()
    };

    assert_pass!(actual);

}


// Request bodies from Responses ----------------------------------------------
#[test]
fn test_provided_response_with_expected_body_text() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").expected_body("Response Body")
            ])
            .with_body("Response Body")
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_with_expected_body_text_mismatch_diff_added() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").expected_body("Response")
            ])
            .with_body("Response Body")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>does not match, expected:

              \"<bg>Response\"

          <by>but got:

              \"<br>Response Body\"

          <by>difference:

              \"Response <gbg>Body\"


"#, actual);

}

#[test]
fn test_provided_response_with_expected_body_text_mismatch_diff_removed() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").expected_body("Response Body")
            ])
            .with_body("Response")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>does not match, expected:

              \"<bg>Response Body\"

          <by>but got:

              \"<br>Response\"

          <by>difference:

              \"Response <gbr>Body\"


"#, actual);

}

#[test]
fn test_provided_response_with_expected_body_text_invalid_utf8() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").expected_body("Response Body")
            ])
            .with_body([0xf8, 0xa1, 0xa1, 0xa1, 0xa1].to_vec())
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>text body contains invalid UTF-8:

              <br>Utf8Error { valid_up_to: 0 }.


"#, actual);
}

#[test]
fn test_provided_response_with_expected_body_raw() {

    let actual = {
        API::post("/response/forward")
            .provide(responses![
                EXAMPLE.post("/forward").expected_body(vec![
                    0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                    0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89
                ])
            ])
            .with_body(vec![
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89
            ])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_with_expected_body_raw_mismatch() {

    let actual = {
        API::post("/response/forward")
            .provide(responses![
                EXAMPLE.post("/forward").expected_body(vec![
                    0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                    0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89
                ])
            ])
            .with_body(vec![
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89,
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78
            ])
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>raw body data does not match, expected the following <bg>16 bytes<by>:

             [<bg>0x00, <bg>0xA0, <bg>0xFF, <bg>0x80, <bg>0x45, <bg>0x13, <bg>0x21, <bg>0x78, <bg>0x67, <bg>0x08, <bg>0x90, <bg>0xCA, <bg>0xD4, <bg>0xE5, <bg>0xF4, <bg>0x89]

          <by>but got the following <br>16 bytes <by>instead:

             [<br>0x67, <br>0x08, <br>0x90, <br>0xCA, <br>0xD4, <br>0xE5, <br>0xF4, <br>0x89, <br>0x00, <br>0xA0, <br>0xFF, <br>0x80, <br>0x45, <br>0x13, <br>0x21, <br>0x78]


"#, actual);

}

#[test]
fn test_provided_response_with_expected_body_json() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").expected_body(object! {
                    "key" => "value",
                    "list" => vec![2, 3, 4],
                    "some" => object! {
                        "very" => object! {
                            "deeply" => object! {
                                "nested" => object! {
                                    "array" => array![true, false]
                                }
                            }
                        }
                    },
                    "missing" => JsonValue::Null
                })
            ])
            .with_body(object! {
                "key" => "value",
                "list" => vec![2, 3, 4],
                "some" => object! {
                    "very" => object! {
                        "deeply" => object! {
                            "nested" => object! {
                                "array" => array![true, false]
                            }
                        }
                    }
                },
                "missing" => JsonValue::Null
            })
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_with_expected_body_json_mismatch() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").expected_body(object! {
                    "key" => "value",
                    "list" => vec![2, 3, 4],
                    "some" => object! {
                        "very" => object! {
                            "deeply" => object! {
                                "nested" => object! {
                                    "array" => array![true, false]
                                }
                            }
                        }
                    },
                    "missing" => JsonValue::Null
                })
            ])
            .with_body(object! {
                "key" => "different value",
                "list" => vec![2, 3],
                "some" => object! {
                    "very" => object! {
                        "deeply" => object! {
                            "nested" => object! {
                                "array" => array![false, true, false]
                            }
                        }
                    }
                },
                "additional" => 32
            })
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>body json does not match, expected:

              - <bb>json.<bb>key: <bg>String <by>does not match, expected:

                    \"<bg>different value\"

                <by>but got:

                    \"<br>value\"

                <by>difference:

                    \"<gbr>different value\"

              - <bb>json.<bb>list: <bg>Array <by>with <br>2 <by>item(s) does not match expected length of <bg>3

              - <bb>json.<bb>some.<bb>very.<bb>deeply.<bb>nested.<bb>array: <bg>Array <by>with <br>3 <by>item(s) does not match expected length of <bg>2

              - <bb>json.<bb>some.<bb>very.<bb>deeply.<bb>nested.<bb>array<bp>[0]: <bg>Boolean (<br>false) <by>does not match expected value (<bg>true)

              - <bb>json.<bb>some.<bb>very.<bb>deeply.<bb>nested.<bb>array<bp>[1]: <bg>Boolean (<br>true) <by>does not match expected value (<bg>false)

              - <bb>json: <bg>Object <by>is missing <br>1 <by>key(s) (<br>missing)


"#, actual);

}

#[test]
fn test_provided_response_with_expected_body_json_mismatch_exact() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").expected_exact_body(object! {
                    "key" => "value",
                    "list" => vec![2, 3, 4],
                    "some" => object! {
                        "very" => object! {
                            "deeply" => object! {
                                "nested" => object! {
                                    "array" => array![true, false]
                                }
                            }
                        }
                    },
                    "missing" => JsonValue::Null
                })
            ])
            .with_body(object! {
                "key" => "different value",
                "list" => vec![2, 3],
                "some" => object! {
                    "very" => object! {
                        "deeply" => object! {
                            "nested" => object! {
                                "array" => array![false, true, false]
                            }
                        }
                    }
                },
                "additional" => 32
            })
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>body json does not match, expected:

              - <bb>json.<bb>key: <bg>String <by>does not match, expected:

                    \"<bg>different value\"

                <by>but got:

                    \"<br>value\"

                <by>difference:

                    \"<gbr>different value\"

              - <bb>json.<bb>list: <bg>Array <by>with <br>2 <by>item(s) does not match expected length of <bg>3

              - <bb>json.<bb>some.<bb>very.<bb>deeply.<bb>nested.<bb>array: <bg>Array <by>with <br>3 <by>item(s) does not match expected length of <bg>2

              - <bb>json.<bb>some.<bb>very.<bb>deeply.<bb>nested.<bb>array<bp>[0]: <bg>Boolean (<br>false) <by>does not match expected value (<bg>true)

              - <bb>json.<bb>some.<bb>very.<bb>deeply.<bb>nested.<bb>array<bp>[1]: <bg>Boolean (<br>true) <by>does not match expected value (<bg>false)

              - <bb>json: <bg>Object <by>is missing <br>1 <by>key(s) (<br>missing)

              - <bb>json: <bg>Object <by>has <br>1 <by>additional unexpected key(s) (<br>additional)


"#, actual);

}

#[test]
fn test_provided_response_with_expected_body_json_invalid() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").expected_body(object! {
                    "key" => "value"
                })
            ])
            .with_body("{\"foo\": }")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>body contains invalid json:

              <br>UnexpectedCharacter { ch: \'}\', line: 1, column: 9 }.


"#, actual);
}

#[test]
fn test_provided_response_with_expected_body_json_invalid_utf8() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").expected_body(object! {
                    "key" => "value"
                })
            ])
            .with_body([0xf8, 0xa1, 0xa1, 0xa1, 0xa1].to_vec())
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>json body contains invalid UTF-8:

              <br>Utf8Error { valid_up_to: 0 }.


"#, actual);
}

#[test]
fn test_provided_response_set_header_from_body_raw() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_body(vec![1, 2, 3, 4, 5])
            ])
            .expected_header(ContentType(
                Mime(TopLevel::Application, SubLevel::OctetStream, vec![])
            ))
            .expected_body(vec![1, 2, 3, 4, 5])
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_set_header_from_body_text() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_body("Hello World")
            ])
            .expected_header(ContentType(
                Mime(TopLevel::Text, SubLevel::Plain, vec![])
            ))
            .expected_body("Hello World")
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_set_header_from_body_json() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_body(object! {
                    "key" => "value"
                })
            ])
            .expected_header(ContentType(
                Mime(TopLevel::Application, SubLevel::Json, vec![])
            ))
            .expected_body(object! {
                "key" => "value"
            })
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_override_header_from_body() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                    .with_header(ContentType(
                        Mime(TopLevel::Text, SubLevel::Plain, vec![])
                    ))
                    .with_body(object! {
                        "key" => "value"
                    })
            ])
            .expected_header(ContentType(
                Mime(TopLevel::Text, SubLevel::Plain, vec![])
            ))
            .expected_body("{\"key\":\"value\"}")
            .collect()
    };

    assert_pass!(actual);

}

