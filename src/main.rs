#![allow(dead_code)]
#![deny(unused_must_use)]

mod cli;
mod html;
mod parser;

use crate::cli::Backend;
use crate::html::{write_html, HtmlOptions};
use crate::parser::parse;

// TODO: make tests for stage1 - conditions where each type of term parses
// TODO: make tests for stage2 - mostly the indent and spacing stuff

fn main() {
    match app() {
        Ok(()) => {}
        Err(e) => {
            if e.len() > 0 {
                eprintln!("error: {}", e);
            }
            std::process::exit(1);
        }
    }
}

fn app() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    let options = cli::parse_options(&args)?;

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
    }

    Ok(())
}
