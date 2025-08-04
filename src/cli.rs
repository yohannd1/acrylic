use std::path::Path;

#[derive(Debug)]
pub struct CliOption {
    pub name: String,
    pub short: String,
    pub help: String,
    pub has_arg: bool,
    pub value: Option<String>,
}

impl CliOption {
    pub fn is_specified(&self) -> bool {
        self.value.is_some()
    }
}

#[derive(Debug)]
pub struct CliArg {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug)]
pub struct CliParser<'a> {
    pub options: Vec<CliOption>,
    pub args: Vec<CliArg>,
    pub progname: &'a str,
}

impl<'a> CliParser<'a> {
    pub fn new(arg0: &'a str) -> Self {
        Self {
            options: Vec::new(),
            args: Vec::new(),
            progname: get_progname(arg0).expect("failed to get program name"),
        }
    }

    pub fn add_option(&mut self, option: CliOption) {
        let CliOption { name, short, .. } = &option;
        assert!(name.starts_with("--") && name.len() >= 3);
        assert!(short != "--" && short.starts_with("-") && short.len() == 2);
        self.options.push(option);
    }

    pub fn add_arg(&mut self, arg: CliArg) {
        self.args.push(arg);
    }

    pub fn get_option(&self, query: &str) -> Option<&CliOption> {
        self.options
            .iter()
            .find(|o| o.short == query || o.name == query)
    }

    pub fn get_option_mut(&mut self, query: &str) -> Option<&mut CliOption> {
        self.options
            .iter_mut()
            .find(|o| o.short == query || o.name == query)
    }

    pub fn get_arg(&mut self, query: &str) -> Option<&CliArg> {
        self.args.iter().find(|a| query == a.name)
    }

    pub fn get_arg_mut(&mut self, query: &str) -> Option<&mut CliArg> {
        self.args.iter_mut().find(|a| query == a.name)
    }

    pub fn parse_args<I>(&mut self, args: I) -> Result<(), String>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: AsRef<str>,
    {
        let mut cur_option: Option<String> = None;
        let mut skip_opt = false;
        let mut arg_i = 0;

        fn is_option(arg: &str, skip_opt: bool) -> bool {
            !skip_opt && arg.starts_with("-") && arg != "-" && arg != "--"
        }

        fn is_skip_opt(arg: &str, skip_opt: bool) -> bool {
            !skip_opt && arg == "--"
        }

        let mut it = args.into_iter().peekable();
        while let Some(arg) = it.peek() {
            let arg: &str = arg.as_ref();

            if is_skip_opt(arg, skip_opt) {
                skip_opt = true;
                continue;
            }

            if let Some(opt) = &cur_option {
                if is_option(arg, skip_opt) {
                    return Err(self.error_help(format!("not enough args for option {opt}")));
                } else {
                    let opt_data = self.get_option_mut(opt).unwrap();
                    assert!(opt_data.has_arg && opt_data.value.is_none());
                    opt_data.value = Some(arg.to_owned());
                    cur_option = None;
                }
            } else {
                if is_option(arg, skip_opt) {
                    if arg == "-h" || arg == "--help" {
                        self.help();
                        return Err("".into());
                    }

                    let opt_data = match self.get_option_mut(arg) {
                        Some(x) => x,
                        None => return Err(self.error_help(format!("unknown option: {arg}"))),
                    };
                    if opt_data.is_specified() {
                        return Err(self.error_help(format!("option {arg} passed more than once")));
                    }

                    if opt_data.has_arg {
                        cur_option = Some(arg.to_owned());
                    } else {
                        // empty string just to signal it has been provided
                        opt_data.value = Some(String::new());
                    }
                } else {
                    let cur_arg = match self.args.get_mut(arg_i) {
                        Some(x) => x,
                        None => return Err(self.error_help(format!("too many arguments"))),
                    };
                    cur_arg.value = Some(arg.to_owned());
                    arg_i += 1;
                }
            }
            _ = it.next();
        }

        if let Some(opt) = cur_option {
            return Err(self.error_help(format!("missing argument for {opt:?}")));
        }

        if arg_i != self.args.len() {
            return Err(self.error_help(format!("too few arguments")));
        }

        Ok(())
    }

    pub fn error_help(&self, error: String) -> String {
        self.help();
        return error;
    }

    pub fn help(&self) {
        let Self { progname, .. } = self;
        eprintln!("{progname}: parse and render acrylic files (.acr)");
        eprintln!();
        eprint!("Usage: {progname} [OPTIONS]");
        for arg_def in &self.args {
            eprint!(" <{}>", arg_def.name);
        }
        eprintln!("\nOptions:");
        eprintln!("  --help/-h: show this help message");
        for opt in &self.options {
            let CliOption {
                name, short, help, ..
            } = opt;
            eprintln!("  {name}/{short}: {help}");
        }
    }
}

fn get_progname(arg0: &str) -> Option<&str> {
    Some(arg0)
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(|x| x.try_into().ok())
}
