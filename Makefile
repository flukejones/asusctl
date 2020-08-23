prefix ?= /usr
sysconfdir ?= /etc
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
libdir = $(exec_prefix)/lib
includedir = $(prefix)/include
datarootdir = $(prefix)/share
datadir = $(datarootdir)

SRC = Cargo.toml Cargo.lock Makefile $(shell find -type f -wholename '**/src/*.rs')

.PHONY: all clean distclean install uninstall update

BIN_C=asusctl
BIN_D=asusd
LEDCONFIG=asusd-ledmodes.toml
VERSION:=$(shell grep -P 'version = "(\d.\d.\d)"' asus-nb-ctrl/Cargo.toml | cut -d'"' -f2)

DEBUG ?= 0
ifeq ($(DEBUG),0)
	ARGS += "--release"
	TARGET = release
endif

VENDORED ?= 0
ifeq ($(VENDORED),1)
	ARGS += "--frozen"
endif

all: target/release/$(BIN_D)

clean:
	cargo clean

distclean:
	rm -rf .cargo vendor vendor.tar.xz

install: all
	install -D -m 0755 "target/release/$(BIN_C)" "$(DESTDIR)$(bindir)/$(BIN_C)"
	install -D -m 0755 "target/release/$(BIN_D)" "$(DESTDIR)$(bindir)/$(BIN_D)"
	install -D -m 0644 "data/$(BIN_D).rules" "$(DESTDIR)/lib/udev/rules.d/99-$(BIN_D).rules"
	install -D -m 0644 "data/$(LEDCONFIG)" "$(DESTDIR)$(sysconfdir)/asusd/$(LEDCONFIG)"
	install -D -m 0644 "data/$(BIN_D).conf" "$(DESTDIR)$(sysconfdir)/dbus-1/system.d/$(BIN_D).conf"
	install -D -m 0644 "data/$(BIN_D).service" "$(DESTDIR)/lib/systemd/system/$(BIN_D).service"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/$(BIN_C)"
	rm -f "$(DESTDIR)$(bindir)/$(BIN_D)"
	rm -f "$(DESTDIR)/lib/udev/rules.d/99-$(BIN_D).rules"
	rm -f "$(DESTDIR)$(sysconfdir)/dbus-1/system.d/$(BIN_D).conf"
	rm -f "$(DESTDIR)/lib/systemd/system/$(BIN_D).service"

update:
	cargo update

vendor:
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	mv .cargo/config ./cargo-config
	rm -rf .cargo
	tar pcfJ vendor-$(VERSION).tar.xz vendor
	rm -rf vendor

target/release/$(BIN_D): $(SRC)
ifeq ($(VENDORED),1)
	@echo "version = $(VERSION)"
	tar pxf vendor-$(VERSION).tar.xz
endif
	cargo build $(ARGS)
