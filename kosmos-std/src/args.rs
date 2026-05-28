use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;
use crate::prelude::*;

extern crate alloc;

pub struct Args {
    positionals: Vec<String>,
    flags: Vec<String>,
    values: Vec<(String, String)>,
}

impl Args {
    pub fn parse(argv: &[&str]) -> Self {
        let mut positionals = Vec::new();
        let mut flags = Vec::new();
        let mut values: Vec<(String, String)> = Vec::new();
        let mut i = 1; // skip argv[0]

        while i < argv.len() {
            let token = argv[i];

            if token.starts_with("--") {
                // --long or --long value
                let name = token[2..].to_string();
                if let Some(next) = argv.get(i + 1) {
                    if !next.starts_with('-') {
                        values.push((name, next.to_string()));
                        i += 2;
                        continue;
                    }
                }
                flags.push(name);

            } else if token.starts_with('-') && token.len() > 1 {
                // -a or -la or -l
                for c in token[1..].chars() {
                    flags.push(c.to_string());
                }
            } else {
                positionals.push(token.to_string());
            }

            i += 1;
        }

        Self { positionals, flags, values }
    }

    // check if a flag is set, by short or long name
    // flag("a") or flag("all") both work
    pub fn flag(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f == name)
    }

    // get a value for --key value
    pub fn value(&self, name: &str) -> Option<&str> {
        self.values.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    // positional args (not flags)
    pub fn positionals(&self) -> &[String] {
        self.positionals.as_slice()
    }

    // get first positional or a default
    pub fn positional(&self, index: usize, default: &str) -> String {
        self.positionals.get(index).cloned().unwrap_or_else(|| default.to_string())
    }
}

pub struct Help {
    name: &'static str,
    about: &'static str,
    entries: Vec<(&'static str, &'static str, &'static str)>, // short, long, help
}

impl Help {
    pub fn new(name: &'static str, about: &'static str) -> Self {
        Self { name, about, entries: Vec::new() }
    }

    pub fn flag(mut self, short: &'static str, long: &'static str, help: &'static str) -> Self {
        self.entries.push((short, long, help));
        self
    }

    pub fn print(&self) {
        vga_println!("usage: {} [OPTIONS] [ARGS]", self.name);
        vga_println!("{}", self.about);
        vga_println!("");
        vga_println!("options:");
    
        // determine column width
        let mut col_width = "  -h, --help".len();
        for (short, long, _) in &self.entries {
            let w = alloc::format!("  -{}, --{}", short, long).len();
            if w > col_width { 
                col_width = w; 
            }
        }
        col_width += 2; // gap between flag and help
    
        // print --help first
        let help_flag = "  -h, --help";
        vga_print!("{}", help_flag);
        for _ in help_flag.len()..col_width {
            vga_print!(" ");
        }
        vga_println!("print this help message");
    
        for (short, long, help) in &self.entries {
            let flag = alloc::format!("  -{}, --{}", short, long);
            vga_print!("{}", flag);
            for _ in flag.len()..col_width {
                vga_print!(" ");
            }
            vga_println!("{}", help);
        }
    }
}