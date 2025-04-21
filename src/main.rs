mod parser;
mod tree;

fn main() {
    match do_everything() {
        Ok(()) => {}
        Err(e) => eprintln!("error: {}", e),
    }
}

fn do_everything() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    eprintln!("Size of Term: {:?}", std::mem::size_of::<tree::Term>());

    eprintln!("Args: {:?}", args);
    if args.len() != 2 {
        return Err("bad arguments.\nUsage: PROGNAME <FILE>".into());
    }

    let file_contents = std::fs::read_to_string(&args[1])
        .map_err(|e| format!("failed to open input file: {:?}", e))?;

    eprintln!("{:#?}", crate::parser::parse_str(&file_contents)?);

    Ok(())
}
