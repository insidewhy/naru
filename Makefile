.PHONY: default

default:
	cargo build && echo && echo -e "$$(cat test-input.txt)" | (RUST_BACKTRACE=1 lovely_matches=$$(./target/debug/naru) && echo "matched ($$lovely_matches)")
