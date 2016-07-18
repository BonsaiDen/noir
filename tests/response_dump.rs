#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();


// Dump request bodies --------------------------------------------------------
#[test]
fn test_provided_response_dump_text() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").dump()
            ])
            .with_body("Response Body")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>headers dump:

              <bn>Content-Length: <bp>13
              <bn>  Content-Type: <bp>text/plain
              <bn>          Host: <bp>example.com

          <by>Request <by>body dump:

              \"<bp>Response Body\"


"#, actual);

}

#[test]
fn test_provided_response_dump_invalid_utf8() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").dump()
            ])
            .with_body([0xf8, 0xa1, 0xa1, 0xa1, 0xa1].to_vec())
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>headers dump:

              <bn>Content-Length: <bp>5
              <bn>  Content-Type: <bp>text/plain
              <bn>          Host: <bp>example.com

          <by>Request <by>text body contains invalid UTF-8:

              <br>Utf8Error { valid_up_to: 0 }


"#, actual);

}

#[test]
fn test_provided_response_dump_raw() {

    let actual = {
        API::post("/response/forward")
            .provide(responses![
                EXAMPLE.post("/forward").dump()
            ])
            .with_body(vec![
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89
            ])
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>headers dump:

              <bn>Content-Length: <bp>16
              <bn>  Content-Type: <bp>application/octet-stream
              <bn>          Host: <bp>example.com

          <by>Request <by>raw body dump of <bn>16 bytes<by>:

             [<bp>0x00, <bp>0xA0, <bp>0xFF, <bp>0x80, <bp>0x45, <bp>0x13, <bp>0x21, <bp>0x78, <bp>0x67, <bp>0x08, <bp>0x90, <bp>0xCA, <bp>0xD4, <bp>0xE5, <bp>0xF4, <bp>0x89]


"#, actual);

}

#[test]
fn test_provided_response_dump_json() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").dump()
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

    <br> 1.1) <by>Request <by>headers dump:

              <bn>Content-Length: <bp>121
              <bn>  Content-Type: <bp>application/json
              <bn>          Host: <bp>example.com

          <by>Request <by>body dump:

              <bn>{
                  \"additional\": 32,
                  \"key\": \"different value\",
                  \"list\": [
                      2,
                      3
                  ],
                  \"some\": {
                      \"very\": {
                          \"deeply\": {
                              \"nested\": {
                                  \"array\": [
                                      false,
                                      true,
                                      false
                                  ]
                              }
                          }
                      }
                  }
              }


"#, actual);

}

#[test]
fn test_provided_response_dump_json_invalid() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").dump()
            ])
            .with_body("{\"foo\": }")
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>headers dump:

              <bn>Content-Length: <bp>9
              <bn>  Content-Type: <bp>application/json
              <bn>          Host: <bp>example.com

          <by>Request <by>body JSON is invalid:

              <br>UnexpectedCharacter { ch: \'}\', line: 1, column: 9 }


"#, actual);

}

#[test]
fn test_provided_response_dump_json_invalid_utf8() {

    let actual = {
        API::post("/response/forward")
            .with_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .provide(responses![
                EXAMPLE.post("/forward").dump()
            ])
            .with_body([0xf8, 0xa1, 0xa1, 0xa1, 0xa1].to_vec())
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>headers dump:

              <bn>Content-Length: <bp>5
              <bn>  Content-Type: <bp>application/json
              <bn>          Host: <bp>example.com

          <by>Request <by>body JSON contains invalid UTF-8:

              <br>Utf8Error { valid_up_to: 0 }


"#, actual);

}

#[test]
fn test_provided_response_dump_form() {

    use std::fs::File;

    let actual = {
        API::post("/response/forward")
            .provide(responses![
                EXAMPLE.post("/forward").dump()
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

    assert_fail!(r#"
<br>Response Failure: <bc>POST <by>request to \"<bc>http://localhost:4000<bc>/response/forward\" <by>returned <br>1 <by>error(s)

<br> 1) <br>Request Failure: <bc>POST <by>response provided for \"<bc>https://example.com<bc>/forward\" <by>returned <br>1 <by>error(s)

    <br> 1.1) <by>Request <by>headers dump:

              <bn>Content-Length: <bp>857
              <bn>  Content-Type: <bp>application/form-data; boundary=boundary12345
              <bn>          Host: <bp>example.com

          <by>Request <by>form dump with <bn>5 fields<by>:

              <bc> 1) <by>Field \"<bn>field\" <by>dump:

                    \"<bp>someValue\\n\"

              <bc> 2) <by>Array \"<bn>array[]\" (<bp>5 items) <by>dump:

                    \"<bp>1\", \"<bp>2\", \"<bp>3\", \"<bp>4\", \"<bp>5\\n\"

              <bc> 3) <by>File \"<bn>vec_file\" (\"<bp>file.bin\", <bp>application/octet-stream) <by>raw body dump of <bn>8 bytes<by>:

                   [<bp>0x01, <bp>0x02, <bp>0x03, <bp>0x04, <bp>0x05, <bp>0x06, <bp>0x07, <bp>0x08]

              <bc> 4) <by>File \"<bn>str_file\" (\"<bp>readme.md\", <bp>text/plain) <by>body dump:

                    \"<bp>Hello World\"

              <bc> 5) <by>File \"<bn>fs_file\" (\"<bp>form_test.md\", <bp>text/plain) <by>body dump:

                    \"<bp>Form Test Data File\\n\"


"#, actual);
}

