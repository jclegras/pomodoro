use clap::Parser;
use colored::*;
use core::fmt;
use crossterm::event::KeyModifiers;
use indicatif::ProgressBar;
use notify_rust::Notification;
use rodio::source::{SineWave, Source};
use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::{thread::sleep, time::Duration};

/// Configuration for the Pomodoro timer.
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

enum SessionType {
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

static POMODORO_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Play a sound using the specified audio sink.
/// @param sink The audio sink to use for playing sounds.
fn play_sound(sink: &rodio::Sink) {
    // Add a dummy source of the sake of the example.
    let source = SineWave::new(440.0)
        .take_duration(Duration::from_secs_f32(0.25))
        .amplify(0.20);
    sink.append(source);

    // The sound plays in a separate thread. This call will block the current thread until the sink
    // has finished playing all its queued sounds.
    sink.sleep_until_end();
}

/// Run a Pomodoro timer.
/// @param duration_mins The duration of the timer in minutes.
/// @param work_type The type of work session (e.g., work, short break, long break).
/// @param current_cycle The current cycle number.
/// @param total_cycles The total number of cycles.
/// @param sink The audio sink to use for playing sounds.
/// @param paused A flag indicating whether the timer is paused.
/// @param skip A flag indicating whether to skip the timer.
/// @param no_sound A flag indicating whether to disable sound notifications.
fn run_timer(
    duration_mins: u64,
    work_type: &SessionType,
    current_cycle: u64,
    total_cycles: u64,
    sink: &rodio::Sink,
    paused: &Arc<AtomicBool>,
    skip: &Arc<AtomicBool>,
    no_sound: bool,
) {
    let total_seconds = duration_mins * 60;
    let progress_bar = ProgressBar::new(total_seconds);
    progress_bar.set_message(format!(
        "{} (#{}/{})",
        work_type, current_cycle, total_cycles
    ));
    progress_bar.set_style(
        indicatif::ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) < {msg} >",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.tick();
    let mut was_paused = false;
    let mut was_skipped = false;
    for remaining_seconds in (1..=total_seconds).rev() {
        // Check if the timer was skipped
        if skip.load(Ordering::SeqCst) {
            progress_bar.set_message("Skipping ⏸️");
            sleep(Duration::from_millis(500));
            skip.store(false, Ordering::SeqCst);
            was_skipped = true;
            break;
        }
        while paused.load(Ordering::SeqCst) {
            if !was_paused {
                progress_bar.set_message("Paused ⏸️");
            }
            was_paused = true;
            thread::sleep(Duration::from_millis(100));
        }
        if was_paused {
            progress_bar.set_message(work_type.to_string());
            progress_bar.reset_eta();
            was_paused = false;
        }
        if remaining_seconds == 10 {
            send_notification(&format!("{}: 00:10s left", work_type));
        }

        sleep(Duration::from_millis(1000));
        progress_bar.inc(1);
    }
    progress_bar.finish_and_clear();

    if !was_skipped {
        let message = match work_type {
            SessionType::Work(_) => "Work session is over. Time for a break! 💪",
            SessionType::ShortBreak(_) => "Break is over. Time to focus! 🧠",
            SessionType::LongBreak(_) => "Long break is over. Time to get back to work! 💼",
        };
        if let SessionType::Work(_) = work_type {
            let _ = POMODORO_COUNTER.fetch_add(1, Ordering::SeqCst);
        }
        send_notification(message);
        if !no_sound {
            play_sound(&sink);
        }
    }
}

/// Run a break timer.
/// @param cycle The current cycle number.
/// @param total_cycles The total number of cycles.
/// @param short_break The duration of the short break in minutes.
/// @param long_break The duration of the long break in minutes.
/// @param sink The audio sink to use for playing sounds.
/// @param paused A flag indicating whether the timer is paused.
/// @param skip A flag indicating whether to skip the timer.
/// @param no_sound A flag indicating whether to disable sound notifications.
fn run_break_timer(
    cycle: u64,
    total_cycles: u64,
    short_break: u64,
    long_break: u64,
    sink: &rodio::Sink,
    paused: &Arc<AtomicBool>,
    skip: &Arc<AtomicBool>,
    no_sound: bool,
) {
    if cycle < total_cycles {
        run_timer(
            short_break,
            &SessionType::ShortBreak("Break time"),
            cycle,
            total_cycles,
            sink,
            paused,
            skip,
            no_sound,
        );
    } else {
        run_timer(
            long_break,
            &SessionType::LongBreak("Long break time"),
            cycle,
            total_cycles,
            sink,
            paused,
            skip,
            no_sound,
        );
    }
}

/// Send a desktop notification.
/// @param message The message to display in the notification.
fn send_notification(message: &str) {
    Notification::new()
        .summary("Pomodoro Timer")
        .body(message)
        .icon("dialog-information")
        .show()
        .expect("Failed to send notification.");
}

/// Check if the reset flag is set and reset it if so.
/// @param reset The reset flag to check.
fn check_reset(reset: &Arc<AtomicBool>) -> ControlFlow<()> {
    if reset.load(Ordering::SeqCst) {
        reset.store(false, Ordering::SeqCst);
        send_notification("Pomodoro cycle has been reset.");
        return ControlFlow::Break(());
    }
    ControlFlow::Continue(())
}

fn main() {
    let config = Config::parse();
    let stream_handle =
        rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
    let sink = rodio::Sink::connect_new(&stream_handle.mixer());
    let paused = Arc::new(AtomicBool::new(false));
    let skip = Arc::new(AtomicBool::new(false));
    let reset = Arc::new(AtomicBool::new(false));

    let paused_clone = Arc::clone(&paused);
    let skip_clone = Arc::clone(&skip);
    let reset_clone = Arc::clone(&reset);
    thread::spawn(move || {
        use crossterm::event::{self, Event, KeyCode};
        // Crossterm needs to be in "raw mode" to read single key events
        crossterm::terminal::enable_raw_mode().unwrap();
        loop {
            // Poll for an event, waiting for up to 1 second
            if event::poll(Duration::from_secs(1)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    // If the user presses 'r', set the reset flag and skip the current timer
                    if key.code == KeyCode::Char('r') {
                        reset_clone.store(true, Ordering::SeqCst);
                        skip_clone.store(true, Ordering::SeqCst);
                    }
                    // If the user presses 's', set the skip flag
                    if key.code == KeyCode::Char('s') {
                        skip_clone.store(true, Ordering::SeqCst);
                    }
                    // If the user presses space or 'p', toggle the paused state
                    if key.code == KeyCode::Char(' ') || key.code == KeyCode::Char('p') {
                        // fetch_xor is a thread-safe way to flip a boolean
                        paused_clone.fetch_xor(true, Ordering::SeqCst);
                    }
                    // Exit on Ctrl+C, Esc, or 'q'
                    if (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c'))
                        || key.code == KeyCode::Esc
                        || key.code == KeyCode::Char('q')
                    {
                        // Restore terminal state
                        crossterm::terminal::disable_raw_mode().unwrap();
                        println!(
                            "\n\rYou've completed {} {}! 🎉 ({})",
                            POMODORO_COUNTER.load(Ordering::SeqCst),
                            match POMODORO_COUNTER.load(Ordering::SeqCst) {
                                0..=1 => "Pomodoro",
                                _ => "Pomodoros",
                            },
                            if POMODORO_COUNTER.load(Ordering::SeqCst) * config.work_duration > 0 {
                                format!(
                                    "{} minutes",
                                    POMODORO_COUNTER.load(Ordering::SeqCst) * config.work_duration
                                )
                            } else {
                                "No time spent".into()
                            }
                        );
                        println!("\n\r{}", "Exiting...".red().bold());
                        std::process::exit(0);
                    }
                }
            }
        }
    });

    println!("Press 'p' or Space to pause/resume, 's' to skip, 'r' to reset a Pomodoro cycle.");
    println!("Press Ctrl+C, Esc, or 'q' to quit at any time.\n");
    println!(
        "Starting Pomodoro: {} minutes work, {} minutes short break, {} minutes long break, {} cycles.\n",
        config.work_duration, config.short_break, config.long_break, config.cycles
    );

    loop {
        let mut current_cycle = 1;
        while current_cycle <= config.cycles {
            run_timer(
                config.work_duration,
                &SessionType::Work("Work session"),
                current_cycle,
                config.cycles,
                &sink,
                &paused,
                &skip,
                config.no_sound,
            );

            if let ControlFlow::Break(_) = check_reset(&reset) {
                continue;
            }

            run_break_timer(
                current_cycle,
                config.cycles,
                config.short_break,
                config.long_break,
                &sink,
                &paused,
                &skip,
                config.no_sound,
            );

            if let ControlFlow::Break(_) = check_reset(&reset) {
                continue;
            }

            current_cycle += 1;
        }
    }
}
