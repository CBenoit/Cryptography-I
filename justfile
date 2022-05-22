#!/usr/bin/env -S just --justfile

week1:
	cargo run --bin week1 -- resources/week1_ciphertexts.txt
