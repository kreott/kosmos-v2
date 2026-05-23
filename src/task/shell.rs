use alloc::collections::BTreeMap;
use pc_keyboard::{Keyboard, ScancodeSet1, layouts};
use crate::filesystem::fat32;
use crate::macros::*;
use crate::task::keyboard::{self, ScancodeStream};
use pc_keyboard::HandleControl;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;


pub struct Shell {
    scancodes: ScancodeStream,
    keyboard: Keyboard<layouts::Us104Key, ScancodeSet1>,
    vars: BTreeMap<String, String>,
    cwd: String,
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
            cwd: "/".to_string(),
        }
    }

    pub fn init(&mut self) {
        self.vars.insert("path".to_string(), "/usr/bin".to_string());
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
            "echo" => self.cmd_echo(args),
            "clear" =>  self.cmd_clear(args),
            "help" => self.cmd_help(args),
            "cd" => self.cmd_cd(args),
            "" => {}
            _ => self.run_external(cmd, args),
        }
    }

    fn expand_vars(&self, s: &str) -> String {
        // replace $VAR with its value
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '$' {
                let mut name = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        name.push(next);
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

    /// Resolves paths for usage in commands
    fn resolve_path(&self, path: &str) -> String {
        let base = if path.starts_with('/') {
            Vec::new()
        } else {
            self.cwd.split('/').filter(|s| !s.is_empty()).collect()
        };
    
        let mut parts: Vec<&str> = base;
    
        for component in path.split('/') {
            match component {
                "" | "." => {}
                ".." => { parts.pop(); }
                c => parts.push(c),
            }
        }
    
        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parts.join("/"))
        }
    }

    fn cmd_echo(&self, args: &str) {
        println!("{}", self.expand_vars(args));
    }

    fn cmd_clear(&self, _args: &str) {
        clear!();
    }

    fn cmd_help(&self, _args: &str) {
        println!(
            "built-in commands: 
                echo - Echo to terminal
                cd - Change directory
                clear - Clear terminal
                
            for external commands, check /usr/bin/ for executables."
        );
    }

    fn cmd_cd(&mut self, path: &str) {
        let path = path.trim();
    
        if path.is_empty() {
            self.cwd = "/".to_string();
            return;
        }
    
        if path.split_whitespace().count() > 1 {
            println!("cd: too many arguments");
            return;
        }
    
        let new_path = self.resolve_path(path);
        let check = new_path.clone();
        let mut exists = false;
    
        fat32::with_fs(|fs| {
            exists = if check == "/" {
                true
            } else {
                fs.root_dir()
                    .open_dir(check.trim_start_matches('/'))
                    .is_ok()
            };
        });
    
        if exists {
            self.cwd = new_path;
        } else {
            println!("cd: {}: no such directory", path);
        }
    }

    fn run_external(&mut self, cmd: &str, _args: &str) {
        let path = alloc::format!("/usr/bin/{}", cmd);
        if !crate::loader::load_and_run(&path) {
            println!("{}: command not found", cmd);
        }
    }
}

pub async fn shell_task() {
    let mut shell = Shell::new();
    shell.init();

    loop {
        print!("{} > ", shell.cwd);
        let input = shell.read_line().await;
        shell.handle_command(input.as_str());
    }
}