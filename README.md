# RTL-SDR Radio

> As of writing this (July 22nd, 2024), this app is a minimalistic UI for listening to FM and HD Radio on your RTL-SDR. However, I envision this project to evolve and bundle all the tools into a nice user interface to make the most out of your RTL-SDR.

RTL-SDR Radio is your one-stop shop for listening to the radio frequencies in the air. Using this app, you can listen to any [HD Radio Station](https://hdradio.com/stations/) or [FM Radio Station](https://radio-locator.com/) in your area!

## Installation

> Note: I have been having issues with building through GitHub Actions, so the GitHub Releases have HD Radio functionality disabled (FM Radio still works though!). I recommend [compiling from source](#compiling-from-source) if you want to listen to HD Radio.

Installation should be as simple as going to the [GitHub Releases](https://github.com/njfdev/rtlsdr-radio/releases) and downloading the most recent application from the "Assets" dropdown for your specific OS.

## Compiling from Source

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
brew install cmake autoconf automake libtool git librtlsdr libao fftw soapyrtlsdr

# Debian/Ubuntu Based Linux OSes
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.0-dev librsvg2-dev patchelf git build-essential cmake autoconf automake libtool libao-dev libfftw3-dev librtlsdr-dev nodejs npm libsoapysdr-dev soapysdr-module-rtlsdr
```

Then, clone the git repository and build with `tauri`:

```bash
git clone https://github.com/njfdev/rtlsdr-radio.git
cd rtlsdr-radio
sudo npm install --global yarn
yarn install
cargo install tauri-cli
cargo tauri build
```

Optionally, you can build without HD Radio functionality (if building is causing issues). Just replace the last command with this one:

```bash
NEXT_PUBLIC_EXCLUDE_SIDECAR=true cargo tauri build
```
