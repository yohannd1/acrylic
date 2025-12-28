#![allow(dead_code)]
#![deny(unused_must_use)]

mod cli;
mod html;
mod parser;

use crate::cli::{CliArg, CliOption, CliParser};
use crate::html::{write_html, HtmlOptions};
use crate::parser::parse;
use std::fs::File;
use std::io::{self, Read, Write};

// TODO: make tests for stage1 - conditions where each type of term parses
//
// TODO: make tests for stage2 - mostly the indent and spacing stuff
//
// TODO: pipeline stage 3 - some terms become line attributes or types of lines - display math
// becomes its own kind of line; task prefixes can affect the style of the entire line; bullet point
// styles; and i might be able to more efficiently handle spacing at the end of fold blocks (they
// shouldn't be hidden inside it) ...

pub type ReadFileGen = Box<dyn FnOnce() -> io::Result<Box<dyn Read>>>;
pub type WriteFileGen = Box<dyn FnOnce() -> io::Result<Box<dyn Write>>>;

pub struct Options {
    pub katex_path: String,
    pub backend: Backend,
    pub in_file_gen: ReadFileGen,
    pub out_file_gen: WriteFileGen,
}

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    Html,
    Debug,
    None,
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

    let file_contents = {
        let mut s = String::new();
        let mut f =
            (options.in_file_gen)().map_err(|e| format!("failed to open input file: {:?}", e))?;
        f.read_to_string(&mut s)
            .map_err(|e| format!("failed to read from file: {:?}", e))?;
        s
    };

    let result = parse(&file_contents)?;

    let mut file =
        (options.out_file_gen)().map_err(|e| format!("failed to open output file: {:?}", e))?;

    match options.backend {
        Backend::Html => {
            let html_options = HtmlOptions {
                katex_path: &options.katex_path,
            };
            write_html(&mut file, &result, &html_options)
                .map_err(|e| format!("failed to write to file: {:?}", e))?;
        }
        Backend::Debug => {
            write!(&mut file, "{result:#?}\n")
                .map_err(|e| format!("failed to write to file: {:?}", e))?;
        }
        Backend::None => {}
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
        help: "the output backend (options: html, debug, none; default: html)".into(),
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
        Some("none") => Backend::None,
        Some(x) => return Err(p.error_help(format!("Unknown backend {x:?}"))),
    };

    let katex_path = p
        .get_option("--katex-path")
        .and_then(|x| x.value.clone())
        .unwrap_or_else(String::new);

    let in_file_gen: ReadFileGen = match p.get_arg("FILE").and_then(|x| x.value.as_deref()).unwrap()
    {
        "-" => Box::new(|| Ok(Box::new(io::stdin()))),
        path_ref => {
            let path = path_ref.to_owned();
            Box::new(move || File::open(path).and_then(|x| Ok(Box::new(x) as Box<dyn Read>)))
        }
    };

    let out_file_gen: WriteFileGen = match p.get_option("--output").and_then(|x| x.value.as_deref())
    {
        Some("-") | None => Box::new(|| Ok(Box::new(io::stdout()))),
        Some(path_ref) => {
            let path = path_ref.to_owned();
            Box::new(move || File::create(path).and_then(|x| Ok(Box::new(x) as Box<dyn Write>)))
        }
    };

    Ok(Options {
        katex_path,
        backend,
        in_file_gen,
        out_file_gen,
    })
}
