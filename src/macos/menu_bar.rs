use anyhow::Result;
use core_graphics::event::{CGEvent, CGEventTap, CGEventTapCallBack, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType};
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use std::sync::{Arc, Mutex};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::menu::{MenuBar, MenuItem};
use tracing::{error, info};
use tray_icon::{TrayIcon, TrayIconBuilder};

use super::MacOSHotkeyManager;

pub fn run_menu_bar_app(model_size: String, hotkey_str: String) -> Result<()> {
    info!("Starting macOS menu bar app");
    info!("Model: {}, Hotkey: {}", model_size, hotkey_str);

    // Create event loop
    let event_loop = EventLoop::new();

    // Create menu bar
    let mut menu = MenuBar::new();
    menu.add_native_item(MenuItem::About("Claude Code Voice".to_string()));
    menu.add_native_item(MenuItem::Separator);
    menu.add_native_item(MenuItem::Quit);

    // Create tray icon
    let tray_icon = TrayIconBuilder::new()
        .with_tooltip("Claude Code Voice")
        .with_menu(Box::new(menu))
        .build()?;

    info!("Menu bar icon created");

    // Initialize hotkey manager
    let hotkey_manager = Arc::new(MacOSHotkeyManager::new(&hotkey_str)?);

    // Install event tap for global keyboard monitoring
    install_event_tap(hotkey_manager.clone())?;

    // Initialize transcriber (in background)
    let transcriber = Arc::new(Mutex::new(
        crate::transcription::Transcriber::new(&model_size)?
    ));

    // Initialize audio capturer
    let audio_capturer = Arc::new(Mutex::new(crate::audio::AudioCapturer::new()?));

    // Initialize clipboard handler
    let clipboard_handler = Arc::new(Mutex::new(crate::clipboard::ClipboardHandler::new()?));

    info!("All components initialized. Ready for voice input!");

    // Start monitoring hotkey in background thread
    let hotkey_clone = hotkey_manager.clone();
    let transcriber_clone = transcriber.clone();
    let audio_clone = audio_capturer.clone();
    let clipboard_clone = clipboard_handler.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            monitor_hotkey_and_record(
                hotkey_clone,
                transcriber_clone,
                audio_clone,
                clipboard_clone,
            ).await
        });
    });

    // Run event loop
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
    });
}

async fn monitor_hotkey_and_record(
    hotkey_manager: Arc<MacOSHotkeyManager>,
    transcriber: Arc<Mutex<crate::transcription::Transcriber>>,
    audio_capturer: Arc<Mutex<crate::audio::AudioCapturer>>,
    mut clipboard_handler: Arc<Mutex<crate::clipboard::ClipboardHandler>>,
) {
    loop {
        // Wait for hotkey press
        while !hotkey_manager.is_pressed() {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        info!("Recording started");

        // Capture audio while hotkey is held
        // We need a custom implementation here since we can't use wait_for_release
        let audio_samples = capture_while_pressed(&hotkey_manager, &audio_capturer).await;

        if let Err(e) = audio_samples {
            error!("Audio capture error: {}", e);
            continue;
        }

        let audio_data = audio_samples.unwrap();

        if audio_data.is_empty() {
            info!("No audio captured");
            continue;
        }

        info!("Recording stopped. Transcribing...");

        // Transcribe
        let text = {
            let transcriber = transcriber.lock().unwrap();
            transcriber.transcribe(&audio_data).await
        };

        if let Err(e) = text {
            error!("Transcription error: {}", e);
            continue;
        }

        let text = text.unwrap();

        if text.is_empty() {
            info!("No speech detected");
            continue;
        }

        info!("Transcribed: {}", text);

        // Paste
        let mut clipboard = clipboard_handler.lock().unwrap();
        if let Err(e) = clipboard.paste_text(&text).await {
            error!("Paste error: {}", e);
        }
    }
}

async fn capture_while_pressed(
    hotkey_manager: &Arc<MacOSHotkeyManager>,
    audio_capturer: &Arc<Mutex<crate::audio::AudioCapturer>>,
) -> Result<Vec<f32>> {
    use cpal::traits::StreamTrait;
    use std::sync::{Arc as StdArc, Mutex as StdMutex};

    let samples = StdArc::new(StdMutex::new(Vec::new()));
    let samples_clone = samples.clone();

    let capturer = audio_capturer.lock().unwrap();

    // Build and start stream
    let stream = capturer.build_stream::<f32>(samples_clone)?;
    stream.play()?;

    // Wait for hotkey release
    while hotkey_manager.is_pressed() {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Stop stream
    drop(stream);

    let captured = samples.lock().unwrap().clone();
    Ok(captured)
}

fn install_event_tap(hotkey_manager: Arc<MacOSHotkeyManager>) -> Result<()> {
    unsafe {
        let event_mask = CGEventType::KeyDown as u64 | CGEventType::KeyUp as u64;

        let callback: CGEventTapCallBack = {
            let hotkey_clone = hotkey_manager.clone();

            Box::new(move |_proxy, event_type, event, _user_info| {
                let event_type_value = event_type as u32;

                if event_type_value == CGEventType::KeyDown as u32 {
                    if hotkey_clone.matches_event(&event) {
                        hotkey_clone.set_pressed(true);
                    }
                } else if event_type_value == CGEventType::KeyUp as u32 {
                    if hotkey_clone.matches_event(&event) {
                        hotkey_clone.set_pressed(false);
                    }
                }

                event
            })
        };

        let tap = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            event_mask,
            callback,
        ).ok_or_else(|| anyhow::anyhow!("Failed to create event tap. Make sure Accessibility permissions are granted."))?;

        let loop_source = tap.mach_port.create_runloop_source(0)?;
        let current_loop = CFRunLoop::get_current();
        current_loop.add_source(&loop_source, unsafe { kCFRunLoopCommonModes });
        tap.enable();

        info!("Event tap installed successfully");
    }

    Ok(())
}
