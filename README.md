# Pomodoro

**Pomodoro** is a simple CLI Pomodoro timer written in Rust. Boost your productivity by working in focused intervals with scheduled breaks.

## Features

- Start, pause, and reset Pomodoro sessions
- Customizable work and break durations
- Minimal, distraction-free terminal interface

## Installation

```sh
git clone https://github.com/jclegras/pomodoro.git
cd rustodoro
cargo build --release
```

## Usage

```sh
./pomodoro [OPTIONS]
```

### Options

| Option                       | Description                          | Default |
|------------------------------|--------------------------------------|---------|
| `-w`, `--work <mins>`        | Set work duration in minutes         | 25      |
| `-s`, `--short-break <mins>` | Set break duration in minutes        | 5       |
| `-l`, `--long-break <mins>`  | Set long break duration in minutes   | 15      |
| `-c`, `--cycles <n>`         | Number of Pomodoro cycles            | 4       |
| `-n`, `--no-sound`           | Disable sound notifications          | false   |
| `-h`, `--help`               | Show help message                    |         |

### In-App Controls

While the app is running, you can use the following commands:

- Press **`p`** or **Space** to pause or resume the timer.
- Press **`s`** to skip the current interval.
- Press **`r`** to reset the Pomodoro cycle.
- Press **Ctrl+C**, **Esc**, or **`q`** to quit at any time.

### Example

Start a Pomodoro session with 50-minute work intervals and 10-minute breaks:

```sh
./pomodoro --work 50 --break 10
```

Start a session with custom durations and no sound notifications, running for 6 cycles:

```sh
./pomodoro --work 40 --break 8 --long 20 --cycles 6 --no-sound
```

## How It Works

1. Start a session: Timer counts down your work interval.
2. Take a short break: Timer notifies you when to rest.
3. Repeat: After several cycles, enjoy a longer break.

## Contributing

Contributions are welcome! Please open issues or pull requests.

## License

MIT License
