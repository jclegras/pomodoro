// filepath: src/session_timer.rs
//! Module handling the session timer logic for a Pomodoro timer application.
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use indicatif::ProgressBar;
use notify_rust::Notification;
use rodio::OutputStream;
use rodio::source::{SineWave, Source};

// Replace these with the correct paths to your types:
use crate::AppError;
use crate::Command;
use crate::SessionType;

pub struct SessionTimer {
    rx: Arc<Mutex<Receiver<Command>>>,
    duration: Duration,
    is_paused: bool,
    session: SessionType,
    current_cycle: u64,
    total_cycles: u64,
    sound: bool,
    sink: rodio::Sink,
    _stream: OutputStream, // Keep the stream alive
}

impl SessionTimer {
    pub fn new(
        rx: Arc<Mutex<Receiver<Command>>>,
        duration: Duration,
        session: SessionType,
        current_cycle: u64,
        total_cycles: u64,
        no_sound: bool,
    ) -> Self {
        let mut stream =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        stream.log_on_drop(false);
        SessionTimer {
            rx: rx,
            is_paused: false,
            duration,
            session,
            current_cycle,
            total_cycles,
            sound: !no_sound,
            sink: rodio::Sink::connect_new(stream.mixer()),
            _stream: stream,
        }
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        let progress_bar = ProgressBar::new(self.duration.as_secs());
        progress_bar.set_message(format!(
            "{} (#{}/{})",
            self.session, self.current_cycle, self.total_cycles,
        ));
        progress_bar.set_style(
            indicatif::ProgressStyle::with_template(
                "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta}) < {msg} >",
            )
            .unwrap()
            .progress_chars("#>-"),
        );
        progress_bar.tick();

        let mut remaining_secs = self.duration.as_secs();
        while remaining_secs > 0 {
            if remaining_secs == 10 {
                send_notification(&format!("{}: 00:10s left", self.session));
            }

            if self.is_paused {
                match self.rx.lock().unwrap().recv() {
                    Ok(cmd) => match cmd {
                        Command::Resume | Command::PauseResume => {
                            self.is_paused = false;
                            progress_bar.reset_eta();
                        }
                        _ => {}
                    },
                    Err(e) => return Err(AppError::ChannelRecv(e)),
                }
            } else {
                match self.rx.lock().unwrap().recv_timeout(Duration::from_secs(1)) {
                    Ok(cmd) => match cmd {
                        Command::Skip if !matches!(self.session, SessionType::Work(_)) => {
                            break;
                        }
                        Command::Pause | Command::PauseResume => {
                            self.is_paused = true;
                        }
                        Command::Reset => {
                            remaining_secs = self.duration.as_secs();
                            progress_bar.set_position(0);
                            progress_bar.reset_eta();
                        }
                        _ => {}
                    },
                    Err(RecvTimeoutError::Timeout) => {
                        progress_bar.inc(1);
                        remaining_secs -= 1;
                    }
                    Err(e) => {
                        return Err(AppError::ChannelRecvTimeout(e)); // Command Dispatcher stopped
                    }
                }
            }
        }
        if remaining_secs == 0 && self.sound {
            play_sound(&self.sink);
        }
        Ok(())
    }
}

fn play_sound(sink: &rodio::Sink) {
    let source = SineWave::new(440.0)
        .take_duration(Duration::from_secs_f32(0.25))
        .amplify(0.20);

    sink.append(source);

    // The sound plays in a separate thread. This call will block the current thread until the sink
    // has finished playing all its queued sounds.
    sink.sleep_until_end();
}

fn send_notification(message: &str) {
    Notification::new()
        .summary("Pomodoro Timer")
        .body(message)
        .icon("dialog-information")
        .show()
        .expect("Failed to send notification.");
}
