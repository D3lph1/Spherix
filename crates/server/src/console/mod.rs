use std::io::Write;

use bevy_ecs::prelude::Resource;
use flume::Sender;
use rustyline::config::Configurer;
use rustyline::{Cmd, KeyCode, KeyEvent, Modifiers};
use tokio_util::sync::CancellationToken;

use crate::console::cmd::CmdHub;
use crate::console::msg::{Command, CommandSource};

pub mod cmd;
pub mod msg;

const HISTORY_FILE_PATH: &str = ".console";

pub struct Console {
    messages_tx: Sender<Command>,
    cancel: CancellationToken,
}

impl Console {
    pub fn new(messages_tx: Sender<Command>, cancel: CancellationToken) -> Console {
        Console {
            messages_tx,
            cancel,
        }
    }

    pub fn serve(self) {
        let mut hub = CmdHub::new();

        // hub.add(Box::new(Kick { server: self.server.clone() }));
        // hub.add(Box::new(List { server: self.server.clone() }));

        loop {
            let mut rl = rustyline::DefaultEditor::new().unwrap();
            rl.set_auto_add_history(true);

            let _ = rl.load_history(HISTORY_FILE_PATH);

            rl.bind_sequence(KeyEvent(KeyCode::Up, Modifiers::NONE), Cmd::PreviousHistory);
            rl.bind_sequence(KeyEvent(KeyCode::Down, Modifiers::NONE), Cmd::NextHistory);

            let res = rl.readline("");

            if res.is_err() {
                self.cancel.cancel();

                return;
            }

            let _input = res.unwrap();
            let input = _input.trim();

            if input == "exit" {
                self.cancel.cancel();

                return;
            }

            if input == "" {
                continue;
            }

            if input.to_lowercase() == "cls" {
                print!("\x1B[2J\x1B[1;1H");
                std::io::stdout().flush().unwrap();
            }

            self.messages_tx
                .send(Command {
                    source: CommandSource::Console,
                    text: input.to_owned(),
                })
                .unwrap();

            // hub.match_cmd(input);

            rl.save_history(HISTORY_FILE_PATH).unwrap();
        }
    }
}
