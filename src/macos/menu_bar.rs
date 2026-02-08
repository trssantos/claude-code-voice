use anyhow::Result;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType};
use foreign_types_shared::ForeignType;
use std::sync::Arc;
use tracing::{error, info};
use tray_icon::{menu::Menu, TrayIconBuilder};

use super::MacOSHotkeyManager;

pub fn run_menu_bar_app(model_size: String, hotkey_str: String) -> Result<()> {
    info!("Starting macOS menu bar app");
    info!("Model: {}, Hotkey: {}", model_size, hotkey_str);

    // Create menu
    let menu = Menu::new();

    // Create tray icon
    let _tray_icon = TrayIconBuilder::new()
        .with_tooltip("Claude Code Voice - Push to talk")
        .with_menu(Box::new(menu))
        .build()?;

    info!("Menu bar icon created");

    // Initialize hotkey manager
    let hotkey_manager = Arc::new(MacOSHotkeyManager::new(&hotkey_str)?);

    // Install event tap for global keyboard monitoring
    install_event_tap(hotkey_manager.clone())?;

    // Initialize components in background thread
    let hotkey_clone = hotkey_manager.clone();
    let model_clone = model_size.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = run_voice_input_loop(hotkey_clone, model_clone).await {
                error!("Voice input loop error: {}", e);
            }
        });
    });

    info!("Starting event loop...");

    // Run the main event loop
    CFRunLoop::run_current();

    Ok(())
}

async fn run_voice_input_loop(
    hotkey_manager: Arc<MacOSHotkeyManager>,
    model_size: String,
) -> Result<()> {
    // Initialize transcriber
    let transcriber = crate::transcription::Transcriber::new(&model_size)?;

    // Initialize audio capturer
    let audio_capturer = crate::audio::AudioCapturer::new()?;

    // Initialize clipboard handler
    let mut clipboard_handler = crate::clipboard::ClipboardHandler::new()?;

    info!("All components initialized. Ready for voice input!");
    info!("Press your hotkey to start recording");

    loop {
        // Wait for hotkey press
        while !hotkey_manager.is_pressed() {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        info!("Recording started");

        // Capture audio while hotkey is held
        let audio_samples = capture_while_pressed(&hotkey_manager, &audio_capturer).await?;

        if audio_samples.is_empty() {
            info!("No audio captured");
            continue;
        }

        info!("Recording stopped. Transcribing...");

        // Transcribe
        let text = transcriber.transcribe(&audio_samples).await?;

        if text.is_empty() {
            info!("No speech detected");
            continue;
        }

        info!("Transcribed: {}", text);

        // Paste
        clipboard_handler.paste_text(&text).await?;
    }
}

async fn capture_while_pressed(
    hotkey_manager: &Arc<MacOSHotkeyManager>,
    audio_capturer: &crate::audio::AudioCapturer,
) -> Result<Vec<f32>> {
    use cpal::traits::StreamTrait;
    use std::sync::{Arc, Mutex};

    let samples = Arc::new(Mutex::new(Vec::new()));
    let samples_clone = samples.clone();

    // Build and start stream
    let stream = audio_capturer.build_stream::<f32>(samples_clone)?;
    stream.play()?;

    // Wait for hotkey release
    while hotkey_manager.is_pressed() {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Stop stream
    drop(stream);

    let captured = samples.lock().unwrap().clone();
    info!("Captured {} audio samples", captured.len());

    Ok(captured)
}

fn install_event_tap(hotkey_manager: Arc<MacOSHotkeyManager>) -> Result<()> {
    let event_types = vec![CGEventType::KeyDown, CGEventType::KeyUp];

    let hotkey_clone = hotkey_manager.clone();

    let callback = move |_proxy: core_graphics::event::CGEventTapProxy,
                         event_type: CGEventType,
                         event: &CGEvent|
          -> Option<CGEvent> {
        match event_type {
            CGEventType::KeyDown => {
                if hotkey_clone.matches_event(event) {
                    hotkey_clone.set_pressed(true);
                }
            }
            CGEventType::KeyUp => {
                if hotkey_clone.matches_event(event) {
                    hotkey_clone.set_pressed(false);
                }
            }
            _ => {}
        }

        // Return the event unmodified
        Some(unsafe { CGEvent::from_ptr(event.as_ptr()) })
    };

    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        event_types,
        callback,
    )
    .map_err(|_| {
        anyhow::anyhow!(
            "Failed to create event tap. Make sure Accessibility permissions are granted.\n\
             Go to System Settings → Privacy & Security → Accessibility and add Terminal.app"
        )
    })?;

    let loop_source = tap
        .mach_port
        .create_runloop_source(0)
        .map_err(|e| anyhow::anyhow!("Failed to create run loop source: {}", e))?;

    let current_loop = CFRunLoop::get_current();
    current_loop.add_source(&loop_source, unsafe { kCFRunLoopCommonModes });
    tap.enable();

    info!("Event tap installed successfully");

    // Prevent the tap from being dropped
    std::mem::forget(tap);
    std::mem::forget(hotkey_manager);

    Ok(())
}
