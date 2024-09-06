set dotenv-filename := ".env"

default:
    docker compose -f compose.dev.yaml up -d
    VOICEVOX_ENDPOINT=http://localhost:50021 cargo run -p worker

check:
    cargo check

