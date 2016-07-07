#[macro_use] extern crate json;
#[macro_use] extern crate noir;
#[macro_use]
mod base_test;
test!();

#[derive(Copy, Clone, Default)]
pub struct TimeoutAPI;
impl HttpApi for TimeoutAPI {

    fn hostname(&self) -> &'static str {
        "localhost"
    }

    fn port(&self) -> u16 {
        4001
    }

    fn start(&self) {
        // Do nothing so we timeout
    }

}

#[test]
fn test_api_start_timeout() {

    let actual = {
        TimeoutAPI::get("/").collect()
    };

    assert_fail!(r#"
<br>Noir Api Failure: <by>Server for \"<bc>http://localhost:4001\" <by>did not respond within <bn>1000ms<by>.

"#, actual);

}


#[test]
fn test_api_start_multiple() {

    let actual = {
        API::get("/").collect()
    };

    assert_pass!(actual);

}

