set dotenv-filename := ".env"

default:
    # docker compose up -d voicevox_engine
    VOICEVOX_ENDPOINT=http://localhost:50021 cargo run -p worker

check:
    cargo check

