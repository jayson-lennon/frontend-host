build:
    cargo +nightly build

release:
    cargo +nightly build --release && strip target/release/frontend-host
    cargo +nightly build --release --target=x86_64-pc-windows-gnu