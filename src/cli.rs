use std::path::{Path, PathBuf};

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
}

pub fn show_help(progname: &str) {
    eprintln!("{progname}: parse and render acrylic files (.acr)");
    eprintln!();
    eprintln!("Usage: {progname} [OPTIONS] <FILE>");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --help/-h: show this help message");
    eprintln!("  --output/-o: the output file (defaults to stdout)");
    eprintln!("  --backend/-b: the output backend (defaults to HTML)");
    eprintln!("  --katex-path/-k: the KaTeX path (for the HTML backend)");
}

// TODO: refactor this into a state machine. probably would be muuuch better.
pub fn parse_options(args: &[String]) -> Result<Options, String> {
    let progname = get_progname(&args).expect("program name not available");

    let mut katex_path = None;
    let mut backend = Backend::Html;
    let mut input_path = None;
    let mut output_path = None;

    fn is_option(arg: &str) -> bool {
        arg.starts_with("-") && arg.len() > 1
    }

    let get_arg_value = |i: &mut usize, arg: &str| -> Result<&str, String> {
        match args.get(*i + 1).filter(|x| !is_option(x)) {
            Some(a) => {
                *i += 2;
                Ok(a)
            }
            None => {
                show_help(progname);
                Err(format!("missing value for argument {arg}"))
            }
        }
    };

    let mut i = 1;
    loop {
        match args.get(i).map(String::as_str) {
            Some("-h" | "--help") => {
                show_help(progname);
                return Err("".into());
            }
            Some("-k" | "--katex-path") => {
                let value = get_arg_value(&mut i, "--katex-path")?;
                katex_path = Some(value.to_owned());
            }
            Some("-o" | "--output") => {
                let value = get_arg_value(&mut i, "--output")?;
                output_path = Some(value)
                    .filter(|x| *x != "-")
                    .map(Path::new)
                    .map(Path::to_owned);
            }
            Some("-b" | "--backend") => {
                let value = get_arg_value(&mut i, "--backend")?;
                backend = match value {
                    "html" => Backend::Html,
                    _ => {
                        return Err(format!("unknown backend {value:?}"));
                    }
                };
            }
            Some(x) if is_option(x) => {
                show_help(progname);
                return Err(format!("unknown option {x:?}"));
            }
            Some(x) => {
                if input_path.is_some() {
                    show_help(progname);
                    return Err(format!("unknown argument {x:?}"));
                } else {
                    if x == "-" {
                        input_path = Some(PathBuf::from("/dev/stdin"));
                    } else {
                        input_path = Some(PathBuf::from(x));
                    }
                    i += 1;
                }
            }
            None => break,
        }
    }

    Ok(Options {
        katex_path: katex_path.unwrap_or_else(String::new),
        backend,
        input_path: input_path.ok_or_else(|| {
            show_help(progname);
            format!("missing FILE argument")
        })?,
        output_path: output_path.unwrap_or_else(|| PathBuf::from("/dev/stdout")),
    })
}

fn get_progname(args: &[String]) -> Option<&str> {
    args.get(0)
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(|x| x.try_into().ok())
}
