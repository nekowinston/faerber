<h3 align="center">
<!-- ALL-CONTRIBUTORS-BADGE:START - Do not remove or modify this section -->
[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors-)
<!-- ALL-CONTRIBUTORS-BADGE:END -->
  <img src="assets/logo.png" style="width: 8rem;"/><br/>
  faerber
</h3>
<p align="center">
  <a href="https://github.com/farbenfroh/faerber/actions/workflows/ci.yaml">
    <img alt="GitHub Workflow Status (main)" src="https://img.shields.io/github/workflow/status/farbenfroh/faerber/ci.yaml/main?color=a6e3a1&style=flat-square">
  </a>
  <a href="LICENSE">
    <img alt="License: MIT" src="https://img.shields.io/github/license/farbenfroh/faerber?color=a6e3a1&style=flat-square">
  </a>
</p>

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

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tr>
    <td align="center"><a href="https://www.youtube.com/watch?v=dQw4w9WgXcQ"><img src="https://avatars.githubusercontent.com/u/60423203?v=4?s=100" width="100px;" alt=""/><br /><sub><b>Mirage</b></sub></a><br /><a href="https://github.com/farbenfroh/faerber/commits?author=skinatro" title="Documentation">📖</a></td>
  </tr>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!