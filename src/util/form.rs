// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::str;


// External Dependencies ------------------------------------------------------
use colored::*;
use hyper::mime::Mime;


// Internal Dependencies ------------------------------------------------------
use util;
use Options;
use resource::http::HttpFormDataField;
use resource::http::util::validate_http_multipart_body;


// Deep Form Compare ----------------------------------------------------------
pub fn compare(
    expected: &[HttpFormDataField],
    actual: &[HttpFormDataField],
    check_additional_keys: bool,
    options: &Options

) -> Result<(), Vec<(Vec<String>, String)>> {

    let errors = compare_form(
        expected,
        actual,
        check_additional_keys,
        options,
        vec![]
    );

    if errors.is_empty() {
        Ok(())

    } else {
        Err(errors)
    }

}

pub fn format(errors: Vec<(Vec<String>, String)>) -> String {
    errors.into_iter().map(|(path, message)| {
        format!("- {}{}: {}", "form".blue().bold(), path.join(""), message)

    }).collect::<Vec<String>>().join("\n\n        ")
}


// Recursive Compare Function -------------------------------------------------
fn compare_form(
    expected: &[HttpFormDataField],
    actual: &[HttpFormDataField],
    check_additional_keys: bool,
    options: &Options,
    path: Vec<String>

) -> Vec<(Vec<String>, String)> {

    if expected.is_empty() && actual.is_empty() {
        return vec![];
    }

    let mut expected_fields = map_form_fields(expected);
    let mut actual_fields = map_form_fields(actual);

    expected_fields.sort_by(|a, b| a.0.cmp(b.0));
    actual_fields.sort_by(|a, b| a.0.cmp(b.0));

    let mut missing_fields = Vec::new();
    let mut errors = vec![];

    for (expected_name, expected_type, expected_field) in expected_fields {

        let mut found = false;
        actual_fields.retain(|&(actual_name, actual_type, actual_field)| {
            if expected_name == actual_name {

                let mut field_path = path.clone();
                field_path.push(format!(".{}", expected_name.blue().bold()));
                errors.append(&mut compare_form_field(
                    field_path,
                    "Field",
                    check_additional_keys,
                    options,
                    expected_type,
                    actual_type,
                    expected_field,
                    actual_field
                ));

                found = true;
                false

            } else {
                true
            }
        });

        if !found {
            missing_fields.push((expected_name, expected_type, expected_field));
        }

    }

    missing_field_errors(FormFieldType::Field, &mut errors, &missing_fields);
    missing_field_errors(FormFieldType::Array, &mut errors, &missing_fields);
    missing_field_errors(FormFieldType::File, &mut errors, &missing_fields);

    if check_additional_keys {
        additional_field_errors(FormFieldType::Field, &mut errors, &actual_fields);
        additional_field_errors(FormFieldType::Array, &mut errors, &actual_fields);
        additional_field_errors(FormFieldType::File, &mut errors, &actual_fields);
    }

    errors

}


// Form Field Comparision -----------------------------------------------------
fn compare_form_field(
    path: Vec<String>,
    name: &'static str,
    check_additional_keys: bool,
    options: &Options,
    expected_type: FormFieldType,
    actual_type: FormFieldType,
    expected: &HttpFormDataField,
    actual: &HttpFormDataField

) -> Vec<(Vec<String>, String)> {

    if expected_type == actual_type {

        let mut errors = vec![];
        match *expected {
            HttpFormDataField::Field(_, ref expected_value) =>  {
                if let HttpFormDataField::Field(_, ref actual_value) = *actual {
                    compare_field(
                        &mut errors, path, name, expected_value, actual_value
                    );
                }
            },
            HttpFormDataField::Array(_, ref expected_values) => {
                if let HttpFormDataField::Array(_, ref actual_values) = *actual {
                    compare_array(
                        &mut errors,
                        path,
                        check_additional_keys,
                        options,
                        expected_values,
                        actual_values
                    );
                }
            },
            HttpFormDataField::FileVec(
                _,
                ref expected_filename,
                ref expected_mime,
                ref expected_body

            ) => {
                if let HttpFormDataField::FileVec(
                    _,
                    ref actual_filename,
                    ref actual_mime,
                    ref actual_body

                ) = *actual {
                    compare_file_name(
                        &mut errors,
                        path.clone(),
                        expected_filename,
                        actual_filename
                    );
                    compare_file_mime(
                        &mut errors,
                        path.clone(),
                        expected_mime,
                        actual_mime
                    );
                    compare_file_body(
                        &mut errors,
                        path.clone(),
                        check_additional_keys,
                        options,
                        expected_body,
                        expected_mime,
                        actual_body,
                        actual_mime
                    );
                }
            },
            _ => unreachable!()
        }

        errors

    } else {
        vec![(
            path,
            format!(
                "{} {} {} {}",
                "Expected a".yellow(),
                expected_type.as_str().green().bold(),
                "but found a".yellow(),
                actual_type.as_str().red().bold()
            )
        )]
    }

}

fn compare_field(
    errors: &mut Vec<(Vec<String>, String)>,
    path: Vec<String>,
    name: &'static str,
    expected: &str,
    actual: &str
) {
    if actual != expected {
        let (expected, actual, diff) = util::diff::text(expected, actual);
        errors.push((
            path.clone(),
            format!(
                "{} {}\n\n              \"{}\"\n\n          {}\n\n              \"{}\"\n\n          {}\n\n              \"{}\"",
                name.green().bold(),
                "value does not match, expected:".yellow(),
                expected.green().bold(),
                "but got:".yellow(),
                actual.red().bold(),
                "difference:".yellow(),
                diff
            )
        ))
    }
}

fn compare_array(
    errors: &mut Vec<(Vec<String>, String)>,
    path: Vec<String>,
    check_additional_keys: bool,
    options: &Options,
    expected: &[String],
    actual: &[String]
) {

    let expected_len = expected.len();
    let actual_len = actual.len();

    if expected_len != actual_len {
        errors.push((
            path.clone(),
            format!(
                "{} {} {} {} {}",
                "Array".green().bold(),
                "with".yellow(),
                format!("{}", actual_len).red().bold(),
                "item(s) does not match expected length of".yellow(),
                format!("{}", expected_len).green().bold()
            )
        ));
    }

    for (index, (a_item, b_item)) in expected.iter().zip(actual).enumerate() {

        let mut item_path = path.clone();
        item_path.push(format!("[{}]", index).purple().bold().to_string());

        let expected = HttpFormDataField::Field("".to_string(), a_item.to_string());
        let actual = HttpFormDataField::Field("".to_string(), b_item.to_string());

        errors.append(&mut compare_form_field(
            item_path,
            "ArrayItem",
            check_additional_keys,
            options,
            FormFieldType::Field,
            FormFieldType::Field,
            &expected,
            &actual
        ));

    }

}

fn compare_file_name(
    errors: &mut Vec<(Vec<String>, String)>,
    path: Vec<String>,
    expected: &str,
    actual: &str
) {
    if expected != actual {
        errors.push((
            path.clone(),
            format!(
                "{} (\"{}\") {} (\"{}\")",
                "Filename".green().bold(),
                actual.red().bold(),
                "does not match expected value".yellow(),
                expected.green().bold()
            )
        ))
    }
}

fn compare_file_mime(
    errors: &mut Vec<(Vec<String>, String)>,
    path: Vec<String>,
    expected: &Mime,
    actual: &Mime
) {
    if expected != actual {
        errors.push((
            path.clone(),
            format!(
                "{} ({}) {} ({})",
                "MIME type".green().bold(),
                format!("{}", actual).red().bold(),
                "does not match expected value".yellow(),
                format!("{}", expected).green().bold()
            )
        ))
    }
}

fn compare_file_body(
    errors: &mut Vec<(Vec<String>, String)>,
    path: Vec<String>,
    check_additional_keys: bool,
    options: &Options,
    expected_body: &[u8],
    expected_mime: &Mime,
    actual_body: &[u8],
    actual_mime: &Mime
) {

    for error in validate_http_multipart_body(
        expected_body,
        expected_mime,
        actual_body,
        actual_mime,
        check_additional_keys,
        options,
    ) {
        let error = error.split('\n').enumerate().map(|(i, line)| {
            if i == 0 {
                line.to_string()

            } else {
                format!("      {}", line)
            }

        }).collect::<Vec<String>>().join("\n") ;

        errors.push((
            path.clone(),
            format!("{}{}", "File".yellow(), error)
        ))
    }

}


// Helpers --------------------------------------------------------------------
fn missing_field_errors(
    typ: FormFieldType,
    errors: &mut Vec<(Vec<String>, String)>,
    fields: &[(&String, FormFieldType, &HttpFormDataField)]
) {

    let fields = fields.iter().filter(|v| v.1 == typ).collect::<Vec<&(&String, FormFieldType, &HttpFormDataField)>>();
    if !fields.is_empty() {
        errors.push((
            vec![],
            format!(
                "{} {} {} ({})",
                "Is missing".yellow(),
                format!("{}", fields.len()).red().bold(),
                format!("{}(s)", typ.as_str()).yellow(),
                fields.iter().map(|e| {
                    e.0.as_str().red().bold().to_string()

                }).collect::<Vec<String>>().join(", ")
            )
        ));
    }

}

fn additional_field_errors(
    typ: FormFieldType,
    errors: &mut Vec<(Vec<String>, String)>,
    fields: &[(&String, FormFieldType, &HttpFormDataField)]
) {

    let fields = fields.iter().filter(|v| v.1 == typ).collect::<Vec<&(&String, FormFieldType, &HttpFormDataField)>>();
    if !fields.is_empty() {
        errors.push((
            vec![],
            format!(
                "{} {} {} ({})",
                "Has".yellow(),
                format!("{}", fields.len()).red().bold(),
                format!("additional unexpected {}(s)", typ.as_str()).yellow(),
                fields.iter().map(|e| {
                    e.0.as_str().red().bold().to_string()

                }).collect::<Vec<String>>().join(", ")
            )
        ));
    }

}

fn map_form_fields(
    fields: &[HttpFormDataField]

) -> Vec<(&String, FormFieldType, &HttpFormDataField)> {
    fields.iter().map(|field| {
        match *field {
            HttpFormDataField::Field(ref name, _) => {
                (name, FormFieldType::Field, field)
            },
            HttpFormDataField::Array(ref name, _) => {
                (name, FormFieldType::Array, field)
            },
            HttpFormDataField::FileVec(ref name, _, _, _) |
            HttpFormDataField::FileFs(ref name, _, _, _) => {
                (name, FormFieldType::File, field)
            }
        }

    }).collect()
}

#[derive(Copy, Clone, PartialEq)]
enum FormFieldType {
    Field,
    Array,
    File
}

impl FormFieldType {
    fn as_str(&self) -> &'static str {
        match *self {
            FormFieldType::Field => "plain field",
            FormFieldType::Array => "array",
            FormFieldType::File => "file attachment"
        }
    }
}


// Tests ----------------------------------------------------------------------
#[cfg(test)]
mod tests {

    use hyper::mime::{Mime, TopLevel, SubLevel};
    use resource::http::HttpFormDataField;
    use super::compare;
    use Options;

    macro_rules! form {
        {} => (vec![]);

        { $( $key:expr => $value:expr ),* } => ({

            let mut fields = Vec::new();

            $(
                fields.push(($key, $value).into());
            )*

            fields
        })
    }

    fn uncolor(mut text: String) -> String {
        text = text.replace("\u{1b}", "");
        text = text.replace("[1;31m", "<br>");
        text = text.replace("[1;32m", "<bg>");
        text = text.replace("[33m", "<by>");
        text = text.replace("[1;34m", "<bb>");
        text = text.replace("[1;35m", "<bp>");
        text = text.replace("[36m", "<bn>");
        text = text.replace("[1;36m", "<bc>");
        text = text.replace("[1;42;37m", "<gbg>");
        text = text.replace("[1;41;37m", "<gbr>");
        text.replace("[0m", "")
    }

    fn cmp(
        expected: Vec<HttpFormDataField>,
        actual: Vec<HttpFormDataField>,
        errors: Vec<(Vec<&str>, &str)>
    ) {
        cmp_base(expected, actual, errors, false);
    }

    fn cmp_base(
        expected: Vec<HttpFormDataField>,
        actual: Vec<HttpFormDataField>,
        errors: Vec<(Vec<&str>, &str)>,
        add: bool
    ) {
        let options: Options = Default::default();
        match compare(&expected, &actual, add, &options) {
            Ok(()) => {
                assert!(errors.is_empty());
            },
            Err(e) => {
                let formatted = e.into_iter().map(|e| {
                    (
                        e.0.into_iter().map(|p| uncolor(p)).collect::<Vec<String>>(),
                        uncolor(e.1)
                    )

                }).collect::<Vec<(Vec<String>, String)>>();

                let err = formatted.iter().map(|e| {
                    (
                        e.0.iter().map(|p| p.as_str()).collect::<Vec<&str>>(),
                        e.1.as_str()
                    )

                }).collect::<Vec<(Vec<&str>, &str)>>();

                assert_eq!(err, errors);

            }
        }
    }

    #[test]
    fn test_compare_empty() {
        cmp(form!{}, form!{}, vec![]);
    }

    #[test]
    fn test_compare_fields() {

        cmp(form!{ "field" => "value" }, form!{ "field" => "value" }, vec![
        ]);

        cmp(form!{ "field" => "value" }, form!{ "field" => "other value" }, vec![
            (vec![".<bb>field"], "<bg>Field <by>value does not match, expected:\n\n              \"<bg>value\"\n\n          <by>but got:\n\n              \"<br>other value\"\n\n          <by>difference:\n\n              \"<gbg>other value\"")
        ]);

        cmp(form!{ "field" => "value" }, form!{ "field" => vec!["1"] }, vec![
            (vec![".<bb>field"], "<by>Expected a <bg>plain field <by>but found a <br>array")
        ]);

        cmp(form!{ "field" => "value" }, form!{ "field" => (
            "filename",
            Mime(TopLevel::Text, SubLevel::Plain, vec![]),
            "Data"

        )}, vec![
            (vec![".<bb>field"], "<by>Expected a <bg>plain field <by>but found a <br>file attachment")
        ]);

    }

    #[test]
    fn test_compare_arrays() {

        cmp(form!{ "array[]" => vec!["item"] }, form!{ "array[]" => vec!["item"] }, vec![
        ]);

        cmp(form!{ "array[]" => vec!["item"] }, form!{ "array[]" => "value" }, vec![
            (vec![".<bb>array[]"], "<by>Expected a <bg>array <by>but found a <br>plain field")
        ]);

        cmp(form!{ "array[]" => vec!["item"] }, form!{ "array[]" => (
            "filename",
            Mime(TopLevel::Text, SubLevel::Plain, vec![]),
            "Data"

        )}, vec![
            (vec![".<bb>array[]"], "<by>Expected a <bg>array <by>but found a <br>file attachment")
        ]);

        cmp(form!{ "array[]" => vec!["item"] }, form!{ "array[]" => vec!["item", "item"] }, vec![
            (vec![".<bb>array[]"], "<bg>Array <by>with <br>2 <by>item(s) does not match expected length of <bg>1")
        ]);

        cmp(form!{ "array[]" => vec!["item", "item"] }, form!{ "array[]" => vec!["item"] }, vec![
            (vec![".<bb>array[]"], "<bg>Array <by>with <br>1 <by>item(s) does not match expected length of <bg>2")
        ]);

        cmp(form!{ "array[]" => vec!["item"] }, form!{ "array[]" => vec!["other item"] }, vec![
            (vec![".<bb>array[]", "<bp>[0]"], "<bg>ArrayItem <by>value does not match, expected:\n\n              \"<bg>item\"\n\n          <by>but got:\n\n              \"<br>other item\"\n\n          <by>difference:\n\n              \"<gbg>other item\"")
        ]);

    }

    #[test]
    fn test_compare_missing() {
        cmp(form!{ "field" => "value", "field2" => "value" }, form!{  }, vec![
            (vec![], "<by>Is missing <br>2 <by>plain field(s) (<br>field, <br>field2)")
        ]);
    }

    #[test]
    fn test_compare_missing_ignore_additional() {
        cmp(form!{}, form!{ "otherField" => "value", "otherField2" => "value"}, vec![
        ]);
    }

    #[test]
    fn test_compare_additional_check_additional() {
        cmp_base(form!{  }, form!{ "otherField" => "value", "otherField2" => "value"}, vec![
            (vec![], "<by>Has <br>2 <by>additional unexpected plain field(s) (<br>otherField, <br>otherField2)")

        ], true);
    }

    #[test]
    fn test_compare_files() {

        cmp(form!{
            "file" => (
                "filename",
                Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                "Data"
            )

        }, form!{
            "file" => (
                "filename",
                Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                "Data"
            )

        }, vec![]);

        cmp(form!{
            "file" => (
                "filename",
                Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                "Data"
            )

        }, form!{
            "file" => (
                "otherFilename",
                Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                "Data"
            )

        }, vec![
            (vec![".<bb>file"], "<bg>Filename (\"<br>otherFilename\") <by>does not match expected value (\"<bg>filename\")")
        ]);

        cmp(form!{
            "file" => (
                "filename",
                Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                "Data"
            )

        }, form!{
            "file" => (
                "filename",
                Mime(TopLevel::Text, SubLevel::Html, vec![]),
                "Data"
            )

        }, vec![
            (vec![".<bb>file"], "<bg>MIME type (<br>text/html) <by>does not match expected value (<bg>text/plain)")
        ]);

        cmp(form!{
            "file" => (
                "filename",
                Mime(TopLevel::Application, SubLevel::OctetStream, vec![]),
                vec![0, 1, 2, 3, 4]
            )

        }, form!{
            "file" => (
                "filename",
                Mime(TopLevel::Application, SubLevel::OctetStream, vec![]),
                vec![6, 4, 3, 2, 1, 0]
            )

        }, vec![
            (vec![".<bb>file"], "<by>File<by> <by>raw body data does not match, expected the following <bg>5 bytes<by>:\n      \n             [<bg>0x00, <bg>0x01, <bg>0x02, <bg>0x03, <bg>0x04]\n      \n          <by>but got the following <br>6 bytes <by>instead:\n      \n             [<br>0x06, <br>0x04, <br>0x03, <br>0x02, <br>0x01, <br>0x00]")
        ]);

        cmp(form!{
            "file" => (
                "filename",
                Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                "Data"
            )

        }, form!{
            "file" => (
                "filename",
                Mime(TopLevel::Text, SubLevel::Plain, vec![]),
                "Other Data"
            )

        }, vec![
            (vec![".<bb>file"], "<by>File<by> <by>text body does not match, expected:\n      \n              \"<bg>Data\"\n      \n          <by>but got:\n      \n              \"<br>Other Data\"\n      \n          <by>difference:\n      \n              \"<gbg>Other Data\"")
        ]);

        cmp(form!{
            "file" => (
                "filename",
                Mime(TopLevel::Application, SubLevel::Json, vec![]),
                object! {
                    "key" => "value"
                }
            )

        }, form!{
            "file" => (
                "filename",
                Mime(TopLevel::Application, SubLevel::Json, vec![]),
                object! {
                    "key" => "otherValue"
                }
            )

        }, vec![
            (vec![".<bb>file"], "<by>File<by> <by>body JSON does not match:\n      \n              - <bb>json.<bb>key: <bg>String <by>does not match, expected:\n      \n                    \"<bg>otherValue\"\n      \n                <by>but got:\n      \n                    \"<br>value\"\n      \n                <by>difference:\n      \n                    \"<gbr>otherValue <gbg>value\"")
        ]);

    }

}

