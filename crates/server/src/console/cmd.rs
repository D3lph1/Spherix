use std::collections::HashMap;
use std::sync::Arc;

use clap::{Arg, ArgMatches, Command};
use owo_colors::OwoColorize;

use crate::server::Server;

pub struct CmdHub {
    cmds: HashMap<String, Box<dyn Cmd>>
}

impl CmdHub {
    pub fn new() -> CmdHub {
        CmdHub {
            cmds: HashMap::new()
        }
    }

    pub fn add(&mut self, cmd: Box<dyn Cmd>) {
        let def = cmd.get_def();

        self.cmds.insert(String::from(def.get_name()), cmd);
    }

    pub fn match_cmd(&mut self, input: &str) {
        let words = shellwords::split(input).unwrap();

        if let Some(cmd) = self.cmds.get_mut(&words[0]) {
            let matches = cmd.get_def().try_get_matches_from(words);

            if matches.is_err() {
                matches.err().unwrap().print().unwrap();
            } else {
                cmd.handle(matches.unwrap());
            }
        }
    }
}

pub trait Cmd {
    fn get_def(&self) -> Command;

    fn handle(&mut self, cmd: ArgMatches);
}

pub struct Kick {
    pub(crate) server: Arc<Server>
}

impl Into<Command> for Kick {
    fn into(self) -> Command {
        self.get_def()
    }
}

impl Cmd for Kick {
    fn get_def(&self) -> Command {
        Command::new("kick")
            .about("Kicks a player off the server, and displays an optional reason to it")
            .arg(
                Arg::new("target")
                    .required(true)
            )
            .arg(Arg::new("reason"))
    }

    fn handle(&mut self, cmd: ArgMatches) {
        let target: &str = cmd.get_one::<String>("target").map(|s| s.as_str()).unwrap();
        let reason = cmd.get_one::<String>("reason").map(|s| s.as_str());

        if reason.is_some() {
            println!("User {} has been kicked with the following reason: \"{}\"", target.cyan(), reason.unwrap().yellow());
        } else {
            println!("User {} has been kicked", target.cyan());
        }
    }
}

pub struct List {
    pub(crate) server: Arc<Server>
}

pub enum ListArg {
    Uuids
}

impl Into<Command> for List {
    fn into(self) -> Command {
        self.get_def()
    }
}

impl Cmd for List {
    fn get_def(&self) -> Command {
        Command::new("list")
            .about("Lists players on the server")
            .arg(
                Arg::new("uuids")
                    .help("If specified, player UUIDs are shown alongside names")
                    .default_value("")
            )
    }

    fn handle(&mut self, _cmd: ArgMatches) {
        // let mut players = Vec::new();

        // for player in self.server.players.clone().lock().unwrap().iter_mut() {
        //     players.push((player.client().name.clone(), player.client().uuid));
        // }

        // for (name, uuid) in players {
        //     println!("{} {}", name, uuid)
        // }
    }
}
