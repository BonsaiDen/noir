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

    fn timeout(&self) -> Duration {
        Duration::from_millis(500)
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
<br>API Failure: <by>Server for \"<bn>http://localhost:4001\" <by>did not respond within <bg>1000ms<by>.

"#, actual);

}


#[test]
fn test_api_start_multiple() {

    let actual = {
        API::get("/").collect()
    };

    assert_pass!(actual);

}


#[test]
fn test_api_request_timeout() {

    let actual = {
        API::get("/timeout").collect()
    };

    assert_fail!(r#"
<br>Response Failure: <bn>GET <by>request to \"<bn>http://localhost:4000<bn>/timeout\" <by>returned <br>1 <by>error(s)

<bb> 1) <br>API Failure: <by>No response within <bg>1000ms<by>.


"#, actual);

}

