// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::str;


// External Dependencies ------------------------------------------------------
use url::form_urlencoded::Serializer;


/// An abstraction over a HTTP query string.
///
/// Created by the `query!{...}` macro:
///
/// ```rust
/// # #[macro_use] extern crate noir;
/// # fn main() {
/// let qs = query! {
///     "key" => "value",
///     "array[]" => vec!["item1", "item2","item3"],
///     "number" => 42,
///     "valid" => true
/// };
///
/// assert_eq!(
///     qs.to_string(),
///     "key=value&array%5B%5D=item1&array%5B%5D=item2&array%5B%5D=item3&number=42&valid=true"
/// );
/// # }
/// ```
pub struct HttpQueryString {
    fields: Vec<HttpQueryStringItem>
}


// Internal -------------------------------------------------------------------
#[doc(hidden)]
impl HttpQueryString {
    pub fn new(fields: Vec<HttpQueryStringItem>) -> HttpQueryString {
        HttpQueryString {
            fields: fields
        }
    }
}

#[doc(hidden)]
impl ToString for HttpQueryString {
    fn to_string(&self) -> String {

        let mut query = Serializer::new(String::new());

        for item in &self.fields {
            match item {
                &HttpQueryStringItem::Value(ref key, ref value) => {
                    query.append_pair(
                        key.as_str(),
                        value.as_str()
                    );
                },
                &HttpQueryStringItem::Array(ref key, ref values) => {
                    for value in values {
                        query.append_pair(
                            key.as_str(),
                            value.as_str()
                        );
                    }
                }
            }
        }

        query.finish()

    }
}

pub enum HttpQueryStringItem {
    Value(String, String),
    Array(String, Vec<String>)
}

macro_rules! impl_query_string_item_type {
    ($T:ty) => (

        impl From<(&'static str, $T)> for HttpQueryStringItem {
            fn from(item: (&'static str, $T)) -> HttpQueryStringItem {
                HttpQueryStringItem::Value(
                    item.0.to_string(),
                    item.1.to_string()
                )
            }
        }

        impl From<(&'static str, Vec<$T>)> for HttpQueryStringItem {
            fn from(item: (&'static str, Vec<$T>)) -> HttpQueryStringItem {
                HttpQueryStringItem::Array(
                    item.0.to_string(),
                    item.1.iter().map(|s| s.to_string()).collect()
                )
            }
        }

    )

}

// TODO IW: Clean up, once impl specialization is stable
impl_query_string_item_type!(&'static str);
impl_query_string_item_type!(String);
impl_query_string_item_type!(bool);
impl_query_string_item_type!(f64);
impl_query_string_item_type!(i64);
impl_query_string_item_type!(u64);
impl_query_string_item_type!(i32);
impl_query_string_item_type!(u32);

