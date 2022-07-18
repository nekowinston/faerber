<div style="text-align: center;">
  <img src="assets/logo.png" style="width: 8rem;"/>
</div>

<div style="text-align: center;">
  <a href="https://github.com/farbenfroh/faerber/actions/workflows/ci.yaml">
    <img alt="GitHub Workflow Status (main)" src="https://img.shields.io/github/workflow/status/farbenfroh/faerber/ci.yaml/main?color=a6e3a1&style=flat-square">
  </a>
  <a href="LICENSE">
    <img alt="License: MIT" src="https://img.shields.io/github/license/farbenfroh/faerber?color=a6e3a1&style=flat-square">
  </a>
</div>

faerber is a CLI tool written in Rust, which matches your RGB images to different colourschemes.
It comes with support for multiple colourschemes and has crossplatform support for Android, Linux, macOS, and Microsoft Windows.

## Installation

There are multiple ways to download this package

1. GitHub Release
2. Brew (\*nix) (WIP)
3. AUR (WIP)
4. Windows Package Manager (Windows)(WIP)
5. [Building from source](#Build-instructions)

## Build instructions

### **Prerequisites**

Make sure Rust is installed. If it isn't, you can install it via [rustup.rs](https://rustup.rs).
If you are using Android via termux then install it via `pkg install rust`.

- Clone this repo and cd into the repo directory.
- Type `cargo build --release` to build faerber.
- cd into the native dir.
- Type `cargo install --path .` to install it.
- The resulting binary should be in `~/.cargo/bin` directory.
