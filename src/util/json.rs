// Copyright (c) 2016 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use colored::*;
use json::JsonValue;


// Deep JSON Compare ----------------------------------------------------------
pub fn compare(
    expected: &JsonValue,
    actual: &JsonValue,
    max_depth: usize,
    check_additional_keys: bool

) -> Result<(), Vec<(Vec<String>, String)>> {

    let errors = compare_json(
        1,
        max_depth,
        check_additional_keys,
        vec![],
        expected,
        actual
    );

    if errors.is_empty() {
        Ok(())

    } else {
        Err(errors)
    }

}

pub fn format(errors: Vec<(Vec<String>, String)>) -> String {
    errors.into_iter().map(|(path, message)| {
        format!("- {}{}: {}", "json".blue().bold(), path.join(""), message)

    }).collect::<Vec<String>>().join("\n\n        ")
}


// Recursive Compare Function -------------------------------------------------
fn compare_json(
    depth: usize,
    max_depth: usize,
    check_additional_keys: bool,
    path: Vec<String>,
    a: &JsonValue,
    b: &JsonValue

) -> Vec<(Vec<String>, String)> {

    let type_a = json_type(a);
    let type_b = json_type(b);

    if depth > max_depth {
        vec![]

    } else if a.is_object() && b.is_object() {
        compare_object(depth, max_depth, check_additional_keys, path, a, b)

    } else if a.is_array() && b.is_array() {
        compare_array(depth, max_depth, check_additional_keys, path, a, b)

    } else if type_a == type_b {
        compare_type(path, a, b)

    } else {
        type_mismatch(path, type_a, type_b, b)
    }

}

fn compare_object(
    depth: usize,
    max_depth: usize,
    check_additional_keys: bool,
    path: Vec<String>,
    a: &JsonValue,
    b: &JsonValue

) -> Vec<(Vec<String>, String)> {

    if a.is_empty() && b.is_empty() {
        return vec![];
    }

    let mut a_entries = a.entries().collect::<Vec<(&String, &JsonValue)>>();
    let mut b_entries = b.entries().collect::<Vec<(&String, &JsonValue)>>();

    a_entries.sort_by(|a, b| a.0.cmp(b.0));
    b_entries.sort_by(|a, b| a.0.cmp(b.0));

    let mut missing_entries = Vec::new();
    let mut errors = vec![];

    for (a_key, a_entry) in a_entries {

        let mut found = false;
        b_entries.retain(|&(b_key, b_entry)| {
            if a_key == b_key {

                found = true;

                let mut entry_path = path.clone();
                entry_path.push(format!(".{}", a_key.blue().bold()));
                errors.append(&mut compare_json(
                    depth + 1,
                    max_depth,
                    check_additional_keys,
                    entry_path,
                    a_entry,
                    b_entry
                ));

                false

            } else {
                true
            }
        });

        if !found {
            missing_entries.push((a_key, a_entry));
        }

    }

    if !missing_entries.is_empty() {
        errors.push((
            path.clone(),
            format!(
                "{} {} {} {} ({})",
                "Object".green().bold(),
                "is missing".yellow(),
                format!("{}", missing_entries.len()).red().bold(),
                "key(s)".yellow(),
                missing_entries.iter().map(|e| {
                    e.0.as_str().red().bold().to_string()

                }).collect::<Vec<String>>().join(", ")
            )
        ));
    }

    if check_additional_keys && !b_entries.is_empty() {
        errors.push((
            path.clone(),
            format!(
                "{} {} {} {} ({})",
                "Object".green().bold(),
                "has".yellow(),
                format!("{}", b_entries.len()).red().bold(),
                "additional unexpected key(s)".yellow(),
                b_entries.iter().map(|e| {
                    e.0.as_str().red().bold().to_string()

                }).collect::<Vec<String>>().join(", ")
            )
        ));
    }

    errors

}

fn compare_array(
    depth: usize,
    max_depth: usize,
    check_additional_keys: bool,
    path: Vec<String>,
    a: &JsonValue,
    b: &JsonValue

) -> Vec<(Vec<String>, String)> {

    if a.is_empty() && b.is_empty() {
        return vec![];
    }

    let len_a = a.len();
    let len_b = b.len();

    let mut errors = vec![];
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

        if depth >= max_depth {
            return errors;
        }

    }

    for (index, (a_item, b_item)) in a.members().zip(b.members()).enumerate() {
        let mut item_path = path.clone();
        item_path.push(format!("[{}]", index).purple().bold().to_string());
        errors.append(&mut compare_json(
            depth + 1,
            max_depth,
            check_additional_keys,
            item_path,
            a_item,
            b_item
        ));
    }

    errors

}


fn compare_type(
    path: Vec<String>,
    a: &JsonValue,
    b: &JsonValue

) -> Vec<(Vec<String>, String)> {

    match *a {
        JsonValue::String(ref value_a) => {
            let value_b = b.as_str().unwrap();
            if value_a == value_b {
                vec![]

            } else {
                vec![(
                    path,
                    format!(
                        // TODO escape \n etc.
                        // TODO use text diff?
                        "{} (\"{}\") {} (\"{}\")",
                        "String".green().bold(),
                        value_b.red().bold(),
                        "does not match expected value".yellow(),
                        value_a.green().bold()
                    )
                )]
            }
        },
        JsonValue::Number(value_a) => {
            let value_b = b.as_f64().unwrap();
            if (value_a - value_b).abs() < 0.00000001 {
                vec![]

            } else {
                vec![(
                    path,
                    format!(
                        "{} ({}) {} ({})",
                        "Number".green().bold(),
                        format!("{}", value_b).red().bold(),
                        "does not match expected value".yellow(),
                        format!("{}", value_a).green().bold()
                    )
                )]
            }
        },
        JsonValue::Boolean(value_a) => {
            let value_b = b.as_bool().unwrap();
            if value_a == value_b {
                vec![]

            } else {
                vec![(
                    path,
                    format!(
                        "{} ({}) {} ({})",
                        "Boolean".green().bold(),
                        format!("{}", value_b).red().bold(),
                        "does not match expected value".yellow(),
                        format!("{}", value_a).green().bold()
                    )
                )]
            }
        },
        JsonValue::Null => {
            vec![]
        },
        _ => unreachable!()
    }

}

fn type_mismatch(
    path: Vec<String>,
    type_a: JsonType,
    type_b: JsonType,
    b: &JsonValue

) -> Vec<(Vec<String>, String)> {

    if b.is_string() {
        vec![(
            path,
            format!(
                "{} {} {} {} (\"{}\")",
                "Expected a".yellow(),
                type_a.as_str().green().bold(),
                "but found a".yellow(),
                "String".red().bold(),
                b.as_str().unwrap().red().bold()
            )
        )]

    } else if b.is_number() {
        vec![(
            path,
            format!(
                "{} {} {} {} ({})",
                "Expected a".yellow(),
                type_a.as_str().green().bold(),
                "but found a".yellow(),
                "Number".red().bold(),
                format!("{}", b.as_f64().unwrap()).red().bold()
            )
        )]

    } else if b.is_boolean() {
        vec![(
            path,
            format!(
                "{} {} {} {} ({})",
                "Expected a".yellow(),
                type_a.as_str().green().bold(),
                "but found a".yellow(),
                "Boolean".red().bold(),
                format!("{}", b.as_bool().unwrap()).red().bold()
            )
        )]

    } else if b.is_object() {
        vec![(
            path,
            format!(
                "{} {} {} {} {} {} {}",
                "Expected a".yellow(),
                type_a.as_str().green().bold(),
                "but found a".yellow(),
                "Object".red().bold(),
                "with".yellow(),
                format!("{}", b.len()).red().bold(),
                "key(s)".yellow()
            )
        )]

    } else if b.is_array() {
        vec![(
            path,
            format!(
                "{} {} {} {} {} {} {}",
                "Expected a".yellow(),
                type_a.as_str().green().bold(),
                "but found a".yellow(),
                "Array".red().bold(),
                "with".yellow(),
                format!("{}", b.len()).red().bold(),
                "item(s)".yellow()
            )
        )]

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
#[derive(PartialEq)]
enum JsonType {
    String,
    Number,
    Boolean,
    Null,
    Object,
    Array
}

impl JsonType {
    fn as_str(&self) -> &'static str {
        match *self {
            JsonType::String => "String",
            JsonType::Number => "Number",
            JsonType::Boolean => "Boolean",
            JsonType::Null => "Null",
            JsonType::Object => "Object",
            JsonType::Array => "Array"
        }
    }
}

fn json_type(value: &JsonValue) -> JsonType {
    match *value {
        JsonValue::String(_) => JsonType::String,
        JsonValue::Number(_) => JsonType::Number,
        JsonValue::Boolean(_) => JsonType::Boolean,
        JsonValue::Null => JsonType::Null,
        JsonValue::Object(_) => JsonType::Object,
        JsonValue::Array(_) => JsonType::Array
    }
}


// Tests ----------------------------------------------------------------------
#[cfg(test)]
mod tests {

    use super::compare;
    use json::JsonValue;

    fn uncolor(mut text: String) -> String {
        text = text.replace("\u{1b}", "");
        text = text.replace("[1;31m", "<br>");
        text = text.replace("[1;32m", "<bg>");
        text = text.replace("[33m", "<by>");
        text = text.replace("[1;34m", "<bb>");
        text = text.replace("[1;35m", "<bp>");
        text = text.replace("[36m", "<bn>");
        text = text.replace("[1;36m", "<bc>");
        text.replace("[0m", "")
    }

    fn cmp(expected: JsonValue, actual: JsonValue, errors: Vec<(Vec<&str>, &str)>) {
        cmp_base(expected, actual, errors, 1, false);
    }

    fn cmp_base(expected: JsonValue, actual: JsonValue, errors: Vec<(Vec<&str>, &str)>, depth: usize, add: bool) {
        match compare(&expected, &actual, depth, add) {
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
    fn test_compare_string() {
        cmp(JsonValue::String("Foo".to_string()), JsonValue::String("Foo".to_string()), vec![]);
        cmp(JsonValue::String("Foo".to_string()), JsonValue::String("Bar".to_string()), vec![
            (vec![], "<bg>String (\"<br>Bar\") <by>does not match expected value (\"<bg>Foo\")")
        ]);
        cmp(JsonValue::String("Foo".to_string()), JsonValue::Number(2.0), vec![
            (vec![], "<by>Expected a <bg>String <by>but found a <br>Number (<br>2)")
        ]);
        cmp(JsonValue::String("Foo".to_string()), JsonValue::Boolean(false), vec![
            (vec![], "<by>Expected a <bg>String <by>but found a <br>Boolean (<br>false)")
        ]);
        cmp(JsonValue::String("Foo".to_string()), JsonValue::Null, vec![
            (vec![], "<by>Expected a <bg>String <by>but found a <br>Null")
        ]);
        cmp(JsonValue::String("Foo".to_string()), object!{}, vec![
            (vec![], "<by>Expected a <bg>String <by>but found a <br>Object <by>with <br>0 <by>key(s)")
        ]);
        cmp(JsonValue::String("Foo".to_string()), array![], vec![
            (vec![], "<by>Expected a <bg>String <by>but found a <br>Array <by>with <br>0 <by>item(s)")
        ]);
    }

    #[test]
    fn test_compare_number() {
        cmp(JsonValue::Number(42.8), JsonValue::Number(42.8), vec![]);
        cmp(JsonValue::Number(42.8), JsonValue::String("Bar".to_string()), vec![
            (vec![], "<by>Expected a <bg>Number <by>but found a <br>String (\"<br>Bar\")")
        ]);
        cmp(JsonValue::Number(42.8), JsonValue::Number(3.14), vec![
            (vec![], "<bg>Number (<br>3.14) <by>does not match expected value (<bg>42.8)")
        ]);
        cmp(JsonValue::Number(42.8), JsonValue::Boolean(false), vec![
            (vec![], "<by>Expected a <bg>Number <by>but found a <br>Boolean (<br>false)")
        ]);
        cmp(JsonValue::Number(42.8), JsonValue::Null, vec![
            (vec![], "<by>Expected a <bg>Number <by>but found a <br>Null")
        ]);
        cmp(JsonValue::Number(42.8), object!{}, vec![
            (vec![], "<by>Expected a <bg>Number <by>but found a <br>Object <by>with <br>0 <by>key(s)")
        ]);
        cmp(JsonValue::Number(42.8), array![], vec![
            (vec![], "<by>Expected a <bg>Number <by>but found a <br>Array <by>with <br>0 <by>item(s)")
        ]);
    }

    #[test]
    fn test_compare_boolean() {
        cmp(JsonValue::Boolean(true), JsonValue::Boolean(true), vec![]);
        cmp(JsonValue::Boolean(true), JsonValue::String("Bar".to_string()), vec![
            (vec![], "<by>Expected a <bg>Boolean <by>but found a <br>String (\"<br>Bar\")")
        ]);
        cmp(JsonValue::Boolean(true), JsonValue::Number(3.14), vec![
            (vec![], "<by>Expected a <bg>Boolean <by>but found a <br>Number (<br>3.14)")
        ]);
        cmp(JsonValue::Boolean(true), JsonValue::Boolean(false), vec![
            (vec![], "<bg>Boolean (<br>false) <by>does not match expected value (<bg>true)")
        ]);
        cmp(JsonValue::Boolean(true), JsonValue::Null, vec![
            (vec![], "<by>Expected a <bg>Boolean <by>but found a <br>Null")
        ]);
        cmp(JsonValue::Boolean(true), object!{}, vec![
            (vec![], "<by>Expected a <bg>Boolean <by>but found a <br>Object <by>with <br>0 <by>key(s)")
        ]);
        cmp(JsonValue::Boolean(true), array![], vec![
            (vec![], "<by>Expected a <bg>Boolean <by>but found a <br>Array <by>with <br>0 <by>item(s)")
        ]);
    }

    #[test]
    fn test_compare_null() {
        cmp(JsonValue::Null, JsonValue::Null, vec![]);
        cmp(JsonValue::Null, JsonValue::String("Bar".to_string()), vec![
            (vec![], "<by>Expected a <bg>Null <by>but found a <br>String (\"<br>Bar\")")
        ]);
        cmp(JsonValue::Null, JsonValue::Number(3.14), vec![
            (vec![], "<by>Expected a <bg>Null <by>but found a <br>Number (<br>3.14)")
        ]);
        cmp(JsonValue::Null, JsonValue::Boolean(false), vec![
            (vec![], "<by>Expected a <bg>Null <by>but found a <br>Boolean (<br>false)")
        ]);
        cmp(JsonValue::Null, object!{}, vec![
            (vec![], "<by>Expected a <bg>Null <by>but found a <br>Object <by>with <br>0 <by>key(s)")
        ]);
        cmp(JsonValue::Null, array![], vec![
            (vec![], "<by>Expected a <bg>Null <by>but found a <br>Array <by>with <br>0 <by>item(s)")
        ]);
    }

    #[test]
    fn test_compare_object() {
        cmp(object!{}, object!{}, vec![]);
        cmp(object!{}, JsonValue::String("Bar".to_string()), vec![
            (vec![], "<by>Expected a <bg>Object <by>but found a <br>String (\"<br>Bar\")")
        ]);
        cmp(object!{}, JsonValue::Number(3.14), vec![
            (vec![], "<by>Expected a <bg>Object <by>but found a <br>Number (<br>3.14)")
        ]);
        cmp(object!{}, JsonValue::Boolean(false), vec![
            (vec![], "<by>Expected a <bg>Object <by>but found a <br>Boolean (<br>false)")
        ]);
        cmp(object!{}, JsonValue::Null, vec![
            (vec![], "<by>Expected a <bg>Object <by>but found a <br>Null")
        ]);
        cmp(object!{}, array![], vec![
            (vec![], "<by>Expected a <bg>Object <by>but found a <br>Array <by>with <br>0 <by>item(s)")
        ]);
    }

    #[test]
    fn test_compare_array() {
        cmp(array![], array![], vec![]);
        cmp(array![], JsonValue::String("Bar".to_string()), vec![
            (vec![], "<by>Expected a <bg>Array <by>but found a <br>String (\"<br>Bar\")")
        ]);
        cmp(array![], JsonValue::Number(3.14), vec![
            (vec![], "<by>Expected a <bg>Array <by>but found a <br>Number (<br>3.14)")
        ]);
        cmp(array![], JsonValue::Boolean(false), vec![
            (vec![], "<by>Expected a <bg>Array <by>but found a <br>Boolean (<br>false)")
        ]);
        cmp(array![], JsonValue::Null, vec![
            (vec![], "<by>Expected a <bg>Array <by>but found a <br>Null")
        ]);
        cmp(array![], object!{}, vec![
            (vec![], "<by>Expected a <bg>Array <by>but found a <br>Object <by>with <br>0 <by>key(s)")
        ]);
    }

    #[test]
    fn test_compare_array_length() {
        cmp(array![], JsonValue::Array(vec![JsonValue::Number(2.0)]), vec![
            (vec![], "<bg>Array <by>with <br>1 <by>item(s) does not match expected length of <bg>0")
        ]);
        cmp(JsonValue::Array(vec![JsonValue::Number(2.0)]), array![], vec![
            (vec![], "<bg>Array <by>with <br>0 <by>item(s) does not match expected length of <bg>1")
        ]);
    }

    #[test]
    fn test_compare_objects_keys_missing() {
        cmp(object!{ "key" => "value"}, object!{}, vec![
            (vec![], "<bg>Object <by>is missing <br>1 <by>key(s) (<br>key)")
        ]);

        cmp(object!{ "key" => "value", "other" => "data" }, object!{}, vec![
            (vec![], "<bg>Object <by>is missing <br>2 <by>key(s) (<br>key, <br>other)")
        ]);
    }

    #[test]
    fn test_compare_objects_keys_ignore_additional() {
        cmp(object!{}, object!{ "key" => "value" }, vec![]);
        cmp(object!{}, object!{ "key" => "value", "other" => "data" }, vec![]);
    }

    #[test]
    fn test_compare_objects_keys_check_additional() {

        cmp_base(object!{}, object!{ "key" => "value" }, vec![
            (vec![], "<bg>Object <by>has <br>1 <by>additional unexpected key(s) (<br>key)")

        ], 1, true);

        cmp_base(object!{}, object!{ "key" => "value", "other" => "data" }, vec![
            (vec![], "<bg>Object <by>has <br>2 <by>additional unexpected key(s) (<br>key, <br>other)")

        ], 1, true);

    }

    #[test]
    fn test_compare_objects_deep_ignore_additional() {
        cmp_base(object!{
            "key" => "value",
            "number" => 2,
            "missing" => "key"

        }, object!{
            "key" => "",
            "number" => 4,
            "additional" => "key"

        }, vec![
            (vec![".<bb>key"], "<bg>String (\"<br>\") <by>does not match expected value (\"<bg>value\")"),
            (vec![".<bb>number"], "<bg>Number (<br>4) <by>does not match expected value (<bg>2)"),
            (vec![], "<bg>Object <by>is missing <br>1 <by>key(s) (<br>missing)")

        ], 2, false);
    }

    #[test]
    fn test_compare_objects_deep_check_additional() {
        cmp_base(object!{
            "key" => "value",
            "number" => 2,
            "missing" => "key"

        }, object!{
            "key" => "",
            "number" => 4,
            "additional" => "key"

        }, vec![
            (vec![".<bb>key"], "<bg>String (\"<br>\") <by>does not match expected value (\"<bg>value\")"),
            (vec![".<bb>number"], "<bg>Number (<br>4) <by>does not match expected value (<bg>2)"),
            (vec![], "<bg>Object <by>is missing <br>1 <by>key(s) (<br>missing)"),
            (vec![], "<bg>Object <by>has <br>1 <by>additional unexpected key(s) (<br>additional)")

        ], 2, true);
    }

    #[test]
    fn test_compare_array_deep() {
        cmp_base(array![
            "key",
            2,
            "missing"

        ], array![
            "foo",
            true

        ], vec![
            (vec![], "<bg>Array <by>with <br>2 <by>item(s) does not match expected length of <bg>3"),
            (vec!["<bp>[0]"], "<bg>String (\"<br>foo\") <by>does not match expected value (\"<bg>key\")"),
            (vec!["<bp>[1]"], "<by>Expected a <bg>Number <by>but found a <br>Boolean (<br>true)")

        ], 2, false);

    }

    #[test]
    fn test_compare_deep_paths() {
        cmp_base(object!{
            "top" => object!{
                "sub" => object!{
                    "level" => array![2]
                }
            }

        }, object!{
            "top" => object!{
                "sub" => object!{
                    "level" => array![3]
                }
            }

        }, vec![], 4, false);

        cmp_base(object!{
            "top" => object!{
                "sub" => object!{
                    "level" => array![2]
                }
            }

        }, object!{
            "top" => object!{
                "sub" => object!{
                    "level" => array![3]
                }
            }

        }, vec![
            (vec![".<bb>top", ".<bb>sub", ".<bb>level", "<bp>[0]"], "<bg>Number (<br>3) <by>does not match expected value (<bg>2)")

        ], 5, false);
    }

}

