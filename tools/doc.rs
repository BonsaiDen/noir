// Crates ---------------------------------------------------------------------
extern crate regex;


// STD Dependencies -----------------------------------------------------------
use std::fs;
use std::env;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::path::PathBuf;


// External Dependencies ------------------------------------------------------
use regex::{Regex, Captures};


// Test to Doc Terminal HTML Inserter -----------------------------------------
fn main() {
    for argument in env::args() {
        if argument == "parse" {
            generate_terminal_html();

        } else if argument == "render" {
            generate_docs();
        }
    }
}


fn generate_docs() {

    // Collect rendered Terminal HTML
    println!("Collecting generated terminal HTML...");
    let mut terminal_htmls = Vec::new();
    for entry in fs::read_dir("target/terminal_html").unwrap() {
        let entry = entry.unwrap();
        let meta = fs::metadata(entry.path()).unwrap();
        if meta.is_file() {
            let filename = entry.file_name().into_string().unwrap();
            if filename.ends_with(".html") {
                let mut html = String::new();
                let mut file = fs::File::open(entry.path()).unwrap();
                file.read_to_string(&mut html).expect("Failed to read terminal html file.");
                terminal_htmls.push(
                    (filename.split(".").next().unwrap().to_string(), html, 0)
                );
            }
        }
    }
    println!("Done...");

    // Generate Docs and fill in rendered Terminal HTML
    println!("Running 'cargo doc'...");
    let _ = Command::new("cargo")
                    .arg("doc")
                    .arg("--no-deps")
                    .arg("--package")
                    .arg("noir")
                    .output();

    println!("Done...");

    // Search through all struct and trait html files
    for entry in fs::read_dir("target/doc/noir").unwrap() {
        let entry = entry.unwrap();
        let meta = fs::metadata(entry.path()).unwrap();
        if meta.is_file() {
            let filename = entry.file_name().into_string().unwrap();
            if filename.starts_with("trait") || filename.starts_with("struct") {
                update_html_file(filename, entry.path(), &mut terminal_htmls);
            }
        }
    }

    // Log unused terminal HTMLs
    for (name, _, usage_count) in terminal_htmls {
        if usage_count == 0 {
            println!("'{}' is unused.", name);
        }
    }

}

fn update_html_file(
    filename: String,
    path: PathBuf,
    terminal_htmls: &mut Vec<(String, String, usize)>
) {


    let css_pattern = Regex::new("</head>").unwrap();
    let terminal_pattern = Regex::new("<a href=\"terminal://([a-z_]+)\">(.*?)</a>").unwrap();

    // Read doc HTML file
    let mut html = String::new();
    {
        let mut file = fs::File::open(path.clone()).unwrap();
        file.read_to_string(&mut html).expect("Failed to read doc html file.");
    }

    // Read CSS HTML file
    let mut css = String::new();
    {
        let mut file = fs::File::open("tools/terminal.css").unwrap();
        file.read_to_string(&mut css).expect("Failed to read terminal css file.");
    }

    // Replace all test prefixes with all of the matching terminal HTMLs
    let new_html = terminal_pattern.replace_all(html.as_str(), |caps: &Captures| {

        let prefix = format!("test_{}", caps.at(1).unwrap_or(""));
        let class = caps.at(2).unwrap_or("collapsed");

        terminal_htmls.iter_mut().filter(|t| {
            t.0 == prefix

        }).map(|t| {

            // Increase usage count
            t.2 = t.2 + 1;

            // Render HTML and header
            if class == "expanded" {
                format!(
                    "<p>From <code>{}</code> (<a class=\"expandable\" onclick=\"toggleTerminal(this)\">Collapse</a>)</p><div class=\"terminal-expanded\">{}</div>",
                    t.0,
                    t.1
                )

            } else {
                format!(
                    "<p>From <code>{}</code> (<a class=\"expandable\" onclick=\"toggleTerminal(this)\">Expand</a>)</p><div class=\"terminal-collapsed\">{}</div>",
                    t.0,
                    t.1
                )
            }

        }).collect::<Vec<String>>().join("\n")

    });

    if new_html != html {

        let script = r#"
        <script type="text/javascript">
        function toggleTerminal(el) {

            var code = $(el).parent().next();
            if (code.hasClass('terminal-expanded')) {
                el.innerText = 'Expand';
                code.get(0).className = 'terminal-collapsed';

            } else {
                el.innerText = 'Collapse';
                code.get(0).className = 'terminal-expanded';
            }

            window.event.preventDefault();
            return false;

        }
        </script>
        "#;

        let css = format!("<style>{}</style>{}", css, script);
        let new_html = css_pattern.replace(new_html.as_str(), css.as_str());

        // Write out modified doc HTML file
        let mut file = fs::File::create(path).unwrap();
        file.write_all(new_html.as_bytes()).ok();
        println!("Replaced matches in '{}'...", filename);

    } else {
        println!("No terminal output matches in '{}'...", filename);
    }

}

fn generate_terminal_html() {

    // Run Doc tests and capture output
    println!("Running 'cargo test'...");
    let child = Command::new("cargo")
                        .env("RUST_TEST_THREADS", "1")
                        .arg("test")
                        .arg("--")
                        .arg("--nocapture")
                        .output()
                        .expect("Failed to run 'cargo test'");

    // Parse out potential
    println!("Parsing test output...");
    let data = String::from_utf8_lossy(&child.stdout);
    let test_lines = data.lines().filter(|l| {
        match l.chars().next() {

            // Colored
            Some('\u{1b}') => true,

            // Whitespace prefixed
            Some(' ') => true,

            // Test name markers
            Some(_) => l.starts_with("test test"),

            // Empty lines
            None => true
        }

    }).collect::<Vec<&str>>();

    let mut tests: Vec<(String, String)> = Vec::new();
    let mut successive_lines: Vec<String> = Vec::new();
    let mut empty_lines = 0;

    // Parse out tests and their comparison results
    for line in test_lines {

        if line.is_empty() {
            empty_lines +=1;
            if empty_lines >= 2 {

                let test = successive_lines.join("\n");
                let test = test.trim_matches('\n').to_string();

                successive_lines.clear();
                empty_lines = 0;

                // Ignore anything which probably isn't a test
                if !test.is_empty() && test.starts_with("test test") {

                    let mut lines = test.split('\n').collect::<Vec<&str>>();

                    // Ignore anything which has less than 1 line
                    if lines.len() >= 1 {

                        // Extract the test name
                        let mut test_name = String::new();
                        loop {
                            let is_test = if let Some(line) = lines.get(0) {
                                line.starts_with("test test")

                            } else {
                                false
                            };

                            if is_test {
                                test_name = lines.remove(0).split(' ').skip(1).next().unwrap().to_string();

                            } else {
                                break;
                            }

                        }

                        if !lines.is_empty() {
                            tests.push((test_name.to_string(), lines.join("\n")));
                        }

                    }

                }

            }

        } else {
            empty_lines = 0;
        }

        successive_lines.push(line.trim_right().to_string());

    }

    // Render Terminal Output as HTML
    fs::create_dir("target/terminal_html").ok();

    for (name, output) in tests.into_iter() {

        print!("{:?}.html ...", name);

        let mut file = std::fs::File::create(format!("target/terminal_html/{}.html", name)).unwrap();
        let mut child = Command::new("terminal-to-html")
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .spawn().unwrap();

        child.stdin.as_mut().unwrap().write_all(output.as_bytes()).ok();

        let output = child.wait_with_output().expect("Failed to wait on child");

        file.write_all(b"<div class=\"term-container\">").ok();
        file.write_all(&output.stdout).ok();
        file.write_all(b"</div>").ok();

        println!(" rendered");

    }

}

