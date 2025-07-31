#![allow(dead_code)]

mod parser;
mod tree;

use crate::{
    parser::parse,
    tree::{Node, Term},
};

// TODO: preliminary HTML output
// TODO: automatic testing for stage1 - conditions where each type of term parses
// TODO: automatic testing for stage2 - mostly the indent and spacing stuff

fn main() {
    match begin() {
        Ok(()) => {}
        Err(e) => eprintln!("error: {}", e),
    }
}

fn begin() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    eprintln!("Size of Term: {:?}", std::mem::size_of::<tree::Term>());

    eprintln!("Args: {:?}", args);
    if args.len() != 2 {
        return Err("bad arguments.\nUsage: PROGNAME <FILE>".into());
    }

    let file_contents = std::fs::read_to_string(&args[1])
        .map_err(|e| format!("failed to open input file: {:?}", e))?;

    let result = parse(&file_contents)?;
    for child in &result.nodes {
        print_node(child, 0);
    }

    Ok(())
}

fn print_node(node: &Node, indent: usize) {
    for _ in 0..indent {
        eprint!("  ");
    }
    for term in &node.contents {
        match term {
            Term::InlineWhitespace => eprint!(" "),
            Term::Word(x) => eprint!("{x}"),
            Term::Tag(x) => eprint!("%{x}"),
            Term::InlineMath(x) => eprint!("${{{x}}}"),
            Term::DisplayMath(x) => eprint!("$${{{x}}}"),
            Term::InlineCode(x) => eprint!("`{x}`"),
            Term::InlineBold(x) => eprint!("*{x}*"),
            Term::InlineItalics(x) => eprint!("_{x}_"),
            Term::TaskPrefix { state, format } => eprint!("{state:?} {format:?}"),
        }
    }
    eprintln!();
    if node.bottom_spacing {
        eprintln!("»");
    }

    for child in &node.children {
        print_node(child, indent + 1);
    }
}
