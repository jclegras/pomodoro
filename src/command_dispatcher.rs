// filepath: src/command_dispatcher.rs
//! Module handling command dispatching for a Pomodoro timer application.
use std::{collections::HashMap, sync::mpsc::Sender, time::Duration};

use crossterm::{
    event::{self, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use crate::{AppError, types::Command};

pub struct CommandDispatcher {
    tx: Sender<Command>,
    command_parser: CommandParser,
}

impl CommandDispatcher {
    pub fn new(tx: Sender<Command>) -> Self {
        CommandDispatcher {
            tx: tx,
            command_parser: CommandParser::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        println!(
            "Controls: [p] Pause | [Space] Toggle | [r] Resume | [s] Skip break | [x] Reset | [q]/[Esc]/[Ctrl+C] Quit\n"
        );
        terminal::enable_raw_mode().unwrap();
        loop {
            if event::poll(Duration::from_secs(1)).unwrap() {
                if let event::Event::Key(key_event) = event::read().unwrap() {
                    if (key_event.modifiers == KeyModifiers::CONTROL
                        && (key_event.code == KeyCode::Char('c')))
                        || key_event.code == KeyCode::Char('q')
                        || key_event.code == KeyCode::Esc
                    {
                        break;
                    }
                    if let Some(cmd) = self.command_parser.get(&key_event) {
                        self.tx.send(cmd.clone()).map_err(AppError::ChannelSend)?;
                    }
                }
            }
        }
        terminal::disable_raw_mode().unwrap();
        Ok(())
    }
}

struct CommandParser {
    commands: HashMap<String, Command>,
}

impl CommandParser {
    fn new() -> Self {
        let mut commands = HashMap::new();
        commands.insert(KeyCode::Char('p').to_string(), Command::Pause);
        commands.insert(KeyCode::Char(' ').to_string(), Command::PauseResume);
        commands.insert(KeyCode::Char('x').to_string(), Command::Reset);
        commands.insert(KeyCode::Char('r').to_string(), Command::Resume);
        commands.insert(KeyCode::Char('s').to_string(), Command::Skip);

        CommandParser { commands }
    }

    fn get(&self, input: &KeyEvent) -> Option<Command> {
        self.commands.get(&input.code.to_string()).cloned()
    }
}
