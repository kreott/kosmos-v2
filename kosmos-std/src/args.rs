use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;
use crate::prelude::*;

extern crate alloc;

pub struct ArgDef {
    pub name: &'static str,
    pub short: Option<char>,
    pub long: Option<&'static str>,
    pub takes_value: bool,
    pub required: bool,
    pub help: &'static str,
    pub default: Option<&'static str>,
}

pub struct Matches<'a> {
    flags: Vec<(&'static str, Option<&'a str>)>,
    positionals: Vec<&'a str>,
}

pub enum ParseError {
    UnknownArg(String),
    MissingValue(&'static str),
    MissingRequired(&'static str),
}

pub struct Command {
    name: &'static str,
    about: &'static str,
    args: Vec<ArgDef>,
}

impl ArgDef {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            short: None,
            long: None,
            takes_value: false,
            required: false,
            help: "",
            default: None,
        }
    }

    pub fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    pub fn long(mut self, l: &'static str) -> Self {
        self.long = Some(l);
        self
    }

    pub fn takes_value(mut self) -> Self {
        self.takes_value = true;
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn help(mut self, help: &'static str) -> Self {
        self.help = help;
        self
    }

    pub fn default(mut self, val: &'static str) -> Self {
        self.default = Some(val);
        self
    }
}

pub trait FromArgs<'a>: Sized {
    fn args() -> Vec<ArgDef>;
    fn from_matches(m: &Matches<'a>) -> Self;
}

impl Command {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            about: "",
            args: Vec::new(),
        }
    }

    pub fn about(mut self, about: &'static str) -> Self {
        self.about = about;
        self
    }

    pub fn arg(mut self, arg: ArgDef) -> Self {
        self.args.push(arg);
        self
    }

    pub fn parse<'a>(&self, argv: &[&'a str]) -> Result<Matches<'a>, ParseError> {
        let mut flags = Vec::new();
        let mut positionals = Vec::new();
        let mut i = 1; // skip filename at argv[0]

        while i < argv.len() {
            let token = argv[i];

            if token.starts_with("--") {
                let name = &token[2..];
                let def = self.args.iter().find(|a| a.long == Some(name))
                    .ok_or_else(|| ParseError::UnknownArg(name.to_string()))?; 

                if def.takes_value {
                    i += 1;
                    let val = argv.get(i).ok_or(ParseError::MissingValue(def.name))?;
                    flags.push((def.name, Some(*val)));
                } else {
                    flags.push((def.name, None));
                }

            } else if token.starts_with('-') {
                let c = token.chars().nth(1).ok_or_else(|| ParseError::UnknownArg(token.to_string()))?;
                let def = self.args.iter().find(|a| a.short == Some(c))
                    .ok_or_else(|| ParseError::UnknownArg(token.to_string()))?;

                if def.takes_value {
                    i += 1;
                    let val = argv.get(i).ok_or(ParseError::MissingValue(def.name))?;
                    flags.push((def.name, Some(*val)));
                } else {
                    flags.push((def.name, None));
                }
            } else {
                positionals.push(token);
            }

            i += 1;
        }

        // check required args
        for def in &self.args {
            if def.required {
                let found = flags.iter().any(|(name, _)| *name == def.name);
                if !found {
                    return Err(ParseError::MissingRequired(def.name));
                }
            }
        }

        Ok(Matches { flags, positionals })
    }

    pub fn print_help(&self) {
        vga_print!("usage: {} [OPTIONS]\n\n", self.name);
        vga_print!("{}\n\n", self.about);
        vga_println!("options:");
        vga_println!("  -h, --help          print this help message");

        for arg in &self.args {
            let short = match arg.short {
                Some(c) => {
                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    // cant return &str from buf here so print directly
                    vga_print!("  -{}, ", s);
                    true
                }
                None => {
                    vga_print!("      ");
                    false
                }
            };

            match arg.long {
                Some(l) if arg.takes_value => vga_print!("--{} <{}>", l, arg.name),
                Some(l) => vga_print!("--{}", l),
                None => {}
            }

            vga_print!("\t{}\n", arg.help);
        }
    }
}

impl<'a> Matches<'a> {
    // flags that take a value
    pub fn value(&self, name: &str) -> Option<&'a str> {
        self.flags.iter().find(|(n, _)| *n == name).and_then(|(_, v)| *v)
    }

    // for boolean flags
    pub fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|(n, _)| *n == name)
    }
    
    pub fn positionals(&self) -> &[&'a str] {
        &self.positionals.as_slice()
    }
}