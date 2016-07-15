#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();
use json::JsonValue;



// Expected Body --------------------------------------------------------------
#[test]
fn test_body_expected_text() {

    let actual = {
        API::get("/get/hello")
            .expected_body("Hello World")
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_body_with_expected() {

    let actual = {
        API::post("/body/echo")
            .with_body("Form Data")
            .expected_body("Form Data")
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_body_text_mismatch_diff_added() {

    let actual = {
        API::get("/get/hello")
            .expected_body("Hello     \n")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/get/hello\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>does not match, expected:

        \"<bg>Hello     \\n\"

    <by>but got:

        \"<br>Hello World\"

    <by>difference:

        \"Hello <gbg>World <gbr>\\n\"


"#, actual);

}

#[test]
fn test_body_text_mismatch_diff_removed() {

    let actual = {
        API::get("/get/hello")
            .expected_body("Hello World Message\n")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>GET <by>request to \"<bc>http://localhost:4000<bc>/get/hello\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>does not match, expected:

        \"<bg>Hello World Message\\n\"

    <by>but got:

        \"<br>Hello World\"

    <by>difference:

        \"Hello World <gbr>Message\\n\"


"#, actual);

}

#[test]
fn test_body_with_expected_text_invalid_utf8() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(
                Mime(TopLevel::Text, SubLevel::Plain, vec![])
            ))
            .with_body([0xf8, 0xa1, 0xa1, 0xa1, 0xa1].to_vec())
            .expected_body("")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/echo\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>text body contains invalid UTF-8:

        <br>Utf8Error { valid_up_to: 0 }.


"#, actual);

}

#[test]
fn test_body_with_expected_text_expected_invalid_utf8() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(
                Mime(TopLevel::Text, SubLevel::Plain, vec![])
            ))
            .with_body("")
            .expected_body([0xf8, 0xa1, 0xa1, 0xa1, 0xa1].to_vec())
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/echo\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>body, expected text provided by test contains invalid UTF-8:

        <br>Utf8Error { valid_up_to: 0 }


"#, actual);

}

#[test]
fn test_body_expected_raw() {

    let actual = {
        API::post("/echo")
            .with_body([
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89
            ].to_vec())
            .expected_body([
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89
            ].to_vec())
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_body_expected_raw_mismatch() {

    let actual = {
        API::post("/echo")
            .with_body([
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89
            ].to_vec())
            .expected_body([
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89,
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78
            ].to_vec())
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/echo\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>raw body data does not match, expected the following <bg>16 bytes<by>:

       [<bg>0x67, <bg>0x08, <bg>0x90, <bg>0xCA, <bg>0xD4, <bg>0xE5, <bg>0xF4, <bg>0x89, <bg>0x00, <bg>0xA0, <bg>0xFF, <bg>0x80, <bg>0x45, <bg>0x13, <bg>0x21, <bg>0x78]

    <by>but got the following <br>16 bytes <by>instead:

       [<br>0x00, <br>0xA0, <br>0xFF, <br>0x80, <br>0x45, <br>0x13, <br>0x21, <br>0x78, <br>0x67, <br>0x08, <br>0x90, <br>0xCA, <br>0xD4, <br>0xE5, <br>0xF4, <br>0x89]


"#, actual);

}

#[test]
fn test_body_with_expected_json() {

    let actual = {
        API::post("/echo")
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
            .expected_body(object! {
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
fn test_body_with_expected_json_mismatch() {

    let actual = {
        API::post("/echo")
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
            .expected_body(object! {
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

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/echo\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>body json does not match, expected:

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
fn test_body_with_expected_json_mismatch_exact() {

    let actual = {
        API::post("/echo")
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
            .expected_exact_body(object! {
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

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/echo\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>body json does not match, expected:

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
fn test_body_with_expected_json_invalid() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(
                Mime(TopLevel::Application, SubLevel::Json, vec![])
            ))
            .with_body("{\"foo\": }")
            .expected_body(object! {
                "key" => "value"
            })
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/echo\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>body json is invalid:

        <br>UnexpectedCharacter { ch: \'}\', line: 1, column: 9 }.


"#, actual);

}

#[test]
fn test_body_with_expected_json_invalid_utf8() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(
                Mime(TopLevel::Application, SubLevel::Json, vec![])
            ))
            .with_body([0xf8, 0xa1, 0xa1, 0xa1, 0xa1].to_vec())
            .expected_body(object! {
                "key" => "value"
            })
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/echo\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>body json contains invalid UTF-8:

        <br>Utf8Error { valid_up_to: 0 }.


"#, actual);

}

#[test]
fn test_body_with_expected_json_expected_invalid() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(
                Mime(TopLevel::Application, SubLevel::Json, vec![])
            ))
            .with_body(object! {
                "key" => "value"
            })
            .expected_body("{\"foo\": }")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/echo\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>body, expected <by>body json provided by test is invalid:

        <br>UnexpectedCharacter { ch: \'}\', line: 1, column: 9 }


"#, actual);

}

#[test]
fn test_body_with_expected_json_expected_invalid_utf8() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(
                Mime(TopLevel::Application, SubLevel::Json, vec![])
            ))
            .with_body(object! {
                "key" => "value"
            })
            .expected_body([0xf8, 0xa1, 0xa1, 0xa1, 0xa1].to_vec())
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/echo\" <by>returned <br>1 <by>error(s)

<br> 1) <by>Response <by>body, expected <by>body json provided by test contains invalid UTF-8:

        <br>Utf8Error { valid_up_to: 0 }


"#, actual);

}


// Set Headers from Body ------------------------------------------------------
#[test]
fn test_body_set_header_from_body_raw() {

    let actual = {
        API::post("/echo")
            .with_body(vec![1, 2, 3, 4, 5])
            .expected_body(vec![1, 2, 3, 4, 5])
            .expected_header(ContentType(
                Mime(TopLevel::Application, SubLevel::OctetStream, vec![])
            ))
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_body_set_header_from_body_text() {

    let actual = {
        API::post("/echo")
            .with_body("Hello World")
            .expected_body("Hello World")
            .expected_header(ContentType(
                Mime(TopLevel::Text, SubLevel::Plain, vec![])
            ))
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_body_set_header_from_body_json() {

    let actual = {
        API::post("/echo")
            .with_body(object! {
                "key" => "value"
            })
            .expected_body(object! {
                "key" => "value"
            })
            .expected_header(ContentType(
                Mime(TopLevel::Application, SubLevel::Json, vec![])
            ))
            .collect()
    };

    assert_pass!(actual);

}

#[test]
fn test_body_override_header_from_body() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(
                Mime(TopLevel::Text, SubLevel::Plain, vec![])
            ))
            .with_body(object! {
                "key" => "value"
            })
            .expected_body("{\"key\":\"value\"}")
            .expected_header(ContentType(
                Mime(TopLevel::Text, SubLevel::Plain, vec![])
            ))
            .collect()
    };

    assert_pass!(actual);

}

