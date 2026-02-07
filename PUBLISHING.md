# Publishing to Package Repositories

This guide explains how to publish `claude-code-voice` to various package managers so users can install with commands like `brew install`, `apt install`, etc.

## 1. Publish to crates.io (Rust's package registry)

**Prerequisites:**
- Create account at [crates.io](https://crates.io/)
- Get API token from account settings

**Steps:**
```bash
# Login to crates.io
cargo login <your-api-token>

# Publish (first time)
cargo publish

# Future updates (bump version in Cargo.toml first)
cargo publish
```

After publishing, users can install with:
```bash
cargo install claude-code-voice
```

---

## 2. Homebrew (macOS/Linux)

Homebrew is the most popular package manager for macOS.

### Option A: Tap Repository (Recommended)

1. **Create a Homebrew tap repository:**
   ```bash
   # Create a new repo: homebrew-claude-code-voice
   # Repository name MUST start with "homebrew-"
   ```

2. **Create Formula file** (`claude-code-voice.rb`):
   ```ruby
   class ClaudeCodeVoice < Formula
     desc "Push-to-talk voice input for Claude Code"
     homepage "https://github.com/trssantos/claude-code-voice"
     url "https://github.com/trssantos/claude-code-voice/archive/refs/tags/v0.1.0.tar.gz"
     sha256 "REPLACE_WITH_ACTUAL_SHA256"
     license "MIT"

     depends_on "rust" => :build
     depends_on "pkg-config" => :build

     def install
       system "cargo", "install", *std_cargo_args
     end

     test do
       assert_match "claude-code-voice", shell_output("#{bin}/claude-code-voice --version")
     end
   end
   ```

3. **Generate SHA256:**
   ```bash
   # Create a release tag first
   git tag v0.1.0
   git push origin v0.1.0

   # Download the archive and get SHA256
   curl -L https://github.com/trssantos/claude-code-voice/archive/refs/tags/v0.1.0.tar.gz | shasum -a 256
   ```

4. **Users install with:**
   ```bash
   brew tap trssantos/claude-code-voice
   brew install claude-code-voice
   ```

### Option B: Submit to Homebrew Core

For wider distribution, submit to the main Homebrew repository:

1. Fork [homebrew-core](https://github.com/Homebrew/homebrew-core)
2. Add your formula to `Formula/c/claude-code-voice.rb`
3. Test it: `brew install --build-from-source Formula/c/claude-code-voice.rb`
4. Submit a PR to homebrew-core

**Note:** Homebrew Core has strict requirements. Read their [contribution guide](https://docs.brew.sh/How-To-Open-a-Homebrew-Pull-Request).

---

## 3. APT (Ubuntu/Debian)

### Option A: Personal PPA (Ubuntu only)

1. **Create Launchpad account:** [launchpad.net](https://launchpad.net/)

2. **Create PPA:**
   - Go to your profile → Create a new PPA
   - Name: `claude-code-voice`

3. **Build Debian package:**

   Create `debian/` directory with control files:
   ```bash
   mkdir -p debian
   ```

   `debian/control`:
   ```
   Source: claude-code-voice
   Section: utils
   Priority: optional
   Maintainer: Your Name <your@email.com>
   Build-Depends: debhelper (>= 11), cargo, libasound2-dev, pkg-config
   Standards-Version: 4.5.0

   Package: claude-code-voice
   Architecture: any
   Depends: ${shlibs:Depends}, ${misc:Depends}
   Description: Push-to-talk voice input for Claude Code
    Local speech-to-text transcription using Whisper.
   ```

   `debian/rules`:
   ```makefile
   #!/usr/bin/make -f
   %:
       dh $@

   override_dh_auto_build:
       cargo build --release

   override_dh_auto_install:
       install -D -m 755 target/release/claude-code-voice debian/claude-code-voice/usr/bin/claude-code-voice
   ```

4. **Upload to PPA:**
   ```bash
   debuild -S -sd
   dput ppa:your-username/claude-code-voice ../claude-code-voice_0.1.0_source.changes
   ```

5. **Users install with:**
   ```bash
   sudo add-apt-repository ppa:your-username/claude-code-voice
   sudo apt update
   sudo apt install claude-code-voice
   ```

### Option B: Host your own APT repository

1. **Build the .deb package** (as above)

2. **Create repository structure:**
   ```bash
   mkdir -p repo/pool/main
   cp ../claude-code-voice_0.1.0_amd64.deb repo/pool/main/
   ```

3. **Generate Packages file:**
   ```bash
   cd repo
   dpkg-scanpackages pool/main /dev/null | gzip -9c > dists/stable/main/binary-amd64/Packages.gz
   ```

4. **Host on GitHub Pages or your server**

5. **Users add your repo:**
   ```bash
   echo "deb https://yourusername.github.io/claude-code-voice-repo stable main" | sudo tee /etc/apt/sources.list.d/claude-code-voice.list
   sudo apt update
   sudo apt install claude-code-voice
   ```

---

## 4. AUR (Arch Linux)

1. **Create PKGBUILD:**
   ```bash
   # Maintainer: Your Name <your@email.com>
   pkgname=claude-code-voice
   pkgver=0.1.0
   pkgrel=1
   pkgdesc="Push-to-talk voice input for Claude Code"
   arch=('x86_64')
   url="https://github.com/trssantos/claude-code-voice"
   license=('MIT')
   depends=('alsa-lib')
   makedepends=('rust' 'cargo')
   source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
   sha256sums=('REPLACE_WITH_SHA256')

   build() {
       cd "$pkgname-$pkgver"
       cargo build --release --locked
   }

   package() {
       cd "$pkgname-$pkgver"
       install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
       install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
   }
   ```

2. **Test locally:**
   ```bash
   makepkg -si
   ```

3. **Submit to AUR:**
   ```bash
   # Clone AUR repo (create SSH key first)
   git clone ssh://aur@aur.archlinux.org/claude-code-voice.git
   cd claude-code-voice
   cp ../PKGBUILD .
   makepkg --printsrcinfo > .SRCINFO
   git add PKGBUILD .SRCINFO
   git commit -m "Initial commit"
   git push
   ```

4. **Users install with:**
   ```bash
   yay -S claude-code-voice
   # or
   paru -S claude-code-voice
   ```

---

## 5. Chocolatey (Windows)

1. **Create account:** [chocolatey.org](https://chocolatey.org/)

2. **Create package files:**

   `claude-code-voice.nuspec`:
   ```xml
   <?xml version="1.0"?>
   <package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
     <metadata>
       <id>claude-code-voice</id>
       <version>0.1.0</version>
       <title>Claude Code Voice</title>
       <authors>Your Name</authors>
       <description>Push-to-talk voice input for Claude Code</description>
       <projectUrl>https://github.com/trssantos/claude-code-voice</projectUrl>
       <tags>voice speech-to-text cli</tags>
       <licenseUrl>https://github.com/trssantos/claude-code-voice/blob/main/LICENSE</licenseUrl>
     </metadata>
   </package>
   ```

   `tools/chocolateyinstall.ps1`:
   ```powershell
   $packageName = 'claude-code-voice'
   $url64 = 'https://github.com/trssantos/claude-code-voice/releases/download/v0.1.0/claude-code-voice-windows-x86_64.exe'

   Install-ChocolateyPackage $packageName 'exe' '/S' $url64
   ```

3. **Submit package:**
   ```bash
   choco pack
   choco push claude-code-voice.0.1.0.nupkg --source https://push.chocolatey.org/
   ```

4. **Users install with:**
   ```powershell
   choco install claude-code-voice
   ```

---

## Quick Start Recommendations

For the easiest user experience, I recommend this order:

1. **Start with crates.io** - Takes 5 minutes, works immediately
   ```bash
   cargo publish
   ```

2. **Create Homebrew tap** - Easy to maintain, popular on macOS
   - Create repo: `homebrew-claude-code-voice`
   - Add formula
   - Users: `brew tap trssantos/claude-code-voice && brew install claude-code-voice`

3. **Submit to AUR** - Easy, Arch users love it
   - Create PKGBUILD
   - Submit to AUR

4. **Later:** APT PPA and Chocolatey (more complex setup)

---

## Updating Packages

When you release a new version:

1. Update version in `Cargo.toml`
2. Create git tag: `git tag v0.1.1 && git push origin v0.1.1`
3. Update each package:
   - **crates.io:** `cargo publish`
   - **Homebrew:** Update formula with new version/SHA256
   - **AUR:** Update PKGBUILD
   - **APT:** Upload new .deb to PPA
   - **Chocolatey:** Update and push new .nupkg
