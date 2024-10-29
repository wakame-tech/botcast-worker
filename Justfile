set dotenv-filename := ".env"

default:
    # docker compose up -d voicevox_engine
    VOICEVOX_ENDPOINT=http://localhost:50021 RUST_BACKTRACE=1 cargo run -p worker

check:
    cargo check

run script:
    cargo run -p cli -- -p ./crates/cli/project run {{script}} 
