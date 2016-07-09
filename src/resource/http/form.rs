// (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::fs::File;
use std::io::Read;


// External Dependencies ------------------------------------------------------
use json;
use rand;
use rand::Rng;
use url::form_urlencoded::Serializer;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};


/// An abstraction over HTTP form data.
///
/// Created by the `form!{...}` macro:
///
/// ```rust
/// # #[macro_use] extern crate noir;
/// # #[macro_use] extern crate json;
/// # #[macro_use] extern crate hyper;
/// # use std::fs::File;
/// # use hyper::mime::{Mime, TopLevel, SubLevel};
/// # fn main() {
/// form! {
///     "field" => "value",
///     "array[]" => vec![1, 2, 3, 4, 5],
///     "fs_file" => (
///         "Cargo.toml",
///         Mime(TopLevel::Text, SubLevel::Ext("toml".to_string()), vec![]),
///         File::open("Cargo.toml").unwrap()
///     ),
///     "vec_file" => (
///         "data.bin",
///         Mime(TopLevel::Application, SubLevel::OctetStream, vec![]),
///         vec![1, 2, 3, 4, 5, 6, 7, 8]
///     ),
///     "str_file" => (
///         "readme.txt",
///         Mime(TopLevel::Text, SubLevel::Plain, vec![]),
///         "Hello World"
///     ),
///     "json_file" => (
///         "data.json",
///         Mime(TopLevel::Application, SubLevel::Json, vec![]),
///         object! {
///             "key" => "value"
///         }
///     )
/// };
/// # }
/// ```
pub struct HttpFormData {
    mime: SubLevel,
    fields: Vec<HttpFormDataField>
}


// Internal -------------------------------------------------------------------
#[doc(hidden)]
impl HttpFormData {
    pub fn new(fields: Vec<HttpFormDataField>) -> HttpFormData {

        // Default to non-multipart mime forms
        let mut mime = SubLevel::WwwFormUrlEncoded;

        // Check if any of the fields are files and switch to the corresponding
        // mime type if necessary.
        for field in &fields {
            if let &HttpFormDataField::FileVec(_, _, _, _) = field {
                mime = SubLevel::FormData;
                break;

            } else if let &HttpFormDataField::FileFs(_, _, _, _) = field {
                mime = SubLevel::FormData;
                break;
            }
        }

        HttpFormData {
            mime: mime,
            fields: fields
        }

    }

    fn into_body_parts(self) -> (Mime, Vec<u8>) {

        // TODO IW: Support nested form-data (multipart/mixed)
        if self.mime == SubLevel::FormData {

            let mut body = Vec::new();
            let mut rng = rand::thread_rng();
            let mut parts: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

            // Convert form fields into multiparts
            for field in self.fields {
                match field {
                    HttpFormDataField::Value(name, value) => {
                        parts.push((
                            format!(
                                "\r\nContent-Disposition: form-data; name=\"{}\"\r\n\r\n",
                                name

                            ).as_bytes().to_vec(),
                            value.as_bytes().to_vec()
                        ));
                    },
                    HttpFormDataField::Array(name, values) => {
                        for value in values {
                            parts.push((
                                format!(
                                    "\r\nContent-Disposition: form-data; name=\"{}\"\r\n\r\n",
                                    name

                                ).as_bytes().to_vec(),
                                value.as_bytes().to_vec()
                            ));
                        }
                    },
                    HttpFormDataField::FileVec(name, filename, mime, data) => {
                        parts.push((
                            format!(
                                "\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\nContent-Type: {}\r\n\r\n",
                                name,
                                filename,
                                mime

                            ).as_bytes().to_vec(),
                            data
                        ));
                    },
                    HttpFormDataField::FileFs(name, filename, mime, mut file) => {
                        let mut data = Vec::new();
                        file.read_to_end(&mut data).expect("Failed to read file.");
                        parts.push((
                            format!(
                                "\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\nContent-Type: {}\r\n\r\n",
                                name,
                                filename,
                                mime

                            ).as_bytes().to_vec(),
                            data
                        ));
                    }
                }
            };

            // Generate form boundary
            // TODO IW: Check for collisions in the data
            let boundary = format!("boundary{}{}", rng.next_u64(), rng.next_u64());
            let full_boundary = format!("\r\n--{}", boundary);
            let boundary_line = full_boundary.as_bytes();

            // Build body payload
            for (mut headers, mut data) in parts {
                body.extend_from_slice(boundary_line);
                body.append(&mut headers);
                body.append(&mut data);
            }

            body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

            (
                Mime(TopLevel::Application, self.mime, vec![
                    (Attr::Boundary, Value::Ext(boundary.to_string()))
                ]),
                body
            )

        } else {
            let mut serializer = Serializer::new(String::new());
            for field in self.fields {
                match field {
                    HttpFormDataField::Value(name, value) => {
                        serializer.append_pair(name.as_str(), value.as_str());
                    },
                    HttpFormDataField::Array(name, values) => {
                        for value in values {
                            serializer.append_pair(name.as_str(), value.as_str());
                        }
                    },
                    _ => unreachable!()
                }
            }
            (
                Mime(TopLevel::Application, self.mime, vec![]),
                serializer.finish().into()
            )
        }

    }

}


pub fn http_form_into_body_parts(form: HttpFormData) -> (Mime, Vec<u8>) {
    form.into_body_parts()
}

#[doc(hidden)]
pub enum HttpFormDataField {
    Value(String, String),
    Array(String, Vec<String>),
    FileVec(String, String, Mime, Vec<u8>),
    FileFs(String, String, Mime, File)
}

macro_rules! impl_form_data_field_type {
    ($T:ty) => (

        impl From<(&'static str, $T)> for HttpFormDataField {
            fn from(item: (&'static str, $T)) -> HttpFormDataField {
                HttpFormDataField::Value(
                    item.0.to_string(),
                    item.1.to_string()
                )
            }
        }

        impl From<(&'static str, Vec<$T>)> for HttpFormDataField {
            fn from(item: (&'static str, Vec<$T>)) -> HttpFormDataField {
                HttpFormDataField::Array(
                    item.0.to_string(),
                    item.1.iter().map(|s| s.to_string()).collect()
                )
            }
        }

    )
}

impl From<(&'static str, (&'static str, Mime, File))> for HttpFormDataField {
    fn from(item: (&'static str, (&'static str, Mime, File))) -> HttpFormDataField {
        HttpFormDataField::FileFs(
            item.0.to_string(),
            (item.1).0.to_string(),
            (item.1).1,
            (item.1).2
        )
    }
}

impl From<(&'static str, (&'static str, Mime, &'static str))> for HttpFormDataField {
    fn from(item: (&'static str, (&'static str, Mime, &'static str))) -> HttpFormDataField {
        HttpFormDataField::FileVec(
            item.0.to_string(),
            (item.1).0.to_string(),
            (item.1).1,
            (item.1).2.into()
        )
    }
}

impl From<(&'static str, (&'static str, Mime, String))> for HttpFormDataField {
    fn from(item: (&'static str, (&'static str, Mime, String))) -> HttpFormDataField {
        HttpFormDataField::FileVec(
            item.0.to_string(),
            (item.1).0.to_string(),
            (item.1).1,
            (item.1).2.into()
        )
    }
}

impl From<(&'static str, (&'static str, Mime, Vec<u8>))> for HttpFormDataField {
    fn from(item: (&'static str, (&'static str, Mime, Vec<u8>))) -> HttpFormDataField {
        HttpFormDataField::FileVec(
            item.0.to_string(),
            (item.1).0.to_string(),
            (item.1).1,
            (item.1).2
        )
    }
}

impl From<(&'static str, (&'static str, Mime, json::JsonValue))> for HttpFormDataField {
    fn from(item: (&'static str, (&'static str, Mime, json::JsonValue))) -> HttpFormDataField {
        HttpFormDataField::FileVec(
            item.0.to_string(),
            (item.1).0.to_string(),
            (item.1).1,
            json::stringify((item.1).2).into()
        )
    }
}

// TODO IW: Clean up, once impl specialization is stable
impl_form_data_field_type!(&'static str);
impl_form_data_field_type!(String);
impl_form_data_field_type!(bool);
impl_form_data_field_type!(f64);
impl_form_data_field_type!(i64);
impl_form_data_field_type!(u64);
impl_form_data_field_type!(i32);
impl_form_data_field_type!(u32);

