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


// Internal Dependencies ------------------------------------------------------
use util;
use resource::http::HttpFormDataField;


// Deep Form Compare ----------------------------------------------------------
pub fn compare(
    expected: &Vec<HttpFormDataField>,
    actual: &Vec<HttpFormDataField>,
    check_additional_keys: bool

) -> Result<(), Vec<(Vec<String>, String)>> {

    let errors = compare_form(
        expected,
        actual,
        check_additional_keys,
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
    a: &Vec<HttpFormDataField>,
    b: &Vec<HttpFormDataField>,
    check_additional_keys: bool,
    path: Vec<String>

) -> Vec<(Vec<String>, String)> {

    if a.is_empty() && b.is_empty() {
        return vec![];
    }

    let mut a_fields = map_form_fields(&a);
    let mut b_fields = map_form_fields(&b);

    a_fields.sort_by(|a, b| a.0.cmp(b.0));
    b_fields.sort_by(|a, b| a.0.cmp(b.0));

    let mut missing_fields = Vec::new();
    let mut errors = vec![];

    for (a_name, a_type, a_field) in a_fields {

        let mut found = false;
        b_fields.retain(|&(ref b_name, ref b_type, ref b_field)| {
            if a_name == *b_name {

                let mut field_path = path.clone();
                field_path.push(format!(".{}", a_name.blue().bold()));
                errors.append(&mut compare_form_fields(
                    a_type,
                    b_type.clone(),
                    a_field,
                    b_field,
                    field_path,
                    "Field"
                ));

                found = true;
                false

            } else {
                true
            }
        });

        if !found {
            missing_fields.push((a_name, a_type, a_field));
        }

    }

    missing_field_errors(FormFieldType::Field, &mut errors, &missing_fields);
    missing_field_errors(FormFieldType::Array, &mut errors, &missing_fields);
    missing_field_errors(FormFieldType::File, &mut errors, &missing_fields);

    if check_additional_keys {
        additional_field_errors(FormFieldType::Field, &mut errors, &b_fields);
        additional_field_errors(FormFieldType::Array, &mut errors, &b_fields);
        additional_field_errors(FormFieldType::File, &mut errors, &b_fields);
    }

    errors

}


// Form Field Comparision -----------------------------------------------------
fn compare_form_fields(
    type_a: FormFieldType,
    type_b: FormFieldType,
    a: &HttpFormDataField,
    b: &HttpFormDataField,
    path: Vec<String>,
    name: &'static str

) -> Vec<(Vec<String>, String)> {

    if type_a == type_b {

        let mut errors = vec![];
        match a {

            &HttpFormDataField::Field(_, ref value_a) => {
                if let &HttpFormDataField::Field(_, ref value_b) = b {
                    if value_a != value_b {
                        let (expected, actual, diff) = util::diff::text(value_b, value_a);
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
            },

            &HttpFormDataField::Array(_, ref value_a) => {
                if let &HttpFormDataField::Array(_, ref value_b) = b {

                    let len_a = value_a.len();
                    let len_b = value_b.len();

                    if len_a != len_b {
                        errors.push((
                            path.clone(),
                            format!(
                                "{} {} {} {} {}",
                                "Array".green().bold(),
                                "with".yellow(),
                                format!("{}", len_b).red().bold(),
                                "item(s) does not match expected length of".yellow(),
                                format!("{}", len_a).green().bold()
                            )
                        ));
                    }

                    for (index, (a_item, b_item)) in value_a.iter().zip(value_b).enumerate() {

                        let mut item_path = path.clone();
                        item_path.push(format!("[{}]", index).purple().bold().to_string());

                        let a = HttpFormDataField::Field("".to_string(), a_item.to_string());
                        let b = HttpFormDataField::Field("".to_string(), b_item.to_string());

                        errors.append(&mut compare_form_fields(
                            FormFieldType::Field,
                            FormFieldType::Field,
                            &a,
                            &b,
                            item_path,
                            "ArrayItem"
                        ));

                    }

                }
            },

            &HttpFormDataField::FileVec(_, ref filename_a, ref mime_a, ref data_a) => {
                if let &HttpFormDataField::FileVec(_, ref filename_b, ref mime_b, ref data_b) = b {

                    if filename_a != filename_b {
                        errors.push((
                            path.clone(),
                            format!(
                                "{} (\"{}\") {} (\"{}\")",
                                "Filename".green().bold(),
                                format!("{}", filename_b).red().bold(),
                                "does not match expected value".yellow(),
                                format!("{}", filename_a).green().bold()
                            )
                        ))
                    }

                    if mime_a != mime_b {
                        errors.push((
                            path.clone(),
                            format!(
                                "{} ({}) {} ({})",
                                "Mimetype".green().bold(),
                                format!("{}", mime_b).red().bold(),
                                "does not match expected value".yellow(),
                                format!("{}", mime_a).green().bold()
                            )
                        ))
                    }

                    // TODO IW: Compare file content based on mime type

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
                type_a.as_str().green().bold(),
                "but found a".yellow(),
                type_b.as_str().red().bold()
            )
        )]
    }

}


// Helpers --------------------------------------------------------------------
fn missing_field_errors(typ: FormFieldType, errors: &mut Vec<(Vec<String>, String)>, fields: &Vec<(&String, FormFieldType, &HttpFormDataField)>) {

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

fn additional_field_errors(typ: FormFieldType, errors: &mut Vec<(Vec<String>, String)>, fields: &Vec<(&String, FormFieldType, &HttpFormDataField)>) {

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

fn map_form_fields(fields: &Vec<HttpFormDataField>) -> Vec<(&String, FormFieldType, &HttpFormDataField)> {
    fields.iter().map(|field| {
        match field {
            &HttpFormDataField::Field(ref name, _) => {
                (name, FormFieldType::Field, field)
            },
            &HttpFormDataField::Array(ref name, _) => {
                (name, FormFieldType::Array, field)
            },
            &HttpFormDataField::FileVec(ref name, _, _, _) | &HttpFormDataField::FileFs(ref name, _, _, _) => {
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

    fn cmp(expected: Vec<HttpFormDataField>, actual: Vec<HttpFormDataField>, errors: Vec<(Vec<&str>, &str)>) {
        cmp_base(expected, actual, errors, false);
    }

    fn cmp_base(expected: Vec<HttpFormDataField>, actual: Vec<HttpFormDataField>, errors: Vec<(Vec<&str>, &str)>, add: bool) {
        match compare(&expected, &actual, add) {
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
            (vec![".<bb>field"], "<bg>Field <by>value does not match, expected:\n\n              \"<bg>other value\"\n\n          <by>but got:\n\n              \"<br>value\"\n\n          <by>difference:\n\n              \"<gbr>other value\"")
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
            (vec![".<bb>array[]", "<bp>[0]"], "<bg>ArrayItem <by>value does not match, expected:\n\n              \"<bg>other item\"\n\n          <by>but got:\n\n              \"<br>item\"\n\n          <by>difference:\n\n              \"<gbr>other item\"")
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
            (vec![".<bb>file"], "<bg>Mimetype (<br>text/html) <by>does not match expected value (<bg>text/plain)")
        ]);

    }

}

