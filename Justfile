set dotenv-filename := ".env"

default:
    # docker compose up -d voicevox_engine
    RUST_BACKTRACE=1 cargo run -p worker

check:
    cargo check
