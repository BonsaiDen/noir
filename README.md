# noir [![Build Status](https://img.shields.io/travis/BonsaiDen/noir/master.svg?style=flat-square)](https://travis-ci.org/BonsaiDen/noir) [![Build status](https://img.shields.io/appveyor/ci/BonsaiDen/noir/master.svg?style=flat-square)](https://ci.appveyor.com/project/BonsaiDen/noir) [![Dependency Status](https://img.shields.io/badge/Dependency%20CI-passing-brightgreen.svg?style=flat-square)](https://dependencyci.com/github/BonsaiDen/noir)  [![Crates.io](https://img.shields.io/crates/v/noir.svg?style=flat-square)](https://crates.io/crates/noir) [![License](https://img.shields.io/crates/l/noir.svg?style=flat-square)]() 

A [rust](https://rust-lang.org/) based, DSL alike and request driven, black box testing library 
for HTTP APIs. 

- [Documentation](https://bonsaiden.github.io/noir/doc/noir/) for the latest release on crates.io.


> Please Note: **noir** is still work in progress, there are lots of typos in 
> its docs, quite a few bugs in the code and most likely also few API that won't 
> be the same in the final release.


## Screenshots

![noir](https://cloud.githubusercontent.com/assets/124674/16967785/2129ad12-4e0b-11e6-80f0-d741e8122e04.gif)


## Features

- [x] Describe your API and the external resources it accesses
- [x] Setup and configure HTTP requests against your API

  - [x] Perform requests with specific headers, query strings and bodies
  - [x] Set up expectations for response headers and bodies

- [x] Setup and provide external, mocked HTTP responses to your API

  - [x] These are only available for one specific request
  - [x] Any unexpected external calls made by your API will be caught
  - [x] Request order of the provided responses is also verified
  - [x] Set up expectations for headers and bodies of the requests your API performs

- [x] Detailed and colored test output to helps you quickly figuring out what exactly went wrong
- [x] Great support for JSON featuring deep, detailed object and array diffing, showing you paths, types and values
- [x] Uses hyper for all HTTP related interfaces


### Work in Progress / Unstable

- [ ] Macros for easy definition of HTTP multipart forms in tests
- [ ] API for providing custom / external mocks to be active during a test request


## Testing your API with noir

Since **noir** provides the ability to mock out certain parts of your application 
in tests, you need to run your tests as *library* tests so the mocks can be
enabled during testing.

A fully working example project that integrates with **noir** can be found in 
the `examples/api` folder.

Below you'll find a high level overview of the setup steps required for testing.

### Describing your API 

**noir** comes with direct support for testing HTTP based apis, to describe your 
own HTTP based API create a structure which implements the `HttpApi` trait.

```rust
use noir::HttpApi;

#[derive(Copy, Clone, Default)]
pub struct Api;
impl HttpApi for Api {

    fn hostname(&self) -> &'static str {
        "localhost"
    }

    fn port(&self) -> u16 {
        4000
    }

    fn start(&self) {
        application::server::run(self.host().as_str());
    }

}
```

There are only three things you have to tell **noir** here:

1. The hostname your application will be listening on during testing
2. The port your application will be listening on during testing
3. What *blocking* function will start your webserver 

When executing your tests, **noir** will wait for your webserver to start up and 
then run each test in series. 

Now, the reason for not running in parallel is that once you start using 
**noir** provided macros like `hyper_client!()` (which enable you to mock 
responses to outgoing HTTP requests from your application) there is no simply 
way (i.e. without adding additional logic to your application) to match these 
requests to the responses provided by each test and it would also be rather 
unclear which exact test should fail should your application perform additional, 
unexpected HTTP requests during these tests.

### Describing External Resources


```rust
use noir::HttpEndpoint;
#[derive(Copy, Clone)]
pub struct ExternalResource;
impl HttpEndpoint for ExternalResource {

    fn hostname(&self) -> &'static str {
        "external-resouce.com"
    }

    fn port(&self) -> u16 {
        443
    }

}
```

### Test Requests

Each **noir** test starts with a HTTP Method call on your defined `API` structure, all of these calls then return a `HttpRequest` instance.

A `HttpRequest` instance allows you to set up both the data and expectations for your test request.

You can also provide external resource responses which will be available to your application for the time the request is running.

Once a `HttpRequest` instance goes out of scope, its constructed request is automatically send and any of its expectations are validated.

Below is a, rather contrived, example of what is possible. For full details 
please refer to the [Documentation](https://bonsaiden.github.io/noir/doc/noir/).


```rust
#[macro_use]
extern crate noir;

#[test]
fn test_get_resource_with_missing_optional_data() {

    // Perform a request against our API
    Api::get("/")
        
        // Set up our query string
        .with_query(query! {
            "page" => 2,
            "sort" => "asc",
            "detailed" => true
        })

        // Set the headers of the request
        .with_headers(headers![
            Accept(vec![
                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
            ])
        ])

        // Provide some mocked, external resource responses during the api request
        .provide(responses![

            // Provide a resource for "/data/base.json" that responds with a
            // "200 OK" and a json body and expects a JSON Accept header.
            ExternalResource.get("/data/base.json")
                            .with_status(StatusCode::Ok)
                            .with_body(!object {
                                "key" => "value"
                            })
                            .expected_header(Accept(vec![
                                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
                            ])),

            // Provide another resource with responds with "500"
            ExternalResource.get("/data/optional.json")
                            .with_status(StatusCode::InternalServerError)
                            .expected_header(Accept(vec![
                                qitem(Mime(TopLevel::Application, SubLevel::Json, vec![]))
                            ]))
        ])

        // Expect a "200 OK" response from our API
        .expected_status(StatusCode::Ok)

        // Expect a JSON Content-Type header on our response
        .expected_header(ContentType(
            Mime(TopLevel::Application, SubLevel::Json, vec![])
        ))

        // And finally expect a JSON body
        .expected_body(object!{
            "resource" => object! {
                "key" => "value"
            },
            "optional" => JsonValue::Null
        });
}
```

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

