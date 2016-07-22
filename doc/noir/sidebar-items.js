initSidebarItems({"macro":[["form!","A macro for creating a `HttpFormData ` instance."],["headers!","A convenience macro for creating a vector of `HttpHeader` items."],["hyper_client!","A macro for intercepting `hyper::Client::new()` calls made during tests."],["mocks!","A convenience macro for creating a vector of `Box<MockProvider>` items."],["query!","A macro for creating a `HttpQueryString` instance."],["responses!","A convenience macro for creating a vector of `Box<MockResponse>` items."]],"struct":[["HttpBody","An abstraction over different data types used for HTTP request bodies."],["HttpFormData","An abstraction over HTTP form data."],["HttpHeader","An abstraction over different `hyper::Header` implementations."],["HttpQueryString","An abstraction over a HTTP query string."],["HttpRequest","A HTTP request for API testing."],["HttpResponse","A mocked HTTP response that is being provided to a testable API."],["MockResponseProvider","An interface for `MockRequest` trait implementations allowing them to consume matching `MockResponse` objects that were provided for the current test."],["Options","Additional configuration options for API requests and responses."]],"trait":[["HttpApi","A trait for the description of a testable, HTTP based API."],["HttpEndpoint","A trait for the description of a HTTP based endpoint used to provided mocked responses to a testable API."],["MockProvider","A trait for implementation of a custom mock provider."],["MockRequest","A trait for implementation of a request matched against concrete types of `MockResponse`."],["MockResponse","A trait for implementation of a response provided to a concrete type of `MockRequest`."]],"type":[["MockRequestResponse","A response to a request made against a mocked endpoint."]]});