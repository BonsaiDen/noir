#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();

use json::JsonValue;


// Provided response bodies for request ---------------------------------------
#[test]
fn test_provided_response_without_body() {

    use hyper::header::ContentLength;
    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
            ])
            // Content length should be set to 0
            .expected_header(ContentLength(0))
            .expected_body("Response Body")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>GET <by>request to \"<bn>http://localhost:4000<bn>/responses/one\" <by>returned <br>1 <by>error(s)

<bb> 1) <by>Response <by>raw body data does not match, expected the following <bg>13 bytes<by>:

       [<bg>0x52, <bg>0x65, <bg>0x73, <bg>0x70, <bg>0x6F, <bg>0x6E, <bg>0x73, <bg>0x65, <bg>0x20, <bg>0x42, <bg>0x6F, <bg>0x64, <bg>0x79]

    <by>but got the following <br>0 bytes <by>instead:

       []


"#, actual);

}

#[test]
fn test_provided_response_with_response_body_text() {

    use hyper::header::ContentLength;
    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_body("Response Body")
            ])
            .expected_header(ContentLength(13))
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

#[test]
fn test_provided_response_with_overriden_content_length() {

    use hyper::header::ContentLength;
    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                    .with_header(ContentLength(5))
            ])
            .expected_header(ContentLength(5))
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
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>text body does not match, expected:

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
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>text body does not match, expected:

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
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>text body contains invalid UTF-8:

              <br>Utf8Error { valid_up_to: 0 }


"#, actual);
}


// Raw Body -------------------------------------------------------------------
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
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>raw body data does not match, expected the following <bg>16 bytes<by>:

             [<bg>0x00, <bg>0xA0, <bg>0xFF, <bg>0x80, <bg>0x45, <bg>0x13, <bg>0x21, <bg>0x78, <bg>0x67, <bg>0x08, <bg>0x90, <bg>0xCA, <bg>0xD4, <bg>0xE5, <bg>0xF4, <bg>0x89]

          <by>but got the following <br>16 bytes <by>instead:

             [<br>0x67, <br>0x08, <br>0x90, <br>0xCA, <br>0xD4, <br>0xE5, <br>0xF4, <br>0x89, <br>0x00, <br>0xA0, <br>0xFF, <br>0x80, <br>0x45, <br>0x13, <br>0x21, <br>0x78]


"#, actual);

}


// JSON Body ------------------------------------------------------------------
#[test]
fn test_provided_response_with_expected_body_json() {

    let actual = {
        API::post("/response/forward")
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
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>body JSON does not match:

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
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>body JSON does not match:

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
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>body JSON is invalid:

              <br>UnexpectedCharacter { ch: \'}\', line: 1, column: 9 }


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
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>body JSON contains invalid UTF-8:

              <br>Utf8Error { valid_up_to: 0 }


"#, actual);
}

#[test]
fn test_provided_response_with_expected_body_json_compare_depth_one() {

    use noir::Options;
    let actual = {
        API::post("/response/forward")
            .provide(responses![
                EXAMPLE.post("/forward")
                    .with_options(Options {
                        // This will only enter the first level after the top level
                        json_compare_depth: 1,
                        .. Default::default()
                    })
                    .expected_body(object! {
                        "key" => "otherValue",
                        "deep" => object! {
                            "compare" => "bar"
                        }
                    })
            ])
            .with_body(object! {
                "key" => "value",
                "deep" => object! {
                    "compare" => "foo"
                }
            })
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>body JSON does not match:

              - <bb>json.<bb>key: <bg>String <by>does not match, expected:

                    \"<bg>value\"

                <by>but got:

                    \"<br>otherValue\"

                <by>difference:

                    \"<gbr>value <gbg>otherValue\"


"#, actual);

}

#[test]
fn test_provided_response_with_expected_body_json_compare_depth_zero() {

    use noir::Options;
    let actual = {
        API::post("/response/forward")
            .provide(responses![
                EXAMPLE.post("/forward")
                    .with_options(Options {
                        // This will perform no comparisons at all
                        json_compare_depth: 0,
                        .. Default::default()
                    })
                    .expected_body(object! {
                        "key" => "otherValue",
                        "deep" => object! {
                            "compare" => "bar"
                        }
                    })
            ])
            .with_body(object! {
                "key" => "value",
                "deep" => object! {
                    "compare" => "foo"
                }
            })
            .collect()
    };

    assert_pass!(actual);

}


// Form Body ------------------------------------------------------------------
#[test]
fn test_provided_response_with_expected_body_form() {

    use std::fs::File;
    let actual = {
        API::post("/response/forward")
            .provide(responses![
                EXAMPLE.post("/forward").expected_body(form! {
                    "field" => "someValue\n",
                    "array[]" => vec!["1", "2", "3", "4", "5\n"],
                    "vec_file" => (
                        "file.bin",
                        Mime(TopLevel::Application, SubLevel::OctetStream, vec![]),
                        vec![1, 2, 3, 4, 5, 6, 7, 8]
                    ),
                    "str_file" => (
                        "readme.md",
                        Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                        "Hello World"
                    ),
                    "fs_file" => (
                        "form_test.md",
                        Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                        File::open("./tests/form_test.md").unwrap()
                    )
                })
            ])
            .with_body(form! {
                "field" => "someValue\n",
                "array[]" => vec!["1", "2", "3", "4", "5\n"],
                "vec_file" => (
                    "file.bin",
                    Mime(TopLevel::Application, SubLevel::OctetStream, vec![]),
                    vec![1, 2, 3, 4, 5, 6, 7, 8]
                ),
                "str_file" => (
                    "readme.md",
                    Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                    "Hello World"
                ),
                "fs_file" => (
                    "form_test.md",
                    Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                    File::open("./tests/form_test.md").unwrap()
                )
            })
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_with_expected_body_form_mismatch() {

    let actual = {
        API::post("/response/forward")
            .provide(responses![
                EXAMPLE.post("/forward").expected_body(form! {
                    "field" => "someValue\n",
                    "missingField" => "value",
                    "mismatchedType" => "plain",
                    "array[]" => vec!["1", "2", "3", "4", "5\n"],
                    "vec_file" => (
                        "file.bin",
                        Mime(TopLevel::Application, SubLevel::OctetStream, vec![]),
                        vec![1, 2, 3, 4, 5, 6, 7, 8]
                    ),
                    "str_file" => (
                        "readme.md",
                        Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                        "Hello World"
                    )
                })
            ])
            .with_body(form! {
                "field" => "different someValue\n",
                "additionalField" => "value",
                "mismatchedType" => vec!["array"],
                "array[]" => vec!["1", "2", "3", "5\n"],
                "vec_file" => (
                    "other.bin",
                    Mime(TopLevel::Application, SubLevel::OctetStream, vec![]),
                    vec![1, 2, 3, 4, 5, 6, 7, 8]
                ),
                "str_file" => (
                    "readme.md",
                    Mime(TopLevel::Text, SubLevel::Html, vec![]),
                    "Hello World"
                )
            })
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>body form data does not match:

              - <bb>form.<bb>array[]: <bg>Array <by>with <br>4 <by>item(s) does not match expected length of <bg>5

              - <bb>form.<bb>array[]<bp>[3]: <bg>ArrayItem <by>value does not match, expected:

                    \"<bg>4\"

                <by>but got:

                    \"<br>5\\n\"

                <by>difference:

                    \"<gbr>4 <gbg>5\\n\"

              - <bb>form.<bb>field: <bg>Field <by>value does not match, expected:

                    \"<bg>someValue\\n\"

                <by>but got:

                    \"<br>different someValue\\n\"

                <by>difference:

                    \"<gbg>different someValue\\n\"

              - <bb>form.<bb>mismatchedType: <bg>Field <by>value does not match, expected:

                    \"<bg>plain\"

                <by>but got:

                    \"<br>array\"

                <by>difference:

                    \"<gbr>plain <gbg>array\"

              - <bb>form.<bb>str_file: <bg>MIME type (<br>text/html) <by>does not match expected value (<bg>text/plain)

              - <bb>form.<bb>vec_file: <bg>Filename (\"<br>other.bin\") <by>does not match expected value (\"<bg>file.bin\")

              - <bb>form: <by>Is missing <br>1 <by>plain field(s) (<br>missingField)


"#, actual);

}

#[test]
fn test_provided_response_with_expected_body_form_mismatch_exact() {

    let actual = {
        API::post("/response/forward")
            .provide(responses![
                EXAMPLE.post("/forward").expected_exact_body(form! {
                    "field" => "someValue\n",
                    "missingField" => "value",
                    "mismatchedType" => "plain",
                    "array[]" => vec!["1", "2", "3", "4", "5\n"],
                    "vec_file" => (
                        "file.bin",
                        Mime(TopLevel::Application, SubLevel::OctetStream, vec![]),
                        vec![1, 2, 3, 4, 5, 6, 7, 8]
                    ),
                    "str_file" => (
                        "readme.md",
                        Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                        "Hello World"
                    )
                })
            ])
            .with_body(form! {
                "field" => "different someValue\n",
                "additionalField" => "value",
                "mismatchedType" => vec!["array"],
                "array[]" => vec!["1", "2", "3", "5\n"],
                "vec_file" => (
                    "other.bin",
                    Mime(TopLevel::Application, SubLevel::OctetStream, vec![]),
                    vec![1, 2, 3, 4, 5, 6, 7, 8]
                ),
                "str_file" => (
                    "readme.md",
                    Mime(TopLevel::Text, SubLevel::Html, vec![]),
                    "Hello World"
                )
            })
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/response/forward\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>Request Failure: <bn>POST <by>response provided for \"<bn>https://example.com<bn>/forward\" <by>returned <br>1 <by>error(s)

    <bb> 1.1) <by>Request <by>body form data does not match:

              - <bb>form.<bb>array[]: <bg>Array <by>with <br>4 <by>item(s) does not match expected length of <bg>5

              - <bb>form.<bb>array[]<bp>[3]: <bg>ArrayItem <by>value does not match, expected:

                    \"<bg>4\"

                <by>but got:

                    \"<br>5\\n\"

                <by>difference:

                    \"<gbr>4 <gbg>5\\n\"

              - <bb>form.<bb>field: <bg>Field <by>value does not match, expected:

                    \"<bg>someValue\\n\"

                <by>but got:

                    \"<br>different someValue\\n\"

                <by>difference:

                    \"<gbg>different someValue\\n\"

              - <bb>form.<bb>mismatchedType: <bg>Field <by>value does not match, expected:

                    \"<bg>plain\"

                <by>but got:

                    \"<br>array\"

                <by>difference:

                    \"<gbr>plain <gbg>array\"

              - <bb>form.<bb>str_file: <bg>MIME type (<br>text/html) <by>does not match expected value (<bg>text/plain)

              - <bb>form.<bb>vec_file: <bg>Filename (\"<br>other.bin\") <by>does not match expected value (\"<bg>file.bin\")

              - <bb>form: <by>Is missing <br>1 <by>plain field(s) (<br>missingField)

              - <bb>form: <by>Has <br>1 <by>additional unexpected plain field(s) (<br>additionalField)


"#, actual);

}


// Headers from Body Type -----------------------------------------------------
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
fn test_provided_response_set_header_from_body_form() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_body(form! {
                    "field" => "value"
                })
            ])
            .expected_header(ContentType(
                Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, vec![])
            ))
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_set_header_from_body_form_with_file() {

    use hyper::mime::{Attr, Value};

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one").with_body(form! {
                    "file" => (
                        "filename",
                        Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                        "Data"
                    )
                })
            ])
            .expected_header(ContentType(
                Mime(TopLevel::Application, SubLevel::FormData, vec![
                    (Attr::Boundary, Value::Ext("boundary12345".to_string()))
                ])
            ))
            .collect()
    };

    assert_pass!(actual);

}


// Body Type Header Override --------------------------------------------------
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

#[test]
fn test_provided_response_override_header_from_form() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                    .with_header(ContentType(
                        Mime(TopLevel::Text, SubLevel::Plain, vec![])
                    ))
                    .with_body(form! {
                        "field" => "value"
                    })
            ])
            .expected_header(ContentType(
                Mime(TopLevel::Text, SubLevel::Plain, vec![])
            ))
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_provided_response_override_header_from_form_with_file() {

    let actual = {
        API::get("/responses/one")
            .provide(responses![
                EXAMPLE.get("/one")
                    .with_header(ContentType(
                        Mime(TopLevel::Text, SubLevel::Plain, vec![])
                    ))
                    .with_body(form! {
                        "file" => (
                            "filename",
                            Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                            "Data"
                        )
                    })
            ])
            .expected_header(ContentType(
                Mime(TopLevel::Text, SubLevel::Plain, vec![])
            ))
            .collect()
    };

    assert_pass!(actual);

}

