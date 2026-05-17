use alloc::collections::BTreeMap;
use pc_keyboard::{Keyboard, ScancodeSet1, layouts};

use crate::macros::*;
use crate::task::keyboard::{self, ScancodeStream};
use pc_keyboard::HandleControl;
use alloc::string::{String, ToString};


pub struct Shell {
    scancodes: ScancodeStream,
    keyboard: Keyboard<layouts::Us104Key, ScancodeSet1>,
    vars: BTreeMap<String, String>,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            scancodes: ScancodeStream::new(),
            keyboard: Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore,
            ),
            vars: BTreeMap::new(),            
        }
    }

    pub async fn read_line(&mut self) -> String {
        keyboard::read_line(&mut self.scancodes, &mut self.keyboard).await
    }

    pub fn handle_command(&mut self, input: &str) {
        if let Some((key, val)) = input.split_once('=') {
            let key = key.trim().to_string();
            let val = val.trim().to_string();
            self.vars.insert(key, val);
            return;
        }

        let mut parts = input.splitn(2, ' ');
        let cmd = parts.next().unwrap_or("");
        let args = parts.next().unwrap_or("");

        match cmd {
            "echo" => println!("{}", self.expand_vars(args)),
            "clear" =>  clear!(),
            "help" => println!("commands: echo, clear, help"),
            _ => println!("unknown command: {}", cmd),
        }
    }

    fn expand_vars(&self, s: &str) -> String {
        // replace $VAR with its value
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '$' {
                let mut name = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' {
                        name.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Some(val) = self.vars.get(&name) {
                    result.push_str(val);
                }
            } else {
                result.push(c);
            }
        }
        result
    }
}

pub async fn shell_task() {
    let mut shell = Shell::new();
    println!("shell task started");

    loop {
        print!("placeholder> ");
        let input = shell.read_line().await;
        shell.handle_command(input.as_str());
    }
}