use anyhow::{Context, Result};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
    is_pressed: Arc<AtomicBool>,
}

impl HotkeyManager {
    pub fn new(hotkey_str: &str) -> Result<Self> {
        let manager = GlobalHotKeyManager::new().context("Failed to initialize hotkey manager")?;

        let hotkey = parse_hotkey(hotkey_str)?;

        info!("Registering hotkey: {}", hotkey_str);
        manager
            .register(hotkey)
            .context("Failed to register hotkey")?;

        Ok(Self {
            manager,
            hotkey,
            is_pressed: Arc::new(AtomicBool::new(false)),
        })
    }

    pub async fn wait_for_press(&self) -> Result<()> {
        loop {
            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.id == self.hotkey.id() && event.state == global_hotkey::HotKeyState::Pressed
                {
                    self.is_pressed.store(true, Ordering::SeqCst);
                    return Ok(());
                }
            }
            sleep(Duration::from_millis(10)).await;
        }
    }

    pub async fn wait_for_release(&self) -> Result<()> {
        loop {
            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                if event.id == self.hotkey.id()
                    && event.state == global_hotkey::HotKeyState::Released
                {
                    self.is_pressed.store(false, Ordering::SeqCst);
                    return Ok(());
                }
            }

            // Also check if the key is no longer pressed (fallback)
            if !self.is_pressed.load(Ordering::SeqCst) {
                return Ok(());
            }

            sleep(Duration::from_millis(10)).await;
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.is_pressed.load(Ordering::SeqCst)
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        if let Err(e) = self.manager.unregister(self.hotkey) {
            warn!("Failed to unregister hotkey: {}", e);
        }
    }
}

fn parse_hotkey(hotkey_str: &str) -> Result<HotKey> {
    let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();

    if parts.is_empty() {
        anyhow::bail!("Invalid hotkey string: {}", hotkey_str);
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code = None;

    for part in parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" => modifiers |= Modifiers::ALT,
            "super" | "meta" | "cmd" | "win" => modifiers |= Modifiers::SUPER,
            key => {
                key_code = Some(parse_key_code(key)?);
            }
        }
    }

    let key_code = key_code.context("No key code found in hotkey string")?;

    Ok(HotKey::new(Some(modifiers), key_code))
}

fn parse_key_code(key: &str) -> Result<Code> {
    let code = match key.to_lowercase().as_str() {
        "space" => Code::Space,
        "enter" | "return" => Code::Enter,
        "tab" => Code::Tab,
        "backspace" => Code::Backspace,
        "escape" | "esc" => Code::Escape,
        "a" => Code::KeyA,
        "b" => Code::KeyB,
        "c" => Code::KeyC,
        "d" => Code::KeyD,
        "e" => Code::KeyE,
        "f" => Code::KeyF,
        "g" => Code::KeyG,
        "h" => Code::KeyH,
        "i" => Code::KeyI,
        "j" => Code::KeyJ,
        "k" => Code::KeyK,
        "l" => Code::KeyL,
        "m" => Code::KeyM,
        "n" => Code::KeyN,
        "o" => Code::KeyO,
        "p" => Code::KeyP,
        "q" => Code::KeyQ,
        "r" => Code::KeyR,
        "s" => Code::KeyS,
        "t" => Code::KeyT,
        "u" => Code::KeyU,
        "v" => Code::KeyV,
        "w" => Code::KeyW,
        "x" => Code::KeyX,
        "y" => Code::KeyY,
        "z" => Code::KeyZ,
        "0" => Code::Digit0,
        "1" => Code::Digit1,
        "2" => Code::Digit2,
        "3" => Code::Digit3,
        "4" => Code::Digit4,
        "5" => Code::Digit5,
        "6" => Code::Digit6,
        "7" => Code::Digit7,
        "8" => Code::Digit8,
        "9" => Code::Digit9,
        "f1" => Code::F1,
        "f2" => Code::F2,
        "f3" => Code::F3,
        "f4" => Code::F4,
        "f5" => Code::F5,
        "f6" => Code::F6,
        "f7" => Code::F7,
        "f8" => Code::F8,
        "f9" => Code::F9,
        "f10" => Code::F10,
        "f11" => Code::F11,
        "f12" => Code::F12,
        _ => anyhow::bail!("Unknown key code: {}", key),
    };

    Ok(code)
}
