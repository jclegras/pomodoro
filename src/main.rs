// filepath: src/main.rs
//! A command-line Pomodoro timer application with interactive controls.
mod command_dispatcher;
mod session_timer;
mod types;

use std::sync::{Arc, Mutex};
use std::{
    sync::mpsc::{self},
    thread,
    time::Duration,
};

use clap::Parser;

use command_dispatcher::CommandDispatcher;
use crossterm::terminal;
use session_timer::SessionTimer;
use types::AppError;
use types::SessionType;

use types::Command;

#[derive(Parser)]
struct Config {
    #[arg(short, long = "work", default_value_t = 25)]
    work_duration: u64,
    #[arg(short, long = "short-break", default_value_t = 5)]
    short_break: u64,
    #[arg(short, long = "long-break", default_value_t = 15)]
    long_break: u64,
    #[arg(short, long = "cycles", default_value_t = 4)]
    cycles: u64,
    #[arg(short, long = "no-sound", default_value_t = false)]
    no_sound: bool,
}

fn main() {
    let config = Config::parse();
    let (tx, rx) = mpsc::channel::<Command>();

    let rx_arc = Arc::new(Mutex::new(rx));

    println!(
        "Starting Pomodoro: {} min work, {} min short break, {} min long break, {} cycles, sound: {}\n",
        config.work_duration,
        config.short_break,
        config.long_break,
        config.cycles,
        if config.no_sound { "off" } else { "on" }
    );

    let command_dispatcher_thread = thread::spawn(move || CommandDispatcher::new(tx).run());

    let mut total_work_cycles = 0;

    'controllerCycle: loop {
        for current_cycle in 1..=config.cycles {
            let mut session_timer = SessionTimer::new(
                Arc::clone(&rx_arc),
                Duration::from_secs(config.work_duration) * 60,
                SessionType::Work("Work session"),
                current_cycle,
                config.cycles,
                config.no_sound,
            );

            let session_timer_thread = thread::spawn(move || session_timer.run());

            match session_timer_thread.join() {
                Ok(res) => {
                    if let Err(_) = res {
                        break 'controllerCycle;
                    } else {
                        total_work_cycles += 1;
                    }
                }
                Err(e) => {
                    eprintln!("Worker thread panicked: {:?}", e);
                }
            }

            let (break_duration, break_type) = if current_cycle == config.cycles {
                (
                    Duration::from_secs(config.long_break * 60),
                    SessionType::LongBreak("Long break"),
                )
            } else {
                (
                    Duration::from_secs(config.short_break * 60),
                    SessionType::ShortBreak("Short break"),
                )
            };

            let mut session_timer = SessionTimer::new(
                Arc::clone(&rx_arc),
                break_duration,
                break_type,
                current_cycle,
                config.cycles,
                config.no_sound,
            );

            let session_timer_thread = thread::spawn(move || session_timer.run());

            match session_timer_thread.join() {
                Ok(res) => {
                    if let Err(_) = res {
                        break 'controllerCycle;
                    }
                }
                Err(e) => {
                    eprintln!("Worker thread panicked: {:?}", e);
                }
            }
        }
    }

    println!(
        "\nPomodoro session ended. Total work cycles completed: {} for a total of {} min",
        total_work_cycles,
        total_work_cycles * config.work_duration
    );

    // Wait for the command dispatcher to finish
    match command_dispatcher_thread.join().unwrap() {
        Ok(_) => (),
        Err(_) => terminal::disable_raw_mode().unwrap(),
    }
}
