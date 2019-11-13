.PHONY: default

default:
	cargo build && echo && echo -e "$$(cat test-input.txt)" | (RUST_BACKTRACE=1 fucks=$$(./target/debug/toss) && echo "matched ($$fucks)")
