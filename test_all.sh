#!/bin/sh
set -e

cargo test
cargo test -p chip8

