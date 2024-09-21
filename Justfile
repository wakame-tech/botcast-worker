set dotenv-filename := ".env"

default:
    docker compose up -d
    VOICEVOX_ENDPOINT=http://localhost:50021 cargo run -p worker

check:
    cargo check

