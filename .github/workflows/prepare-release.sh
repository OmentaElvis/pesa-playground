#!/bin/bash
set -eo pipefail # Exit immediately if a pipeline fails

NEW_VERSION=$1 # semantic-release will pass nextRelease.version as the first argument

echo "Updating root Cargo.toml to version ${NEW_VERSION}..."
sed -i "s/\(version = \"\)[0-9.]*\(\"\)/\1${NEW_VERSION}\2/" Cargo.toml
echo "Root Cargo.toml updated."

echo "Update tauri config"
tmp="$(mktemp)"
jq ".version = \"$(NEW_VERSION)\"" crates/pesa-tauri/tauri.conf.json > "$tmp" \
  && mv "$tmp" crates/pesa-tauri/tauri.conf.json

echo "Prepare step for semantic-release completed."
