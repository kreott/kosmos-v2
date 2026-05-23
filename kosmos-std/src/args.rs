use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;

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

pub struct Matches {
    flags: Vec<(&'static str, Option<String>)>,
    positionals: Vec<String>,
    subcommand: Option<(&'static str, Box<Matches>)>,
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
    subcommands: Vec<Command>,
}

impl Command {
    
}