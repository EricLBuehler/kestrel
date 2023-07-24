default:
	cargo build
	cp target/debug/kestrel .
	./kestrel program.ke
	./a.out