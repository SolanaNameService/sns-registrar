set -e

cargo test-sbf --features no-referrer-check
cargo test-sbf --features no-referrer-check no-special-discount-fee
cargo test-sbf test_state_create_split_v1_and_v2