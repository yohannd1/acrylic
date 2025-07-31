#![allow(dead_code)]

mod parser;
mod tree;
mod stage2;

// TODO: refactor code into different stages (one file per stage, except one file for typedefs)
// TODO: preliminary HTML output

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

    let s1 = crate::parser::parse_str(&file_contents)?;
    let s2 = crate::stage2::process(s1);

    eprintln!("{:#?}", s2);

    Ok(())
}
