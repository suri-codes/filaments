# https://just.systems

default:
    @just --list

# release mode by default
mode := "debug"

_cargo_flags := if mode == "release" { "--release" } else { "" }

# Builds everything in the workspace
build:
    cargo build {{_cargo_flags}}

# Runs Filaments
run:
    cargo run {{_cargo_flags}}

# Run all tests
test:
    cargo test {{_cargo_flags}}
