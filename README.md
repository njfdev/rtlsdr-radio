# RTL-SDR Radio

> As of writing this (July 16th, 2024), this is just a simple GUI wrapper for the `nrsc5` CLI. However, I envision this project to evolve and bundle all the tools into a nice user interface to make the most out of your RTL-SDR.

RTL-SDR Radio is your one-stop shop for listening to the radio frequencies in the air. Using this app, you can listen to any [HD Radio Station](https://hdradio.com/stations/) in your area using the `nrsc5` tool!

## Installation

> ⚠️ WARNING! I have been having issues with building through GitHub Actions, so the GitHub Releases might not run. Or, they might run, but not have any sound when playing a radio station. I recommend compiling from source in the next section.

Installation should be as simple as going to the [GitHub Releases](https://github.com/njfdev/rtlsdr-radio/releases) and downloading the most recent application from the "Assets" dropdown for your specific OS (Windows support is expected soon).

## Compiling from Source

Compiling from source is very easy!

First, install Rust if it is not already installed:

```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

Then, make sure to install the prerequisites:

```bash
# MacOS with brew
brew install cmake autoconf automake libtool git librtlsdr libao fftw

# Debian/Ubuntu Based Linux OSes
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.0-dev librsvg2-dev patchelf git build-essential cmake autoconf automake libtool libao-dev libfftw3-dev librtlsdr-dev nodejs npm
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
