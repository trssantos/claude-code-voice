mod audio;
mod clipboard;
mod config;
mod model;
mod transcription;

// Platform-specific modules
#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
mod daemon;

#[cfg(not(target_os = "macos"))]
mod hotkey;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber;

fn default_hotkey() -> String {
    #[cfg(target_os = "macos")]
    {
        "super+shift+v".to_string()
    }
    #[cfg(not(target_os = "macos"))]
    {
        "ctrl+shift+space".to_string()
    }
}

#[derive(Parser)]
#[command(name = "claude-code-voice")]
#[command(about = "Push-to-talk voice input for Claude Code and terminal workflows")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the voice input daemon
    Start {
        /// Run in foreground mode for debugging
        #[arg(short, long)]
        foreground: bool,

        /// Whisper model size (tiny, base, small, medium, large)
        #[arg(short, long, default_value = "base")]
        model: String,

        /// Global hotkey - Format: "modifier+key" (e.g., "super+shift+v")
        ///
        /// Modifiers: ctrl, shift, alt, super (⌘ on Mac, ⊞ on Windows)
        /// Keys: a-z, 0-9, f1-f12, space, enter, tab, etc.
        ///
        /// Examples:
        ///   macOS:    --hotkey "super+shift+v"  (⌘+Shift+V)
        ///   Linux:    --hotkey "ctrl+alt+v"
        ///   Windows:  --hotkey "ctrl+shift+v"
        #[arg(short = 'k', long, default_value_t = default_hotkey())]
        hotkey: String,
    },

    /// Stop the voice input daemon
    Stop,

    /// Check daemon status
    Status,

    /// Download Whisper model
    Download {
        /// Model size (tiny, base, small, medium, large)
        #[arg(default_value = "base")]
        model: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            foreground: _,
            model,
            hotkey,
        } => {
            #[cfg(target_os = "macos")]
            {
                info!("Starting macOS menu bar app");

                // Ensure model is downloaded
                if !model::is_model_downloaded(&model).await? {
                    info!("Model '{}' not found. Downloading...", model);
                    model::download_model(&model).await?;
                }

                // On macOS, ignore foreground flag - always run as menu bar app
                macos::run_menu_bar_app(model, hotkey)?;
            }

            #[cfg(not(target_os = "macos"))]
            {
                info!("Starting claude-code-voice daemon");

                // Ensure model is downloaded
                if !model::is_model_downloaded(&model).await? {
                    info!("Model '{}' not found. Downloading...", model);
                    model::download_model(&model).await?;
                }

                let config = config::Config {
                    model_size: model,
                    hotkey,
                    foreground,
                };

                if foreground {
                    info!("Running in foreground mode");
                    run_voice_input(config).await?;
                } else {
                    info!("Starting daemon");
                    daemon::start_daemon(config).await?;
                }
            }
        }

        Commands::Stop => {
            #[cfg(target_os = "macos")]
            {
                println!("On macOS, use the menu bar icon to quit the app");
            }

            #[cfg(not(target_os = "macos"))]
            {
                info!("Stopping claude-code-voice daemon");
                daemon::stop_daemon().await?;
            }
        }

        Commands::Status => {
            #[cfg(target_os = "macos")]
            {
                println!("On macOS, the app runs as a menu bar application");
                println!("Look for the icon in your menu bar");
            }

            #[cfg(not(target_os = "macos"))]
            {
                let status = daemon::get_status().await?;
                println!("{}", status);
            }
        }

        Commands::Download { model } => {
            info!("Downloading Whisper model: {}", model);
            model::download_model(&model).await?;
            info!("Model downloaded successfully");
        }
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
async fn run_voice_input(config: config::Config) -> Result<()> {
    info!("Initializing voice input with model: {}", config.model_size);

    // Initialize transcription engine
    let transcriber = transcription::Transcriber::new(&config.model_size)?;

    // Initialize audio capture
    let audio_capturer = audio::AudioCapturer::new()?;

    // Initialize clipboard handler
    let mut clipboard_handler = clipboard::ClipboardHandler::new()?;

    // Set up hotkey
    let hotkey_manager = hotkey::HotkeyManager::new(&config.hotkey)?;

    info!("Voice input ready. Press {} to start recording", config.hotkey);

    // Main event loop
    loop {
        // Wait for hotkey press
        hotkey_manager.wait_for_press().await?;
        info!("Recording started");

        // Capture audio while hotkey is held
        let audio_data = audio_capturer.capture_while_held(&hotkey_manager).await?;

        if audio_data.is_empty() {
            info!("No audio captured");
            continue;
        }

        info!("Recording stopped. Transcribing...");

        // Transcribe audio
        let text = transcriber.transcribe(&audio_data).await?;

        if text.is_empty() {
            info!("No speech detected");
            continue;
        }

        info!("Transcribed: {}", text);

        // Paste into active window
        clipboard_handler.paste_text(&text).await?;
    }
}
