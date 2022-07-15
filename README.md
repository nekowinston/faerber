![Logo](assets/logo.png) 

Faerber_rs is a CLI tool written in Rust which modifies your RGB images to your different colourschemes. It comes with support for multiple colourschemes and has crossplatform support for Android, Linux , MacOS and Microsoft Windows.

# Downloads

There are multiple ways to download this package 

1. Github Release
2. Brew (*nix) (WIP)
3. AUR (WIP)
4. Windows Package Manager (Windows)(WIP)
5. Building from source

## Github Release 

[![Build](https://github.com/farbenfroh/faerber/actions/workflows/main.yml/badge.svg)](https://github.com/farbenfroh/faerber_rs/actions/workflows/ci.yml)


[Click Me](https://github.com/hirschmann/farbenfroh/faerber_rs)

## Build Instructions 

**Prerequisites**- Make sure rust is installed
If not then visit [Rust Installation Guide](https://forge.rust-lang.org/infra/other-installation-methods.html) for PC/Mac and if you are using Android via termux then install it via `pkg install rust`.

- Clone this repo and cd into the repo directory.
- Type `cargo build --release` to build faerber.
- cd into the native dir.
- Type `cargo install --path .` to install it.
- The resulting binary should be in `~/.cargo/bin` directory.

