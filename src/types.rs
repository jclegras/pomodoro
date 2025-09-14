// filepath: src/types.rs
//! Module defining types and errors for a Pomodoro timer application.
use std::{fmt, sync::mpsc};

#[derive(Debug, Clone)]
pub enum Command {
    Pause,
    PauseResume,
    Reset,
    Resume,
    Skip,
}

pub enum SessionType {
    Work(&'static str),
    ShortBreak(&'static str),
    LongBreak(&'static str),
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionType::Work(msg) => write!(f, "{}", msg),
            SessionType::ShortBreak(msg) => write!(f, "{}", msg),
            SessionType::LongBreak(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    ChannelSend(mpsc::SendError<Command>),
    ChannelRecv(mpsc::RecvError),
    ChannelRecvTimeout(mpsc::RecvTimeoutError),
}
