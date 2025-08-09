#![allow(dead_code)]
#![deny(unused_must_use)]

mod cli;
mod html;
mod parser;

use crate::cli::{CliArg, CliOption, CliParser};
use crate::html::{write_html, HtmlOptions};
use crate::parser::parse;
use std::io::Write;
use std::path::PathBuf;

// TODO: make tests for stage1 - conditions where each type of term parses
//
// TODO: make tests for stage2 - mostly the indent and spacing stuff
//
// TODO: pipeline stage 3 - some terms become line attributes or types of lines - display math
// becomes its own kind of line; task prefixes can affect the style of the entire line; bullet point
// styles; and i might be able to more efficiently handle spacing at the end of fold blocks (they
// shouldn't be hidden inside it) ...

#[derive(Debug, Clone)]
pub struct Options {
    pub katex_path: String,
    pub backend: Backend,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    Html,
    Debug,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match app(&args) {
        Ok(()) => {}
        Err(e) => {
            if e.len() > 0 {
                eprintln!("error: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn app(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;

    let file_contents = std::fs::read_to_string(options.input_path)
        .map_err(|e| format!("failed to open input file: {:?}", e))?;

    let result = parse(&file_contents)?;

    let mut file = std::fs::File::create(&options.output_path)
        .map_err(|e| format!("failed to open file: {:?}", e))?;

    match options.backend {
        Backend::Html => {
            let html_options = HtmlOptions {
                katex_path: &options.katex_path,
            };
            write_html(&mut file, &result, &html_options)
                .map_err(|e| format!("failed to write to file: {:?}", e))?;
        }
        Backend::Debug => {
            write!(&mut file, "{result:?}")
                .map_err(|e| format!("failed to write to file: {:?}", e))?;
        }
    }

    Ok(())
}

fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut p = CliParser::new(args[0].as_str());
    p.add_arg(CliArg {
        name: "FILE".into(),
        value: None,
    });
    p.add_option(CliOption {
        name: "--output".into(),
        short: "-o".into(),
        help: "the output file (defaults to stdout)".into(),
        has_arg: true,
        value: None,
    });
    p.add_option(CliOption {
        name: "--backend".into(),
        short: "-b".into(),
        help: "the output backend (defaults to HTML)".into(),
        has_arg: true,
        value: None,
    });
    p.add_option(CliOption {
        name: "--katex-path".into(),
        short: "-k".into(),
        help: "the KaTeX path (for the HTML backend)".into(),
        has_arg: true,
        value: None,
    });

    p.parse_args(&args[1..])?;

    let backend = match p.get_option("--backend").and_then(|x| x.value.as_deref()) {
        Some("html") | None => Backend::Html,
        Some("debug") => Backend::Debug,
        Some(x) => return Err(p.error_help(format!("Unknown backend {x:?}"))),
    };

    let katex_path = p
        .get_option("--katex-path")
        .and_then(|x| x.value.clone())
        .unwrap_or_else(String::new);

    let input_path = match p.get_arg("FILE").and_then(|x| x.value.as_deref()).unwrap() {
        "-" => PathBuf::from("/dev/stdin".to_string()),
        other => PathBuf::from(other),
    };

    let output_path = match p.get_option("--output").and_then(|x| x.value.as_deref()) {
        Some("-") | None => PathBuf::from("/dev/stdout"),
        Some(other) => PathBuf::from(other),
    };

    Ok(Options {
        katex_path,
        backend,
        input_path,
        output_path,
    })
}
