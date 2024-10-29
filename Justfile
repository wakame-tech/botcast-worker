set dotenv-filename := ".env"

default:
    # docker compose up -d voicevox_engine
    VOICEVOX_ENDPOINT=http://localhost:50021 RUST_BACKTRACE=1 cargo run -p worker

check:
    cargo check

run script podcastId episodeId:
    cargo run -p cli -- run {{script}} --context '{"podcastId":"{{podcastId}}","episodeId":"{{episodeId}}"}'
