// macOS-specific menu bar app implementation
mod menu_bar;
mod hotkey_macos;

pub use menu_bar::run_menu_bar_app;
pub use hotkey_macos::MacOSHotkeyManager;
