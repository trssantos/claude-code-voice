use anyhow::{Context, Result};
use cocoa::appkit::NSEvent;
use cocoa::base::{id, nil};
use cocoa::foundation::NSAutoreleasePool;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType, EventField};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, info};

pub struct MacOSHotkeyManager {
    is_pressed: Arc<AtomicBool>,
    key_code: u16,
    modifiers: CGEventFlags,
}

impl MacOSHotkeyManager {
    pub fn new(hotkey_str: &str) -> Result<Self> {
        let (key_code, modifiers) = parse_hotkey_macos(hotkey_str)?;

        info!("Registering macOS hotkey: {} (keycode: {}, modifiers: {:?})",
              hotkey_str, key_code, modifiers);

        Ok(Self {
            is_pressed: Arc::new(AtomicBool::new(false)),
            key_code,
            modifiers,
        })
    }

    pub fn is_pressed(&self) -> bool {
        self.is_pressed.load(Ordering::SeqCst)
    }

    pub fn set_pressed(&self, pressed: bool) {
        self.is_pressed.store(pressed, Ordering::SeqCst);
        if pressed {
            info!("Hotkey pressed!");
        } else {
            info!("Hotkey released!");
        }
    }

    pub fn matches_event(&self, event: &CGEvent) -> bool {
        let event_keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
        let event_flags = event.get_flags();

        debug!("Event: keycode={}, flags={:?}", event_keycode, event_flags);

        // Check if keycode matches
        if event_keycode != self.key_code as i64 {
            return false;
        }

        // Check if modifiers match (ignore caps lock and other non-modifier flags)
        let relevant_flags = CGEventFlags::CGEventFlagCommand
            | CGEventFlags::CGEventFlagShift
            | CGEventFlags::CGEventFlagControl
            | CGEventFlags::CGEventFlagAlternate;

        let event_modifiers = event_flags & relevant_flags;
        event_modifiers == self.modifiers
    }
}

fn parse_hotkey_macos(hotkey_str: &str) -> Result<(u16, CGEventFlags)> {
    let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();

    if parts.is_empty() {
        anyhow::bail!("Invalid hotkey string: {}", hotkey_str);
    }

    let mut modifiers = CGEventFlags::empty();
    let mut key_code = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= CGEventFlags::CGEventFlagControl,
            "shift" => modifiers |= CGEventFlags::CGEventFlagShift,
            "alt" | "option" => modifiers |= CGEventFlags::CGEventFlagAlternate,
            "super" | "cmd" | "command" => modifiers |= CGEventFlags::CGEventFlagCommand,
            key => {
                key_code = Some(parse_key_code_macos(key)?);
            }
        }
    }

    let key_code = key_code.context("No key code found in hotkey string")?;
    Ok((key_code, modifiers))
}

fn parse_key_code_macos(key: &str) -> Result<u16> {
    // macOS virtual key codes
    let code = match key.to_lowercase().as_str() {
        "a" => 0x00,
        "b" => 0x0B,
        "c" => 0x08,
        "d" => 0x02,
        "e" => 0x0E,
        "f" => 0x03,
        "g" => 0x05,
        "h" => 0x04,
        "i" => 0x22,
        "j" => 0x26,
        "k" => 0x28,
        "l" => 0x25,
        "m" => 0x2E,
        "n" => 0x2D,
        "o" => 0x1F,
        "p" => 0x23,
        "q" => 0x0C,
        "r" => 0x0F,
        "s" => 0x01,
        "t" => 0x11,
        "u" => 0x20,
        "v" => 0x09,
        "w" => 0x0D,
        "x" => 0x07,
        "y" => 0x10,
        "z" => 0x06,
        "0" => 0x1D,
        "1" => 0x12,
        "2" => 0x13,
        "3" => 0x14,
        "4" => 0x15,
        "5" => 0x17,
        "6" => 0x16,
        "7" => 0x1A,
        "8" => 0x1C,
        "9" => 0x19,
        "space" => 0x31,
        "enter" | "return" => 0x24,
        "tab" => 0x30,
        "escape" | "esc" => 0x35,
        "backspace" => 0x33,
        _ => anyhow::bail!("Unknown key: {}", key),
    };

    Ok(code)
}
