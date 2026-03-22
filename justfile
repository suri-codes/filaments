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
dev-db := justfile_directory() + "/target/dev.db"
dev-db-url := "sqlite:///" + dev-db

# build entities from migrations
[working-directory:"crates/db"]
entity:
    # create the dev db
    rm -f {{dev-db}}
    touch {{dev-db}}

    # run the migration
    cd migration && cargo run -- -u {{dev-db-url}}

    # generate entity files based off the migraiton
    sea-orm-cli generate entity \
        --database-url {{dev-db-url}} \
        --output-dir ./src/entity \
        --entity-format=dense # add flag if expanded format is needed for debugging
        
    # add migraton::types to every file in entity
    sed -i '4i use migration::types::*;' ./src/entity/*.rs

    # replace elementary types with specific ones
    sed -i 's/pub nano_id: String/pub nano_id: NanoId/g' ./src/entity/*.rs
    sed -i 's/pub priority: String/pub priority: Priority/g' ./src/entity/*.rs
    
    # replace parent_group_id with proper nano_id
    sed -i 's/pub parent_group_id: Option<String>/pub parent_group_id: Option<NanoId>/g' ./src/entity/*.rs
    
    # replace group_id with nano_id
    sed -i 's/pub group_id: String/pub group_id: NanoId/g' ./src/entity/*.rs
        








