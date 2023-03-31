#!/usr/bin/env nix-shell
#!nix-shell -i bash -p curl gojq
#shellcheck shell=bash

UPSTREAM="https://raw.githubusercontent.com/wez/wezterm/main/docs/colorschemes/data.json"
dir="$(dirname "$0")"
curl -fsSL "$UPSTREAM" | gojq -cf "${dir}/convert.jq" >"${dir}/../colorschemes/vendor/wezterm.json"
