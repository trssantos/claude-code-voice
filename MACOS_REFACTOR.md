# macOS App Bundle Refactor Plan

## Problem
The current CLI approach doesn't work on macOS because:
- `global-hotkey` crate requires a proper event loop
- macOS blocks keyboard monitoring from CLI apps
- No NSApplication event loop = no global hotkeys

## Solution
Create a proper macOS .app bundle with:
- Menu bar icon (system tray)
- Native macOS hotkey registration
- Runs as a background app
- Installable via Homebrew Cask

## Implementation Plan

### Phase 1: Create Menu Bar App Structure

**Dependencies:**
```toml
[dependencies]
# macOS app framework
tao = { version = "0.16", features = ["tray"] }
tray-icon = "0.14"

# Native macOS hotkeys
rdev = "0.5"  # Works from apps with event loop

# Or use native Cocoa:
cocoa = "0.25"
objc = "0.2"
core-graphics = "0.23"
```

**App Structure:**
```
claude-code-voice.app/
├── Contents/
│   ├── Info.plist
│   ├── MacOS/
│   │   └── claude-code-voice (binary)
│   └── Resources/
│       └── icon.icns
```

### Phase 2: Menu Bar Implementation

1. **Create menu bar icon** (microphone icon)
2. **Menu items:**
   - Start/Stop Recording
   - Settings
   - Quit

3. **Status indicator:**
   - Gray: Idle
   - Red: Recording
   - Green: Transcribing

### Phase 3: Native Hotkey Registration

Use macOS Carbon or Cocoa APIs for global hotkeys:

```rust
use cocoa::appkit::NSEvent;
use cocoa::base::{id, nil};
use core_graphics::event::CGEventFlags;

// Register global hotkey using Carbon
// This actually works from .app bundles!
```

### Phase 4: Homebrew Cask Distribution

Create a `.rb` formula:

```ruby
cask "claude-code-voice" do
  version "0.1.0"

  url "https://github.com/trssantos/claude-code-voice/releases/download/v#{version}/claude-code-voice-macos.dmg"
  name "Claude Code Voice"
  desc "Push-to-talk voice input for Claude Code"
  homepage "https://github.com/trssantos/claude-code-voice"

  app "claude-code-voice.app"

  postflight do
    system_command "/usr/bin/open",
                   args: ["-a", "claude-code-voice"]
  end
end
```

**Install:**
```bash
brew install --cask claude-code-voice
```

### Phase 5: Build Process

**Build script:**
```bash
# Build release binary
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create universal binary
lipo -create \
  target/x86_64-apple-darwin/release/claude-code-voice \
  target/aarch64-apple-darwin/release/claude-code-voice \
  -output claude-code-voice-universal

# Create .app bundle
mkdir -p claude-code-voice.app/Contents/MacOS
mkdir -p claude-code-voice.app/Contents/Resources

cp claude-code-voice-universal claude-code-voice.app/Contents/MacOS/claude-code-voice
cp Info.plist claude-code-voice.app/Contents/
cp icon.icns claude-code-voice.app/Contents/Resources/

# Create DMG
create-dmg \
  --volname "Claude Code Voice" \
  --window-pos 200 120 \
  --window-size 600 400 \
  claude-code-voice.dmg \
  claude-code-voice.app
```

## Benefits

1. **Works properly on macOS** - Native hotkeys actually work
2. **Better UX** - Menu bar icon, visual feedback
3. **Easy install** - `brew install --cask claude-code-voice`
4. **Auto-start** - Can launch on login
5. **Native feel** - Behaves like other macOS apps

## Timeline

- [ ] Phase 1: Menu bar app (2-3 hours)
- [ ] Phase 2: Hotkey registration (1-2 hours)
- [ ] Phase 3: Bundle creation (1 hour)
- [ ] Phase 4: Homebrew cask (1 hour)
- [ ] Phase 5: Testing and polish (2 hours)

**Total: ~8 hours of development**

## Notes

- This will be macOS-specific (drop Linux/Windows support or maintain separate CLI version)
- Requires code signing for distribution (can be done later)
- Accessibility permissions still required, but will be prompted automatically
