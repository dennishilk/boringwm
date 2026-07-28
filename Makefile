PREFIX ?= /usr/local
DESTDIR ?=
.PHONY: build install uninstall
build:
	cargo build --release
install: build
	install -Dm755 target/release/boringwm "$(DESTDIR)$(PREFIX)/bin/boringwm"
	install -Dm644 contrib/boringwm.1 "$(DESTDIR)$(PREFIX)/share/man/man1/boringwm.1"
	install -Dm755 contrib/boringwm-session "$(DESTDIR)$(PREFIX)/bin/boringwm-session"
	install -Dm644 contrib/boringwm.desktop "$(DESTDIR)$(PREFIX)/share/xsessions/boringwm.desktop"
	install -Dm644 config/boringwm.example.toml "$(DESTDIR)$(PREFIX)/share/doc/boringwm/boringwm.example.toml"
uninstall:
	rm -f "$(DESTDIR)$(PREFIX)/bin/boringwm" "$(DESTDIR)$(PREFIX)/bin/boringwm-session" "$(DESTDIR)$(PREFIX)/share/man/man1/boringwm.1" "$(DESTDIR)$(PREFIX)/share/xsessions/boringwm.desktop" "$(DESTDIR)$(PREFIX)/share/doc/boringwm/boringwm.example.toml"
