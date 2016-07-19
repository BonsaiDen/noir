#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();


// Dump Responses -------------------------------------------------------------
#[test]
fn test_dump_response_with_raw_body() {

    let actual = {
        API::post("/echo")
            .with_header(Accept(vec![
                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
            ]))
            .with_body([
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89,
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89,
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89,
                0x00, 0xa0, 0xff, 0x80, 0x45, 0x13, 0x21, 0x78,
                0x67, 0x08, 0x90, 0xca, 0xd4, 0xe5, 0xf4, 0x89
            ].to_vec())
            .dump()
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/echo\" <by>returned <br>1 <by>error(s)

<bb> 1) <by>Response <by>headers dump:

        <bn>        Accept: <bp>application/json
        <bn>Content-Length: <bp>64
        <bn>  Content-Type: <bp>application/octet-stream
        <bn>          Date: <bp>
        <bn>          Host: <bp>localhost:4000

    <by>Response <by>raw body dump of <bn>64 bytes<by>:

       [<bp>0x00, <bp>0xA0, <bp>0xFF, <bp>0x80, <bp>0x45, <bp>0x13, <bp>0x21, <bp>0x78, <bp>0x67, <bp>0x08, <bp>0x90, <bp>0xCA, <bp>0xD4, <bp>0xE5, <bp>0xF4, <bp>0x89,
        <bp>0x00, <bp>0xA0, <bp>0xFF, <bp>0x80, <bp>0x45, <bp>0x13, <bp>0x21, <bp>0x78, <bp>0x67, <bp>0x08, <bp>0x90, <bp>0xCA, <bp>0xD4, <bp>0xE5, <bp>0xF4, <bp>0x89,
        <bp>0x00, <bp>0xA0, <bp>0xFF, <bp>0x80, <bp>0x45, <bp>0x13, <bp>0x21, <bp>0x78, <bp>0x67, <bp>0x08, <bp>0x90, <bp>0xCA, <bp>0xD4, <bp>0xE5, <bp>0xF4, <bp>0x89,
        <bp>0x00, <bp>0xA0, <bp>0xFF, <bp>0x80, <bp>0x45, <bp>0x13, <bp>0x21, <bp>0x78, <bp>0x67, <bp>0x08, <bp>0x90, <bp>0xCA, <bp>0xD4, <bp>0xE5, <bp>0xF4, <bp>0x89]


"#, actual);

}

#[test]
fn test_dump_response_with_text_body() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])))
            .with_header(Accept(vec![
                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
            ]))
            .with_body("Hello World Body Text JSON \nA new line.")
            .dump()
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/echo\" <by>returned <br>1 <by>error(s)

<bb> 1) <by>Response <by>headers dump:

        <bn>        Accept: <bp>application/json
        <bn>Content-Length: <bp>39
        <bn>  Content-Type: <bp>text/plain
        <bn>          Date: <bp>
        <bn>          Host: <bp>localhost:4000

    <by>Response <by>body dump:

        \"<bp>Hello World Body Text JSON \\nA new line.\"


"#, actual);

}

#[test]
fn test_dump_response_with_text_body_invalid_utf8() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![])))
            .with_header(Accept(vec![
                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
            ]))
            .with_body([0xf8, 0xa1, 0xa1, 0xa1, 0xa1].to_vec())
            .dump()
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/echo\" <by>returned <br>1 <by>error(s)

<bb> 1) <by>Response <by>headers dump:

        <bn>        Accept: <bp>application/json
        <bn>Content-Length: <bp>5
        <bn>  Content-Type: <bp>text/plain
        <bn>          Date: <bp>
        <bn>          Host: <bp>localhost:4000

    <by>Response <by>text body contains invalid UTF-8:

        <br>Utf8Error { valid_up_to: 0 }


"#, actual);

}

#[test]
fn test_dump_response_with_json_body() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .with_header(Accept(vec![
                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
            ]))
            .with_body(object!{
                "key" => "Value",
                "number" => 123,
                "array" => vec![0, 1, 2, 3],
                "null" => json::Null
            })
            .dump()
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/echo\" <by>returned <br>1 <by>error(s)

<bb> 1) <by>Response <by>headers dump:

        <bn>        Accept: <bp>application/json
        <bn>Content-Length: <bp>58
        <bn>  Content-Type: <bp>application/json
        <bn>          Date: <bp>
        <bn>          Host: <bp>localhost:4000

    <by>Response <by>body dump:

        <bn>{
            \"array\": [
                0,
                1,
                2,
                3
            ],
            \"key\": \"Value\",
            \"null\": null,
            \"number\": 123
        }


"#, actual);

}

#[test]
fn test_dump_response_with_json_body_invalid() {

    let actual = {
        API::post("/echo")
            .with_header(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])))
            .with_body("{\"key\": }")
            .dump()
            .collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>POST <by>request to \"<bn>http://localhost:4000<bn>/echo\" <by>returned <br>1 <by>error(s)

<bb> 1) <by>Response <by>headers dump:

        <bn>Content-Length: <bp>9
        <bn>  Content-Type: <bp>application/json
        <bn>          Date: <bp>
        <bn>          Host: <bp>localhost:4000

    <by>Response <by>body JSON is invalid:

        <br>UnexpectedCharacter { ch: \'}\', line: 1, column: 9 }


"#, actual);

}

