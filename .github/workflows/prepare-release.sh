#!/bin/bash
set -eo pipefail # Exit immediately if a pipeline fails

NEW_VERSION=$1 # semantic-release will pass nextRelease.version as the first argument

echo "Updating root Cargo.toml to version ${NEW_VERSION}..."
sed -i "s/\(version = \"\)[0-9.]*\(\"\)/\1${NEW_VERSION}\2/" Cargo.toml
echo "Root Cargo.toml updated."

echo "Prepare step for semantic-release completed."