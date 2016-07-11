// (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use rand::Rng;
use std::fs::File;
use std::io::Read;


// External Dependencies ------------------------------------------------------
use json;
use rand;
use httparse;
use url::form_urlencoded;
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use hyper::header::{Headers, ContentType, ContentDisposition, DispositionParam};


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

pub fn http_form_into_fields(form: HttpFormData) -> Vec<HttpFormDataField> {
    form.fields
}

pub fn http_form_into_body_parts(form: HttpFormData) -> (Mime, Vec<u8>) {
    form.into_body_parts()
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
            let parts = form_fields_into_parts(self.fields);

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
            let mut serializer = form_urlencoded::Serializer::new(String::new());
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

pub fn parse_form_data(body: &[u8], boundary: Option<String>) -> Result<HttpFormData, String> {

    let mut fields = Vec::new();

    // FormData
    if let Some(boundary) = boundary {

        // Boundary
        let mut b = String::from("--");
        b.push_str(boundary.as_str());
        let boundary = b.as_bytes();

        // Split the body along the form boundaries
        let mut previous_index = 0;
        for (i, w) in body.windows(boundary.len()).enumerate() {
            if w == boundary {

                // Skip the split before the first actual field
                if previous_index != 0 {

                    // We'll use httparse so we need to pretend to be a http request
                    let mut part = b"POST / HTTP/1.1\r\n".to_vec();

                    part.extend_from_slice(
                        // Strip out boundary and padding
                        &body[previous_index + boundary.len() + 2..i - 2]
                    );

                    parse_form_data_part(&mut fields, part);

                }

                previous_index = i;

            }
        }

    // WwwFormUrlEncoded
    } else {
        for (name, value) in form_urlencoded::parse(body) {
            parse_form_field(&mut fields, name.to_string(), value.to_string());
        }
    };

    // Convert Array fields with a single value back to Value fields
    let fields = fields.into_iter().map(|field| {
        if let HttpFormDataField::Array(name, mut values) = field {
            if values.len() == 1 {
                HttpFormDataField::Value(name, values.remove(0))

            } else {
                HttpFormDataField::Array(name, values)
            }

        } else {
            field
        }

    }).collect::<Vec<HttpFormDataField>>();

    Ok(HttpFormData::new(fields))

}

fn form_fields_into_parts(fields: Vec<HttpFormDataField>) -> Vec<(Vec<u8>, Vec<u8>)> {

    // Convert form fields into multiparts
    let mut parts = Vec::new();
    for field in fields {
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

    parts

}

fn parse_form_field(fields: &mut Vec<HttpFormDataField>, name: String, value: String) {

    // If the field name matches the previous field, push the
    // value into the array instead of adding a new field
    if let Some(&mut HttpFormDataField::Array(ref field_name, ref mut values)) = fields.last_mut() {
        if field_name == &name {
            values.push(value);
            return;
        }
    }

    // Add a new field if the field names did not match
    fields.push(HttpFormDataField::Array(name, vec![value]));

}

fn parse_form_data_part(fields: &mut Vec<HttpFormDataField>, data: Vec<u8>) {

    // We let httparse do the gruntwork for us
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);

    match req.parse(&data[..]) {
        Ok(httparse::Status::Complete(size)) => {

            // Parse field metadata
            let mut part = ParsedFormDataField::from_headers(
                // TODO IW: Return error if headers are not parseable
                Headers::from_raw(req.headers).unwrap()
            );

            // File fields
            if part.filename.is_some() {
                fields.push(HttpFormDataField::FileVec(
                    part.name,
                    part.filename.take().unwrap(),
                    part.mime.take().unwrap(),
                    (&data[size..]).to_vec()
                ));

            // Text fields
            } else {
                parse_form_field(
                    fields,
                    part.name,
                    // TODO return utf-8 errors
                    String::from_utf8((&data[size..]).to_vec()).unwrap()
                )
            }

        },
        _ => unreachable!()
    }

}

#[derive(Debug)]
struct ParsedFormDataField {
    name: String,
    mime: Option<Mime>,
    filename: Option<String>
}

impl ParsedFormDataField {

    fn from_headers(headers: Headers) -> ParsedFormDataField {

        let mut name = String::new();
        let mut mime = None;
        let mut filename = None;

        if let Some(&ContentType(ref m)) = headers.get::<ContentType>() {
            mime = Some(m.clone());
        }

        // TODO IW: Return error if header is missing
        let disposition = headers.get::<ContentDisposition>().unwrap();
        for p in &disposition.parameters {
            match p {
                &DispositionParam::Ext(ref key, ref value) => {
                    if key == "name" {
                        name = value.clone();
                    }
                },
                &DispositionParam::Filename(_, _, ref buf) => {
                    filename = Some(String::from_utf8(buf.clone()).unwrap());
                }
            }
        }

        ParsedFormDataField {
            name: name,
            mime: mime,
            filename: filename,
        }

    }

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

