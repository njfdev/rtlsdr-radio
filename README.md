# RTL-SDR Radio

RTL-SDR Radio is your one-stop shop for listening to the radio frequencies in the air! It is designed to be lightweight, easy-to-use, and minimalistic.

## Features

- Listen to FM Radio
  - Decoding RBDS (Radio Broadcast Data System)
- Listen to AM Radio
- Save Radio Stations to Listen to Later
- 🔋 "Batteries Included" - No need to install anything else! Everything comes bundled within the app.

## Installation

Installation should be as simple as going to the [GitHub Releases](https://github.com/njfdev/rtlsdr-radio/releases) and downloading the most recent application from the "Assets" dropdown for your specific OS.

There is one extra step if you get an error like: `"RTL-SDR Radio" is damaged and can't be opened. You should move it to the Trash.`. I, [njfdev](https://github.com/njfdev), do not have an Apple Developer Account so I cannot sign/notarize the app. This means your Mac will automatically move RTL-SDR Radio to the quarantine, so you will need to remove it from quarantine:

```zsh
# ONLY on MacOS
xattr -d com.apple.quarantine /Applications/RTL-SDR\ Radio.app
```

## Compiling from Source

> Note: A recent upgrade to Tauri V2 has caused HD Radio to stop working. It will probably not work.

> ⚠️ Building on Windows is not tested so there are no instructions to do so.

First, install Rust if it is not already installed:

```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

Then, make sure to install the prerequisites:

```bash
# MacOS with brew
brew tap pothosware/homebrew-pothos
brew update
brew install cmake autoconf automake libtool git librtlsdr libao fftw soapyrtlsdr libusb

# Debian/Ubuntu Based Linux OSes
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev librsvg2-dev patchelf git build-essential cmake autoconf automake libtool libao-dev libfftw3-dev librtlsdr-dev nodejs npm libsoapysdr-dev soapysdr-module-rtlsdr libusb-dev libusb-1.0-0-dev curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev libclang-dev libudev-dev
```

Then, clone the git repository and build with `tauri`:

```bash
git clone https://github.com/njfdev/rtlsdr-radio.git
cd rtlsdr-radio
sudo npm install --global yarn
yarn install
cargo install tauri-cli --version "^2.0.0-rc"
NEXT_PUBLIC_EXCLUDE_SIDECAR=true cargo tauri build
```

Optionally, you can build with HD Radio functionality, but it might cause issues. Just replace the last command with this one:

```bash
NEXT_PUBLIC_EXCLUDE_SIDECAR=false cargo tauri build
```
