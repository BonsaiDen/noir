#[macro_use]
mod base_test;
test!();


// Request Methods ------------------------------------------------------------
#[test]
fn test_method_options() {
    API::options("/echo/method").expected_body("OPTIONS").collect().unwrap();
}

#[test]
fn test_method_get() {
    API::get("/echo/method").expected_body("GET").collect().unwrap();
}

#[test]
fn test_method_post() {
    API::post("/echo/method").expected_body("POST").collect().unwrap();
}

#[test]
fn test_method_put() {
    API::put("/echo/method").expected_body("PUT").collect().unwrap();
}

#[test]
fn test_method_delete() {
    API::delete("/echo/method").expected_body("DELETE").collect().unwrap();
}

#[test]
fn test_method_head() {
    API::head("/echo/method").expected_body("").collect().unwrap();
}

#[test]
fn test_method_trace() {
    API::trace("/echo/method").expected_body("TRACE").collect().unwrap();
}

#[test]
fn test_method_connect() {
    API::connect("/echo/method").expected_body("").collect().unwrap();
}

#[test]
fn test_method_patch() {
    API::patch("/echo/method").expected_body("PATCH").collect().unwrap();
}

#[test]
fn test_method_ext() {
    API::ext("FOO", "/echo/method").expected_body("FOO").collect().unwrap();
}

