default:
	cargo build
	cp target/debug/kestrel .
	./kestrel program.ke
	./a.out

opt_sanitize:
	cargo build
	cp target/debug/kestrel .
	./kestrel program.ke -o -fsanitize
	./a.out
	
opt_no_ou:
	cargo build
	cp target/debug/kestrel .
	./kestrel program.ke -o -fno-ou-checks
	./a.out
	
opt_all:
	cargo build
	cp target/debug/kestrel .
	./kestrel program.ke -o -fsanitize -fno-ou-checks
	./a.out

noopt_sanitize:
	cargo build
	cp target/debug/kestrel .
	./kestrel program.ke -fsanitize
	./a.out
	
noopt_no_ou:
	cargo build
	cp target/debug/kestrel .
	./kestrel program.ke -fno-ou-checks
	./a.out
	
noopt_all:
	cargo build
	cp target/debug/kestrel .
	./kestrel program.ke -fsanitize -fno-ou-checks
	./a.out