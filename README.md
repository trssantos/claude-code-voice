# Claude Code Voice

A push-to-talk voice input tool for Claude Code and terminal workflows. Hold a hotkey, speak, release — your speech is transcribed locally via Whisper and pasted into the focused input.

## Features

- **Push-to-talk**: Hold a hotkey to record, release to transcribe
- **Local transcription**: Uses Whisper.cpp for fast, offline speech-to-text
- **No cloud dependencies**: Everything runs locally on your machine
- **Auto-paste**: Transcribed text is automatically pasted into the active window
- **Daemon mode**: Runs in the background, ready whenever you need it
- **Cross-platform**: Works on macOS, Linux, and Windows
- **Configurable**: Choose model size (tiny/base/small for speed vs accuracy)

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- A microphone
- System audio libraries (see below)

#### Linux

```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev pkg-config

# Fedora
sudo dnf install alsa-lib-devel

# Arch
sudo pacman -S alsa-lib
```

#### macOS

No additional dependencies required (uses CoreAudio).

#### Windows

No additional dependencies required (uses WASAPI).

### Option 1: Install via cargo (easiest with Rust installed)

Once published to crates.io (recommended):

```bash
cargo install claude-code-voice
```

Or install directly from git:

```bash
cargo install --git https://github.com/trssantos/claude-code-voice
```

**Note:** On Linux, install system dependencies first:
```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev pkg-config
```

This will compile and install the binary to `~/.cargo/bin/` automatically.

### Option 2: One-line install script

```bash
curl -sSL https://raw.githubusercontent.com/trssantos/claude-code-voice/main/install.sh | bash
```

This script will:
- Detect your OS and install system dependencies
- Install Rust if not present
- Build and install the tool
- Add it to your PATH

### Option 3: Download pre-built binary (coming soon)

Pre-built binaries will be available from [GitHub Releases](https://github.com/trssantos/claude-code-voice/releases).

### Option 4: Build from source manually

```bash
git clone https://github.com/trssantos/claude-code-voice.git
cd claude-code-voice
./install-deps.sh  # Install system dependencies
cargo build --release
sudo cp target/release/claude-code-voice /usr/local/bin/
```

## Quick Start

### 1. Download the Whisper model

First time setup - download a Whisper model (base is recommended):

```bash
claude-code-voice download base
```

Available models:
- `tiny`: Fastest, least accurate (~75MB)
- `base`: Good balance of speed and accuracy (~142MB) **recommended**
- `small`: Better accuracy, slower (~466MB)
- `medium`: High accuracy, much slower (~1.5GB)
- `large`: Best accuracy, very slow (~2.9GB)

### 2. Start the daemon

```bash
claude-code-voice start
```

This will start the daemon in the background with the default hotkey `Ctrl+Shift+Space`.

### 3. Use voice input

1. Focus on any text input (terminal, editor, browser, etc.)
2. Press and hold `Ctrl+Shift+Space`
3. Speak your text
4. Release the hotkey
5. Your speech will be transcribed and pasted automatically

### 4. Stop the daemon

```bash
claude-code-voice stop
```

## Usage

### Commands

```bash
# Start the daemon (runs in background)
claude-code-voice start

# Start with custom hotkey
claude-code-voice start --hotkey "ctrl+alt+v"

# Start with different model
claude-code-voice start --model small

# Start in foreground (for debugging)
claude-code-voice start --foreground

# Stop the daemon
claude-code-voice stop

# Check daemon status
claude-code-voice status

# Download a model
claude-code-voice download base
```

### Hotkey Format

Hotkeys are specified as modifier keys plus a main key, separated by `+`:

```
ctrl+shift+space
alt+v
ctrl+alt+r
super+space
```

Supported modifiers:
- `ctrl` or `control`
- `shift`
- `alt`
- `super`, `meta`, `cmd` (Command on macOS), or `win` (Windows key)

Supported keys:
- Letters: `a-z`
- Numbers: `0-9`
- Function keys: `f1-f12`
- Special: `space`, `enter`, `tab`, `backspace`, `escape`

## Configuration

Models are stored in `~/.claude-code-voice/models/`

Logs (when running as daemon) are in `~/.claude-code-voice/logs/`

## Troubleshooting

### Audio device not found

Make sure you have a working microphone connected and configured as the default input device.

### Hotkey not working

1. Make sure another application isn't using the same hotkey
2. Try a different hotkey combination
3. On Linux, you may need to run with elevated privileges for global hotkeys

### Transcription is slow

Use a smaller model:

```bash
claude-code-voice stop
claude-code-voice start --model tiny
```

### Paste not working

The tool simulates `Ctrl+V` (or `Cmd+V` on macOS) to paste. Make sure:
1. The target application supports standard paste shortcuts
2. The application has focus when you release the hotkey

### Viewing logs

When running as a daemon:

```bash
tail -f ~/.claude-code-voice/logs/stdout.log
tail -f ~/.claude-code-voice/logs/stderr.log
```

When running in foreground mode:

```bash
claude-code-voice start --foreground
```

## Development

### Running tests

```bash
cargo test
```

### Building for release

```bash
cargo build --release
```

### Debug mode

Run in foreground with verbose logging:

```bash
RUST_LOG=debug claude-code-voice start --foreground
```

## Architecture

The tool consists of several modules:

- `audio.rs`: Captures audio from the microphone using cpal
- `transcription.rs`: Transcribes audio using Whisper.cpp via whisper-rs
- `hotkey.rs`: Handles global hotkey registration and events
- `clipboard.rs`: Manages clipboard and paste simulation via enigo
- `daemon.rs`: Manages background process lifecycle
- `model.rs`: Downloads and manages Whisper models
- `config.rs`: Configuration management

## Performance

Model performance on typical hardware (approximate):

| Model  | Size   | Speed (real-time factor) | Quality |
|--------|--------|--------------------------|---------|
| tiny   | 75 MB  | 32x faster               | Good    |
| base   | 142 MB | 16x faster               | Better  |
| small  | 466 MB | 6x faster                | Great   |
| medium | 1.5 GB | 2x faster                | Excellent |
| large  | 2.9 GB | 1x (real-time)           | Best    |

*Real-time factor: How much faster than real-time the transcription runs. 32x means 1 second of audio transcribes in ~30ms.*

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.

## Credits

- Built with [whisper-rs](https://github.com/tazz4843/whisper-rs) (Rust bindings for Whisper.cpp)
- Uses [Whisper.cpp](https://github.com/ggerganov/whisper.cpp) by Georgi Gerganov
- Based on [OpenAI's Whisper](https://github.com/openai/whisper) model
