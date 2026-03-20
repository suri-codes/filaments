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


# Only used to build / generate entities
dev-db := "sqlite:///" + justfile_directory() + "/target/dev.db"

# build entities from migrations
[working-directory:"crates/db"]
entity:
    touch ../../target/dev.db
    cd migration && cargo run -- -u {{dev-db}}
    sea-orm-cli generate entity \
        --database-url {{dev-db}} \
        --output-dir ./src/entity \
        # --expanded-format # add flag if expanded format is needed for debugging
        








