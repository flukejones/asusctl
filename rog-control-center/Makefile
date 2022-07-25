VERSION := $(shell grep -Pm1 'version = "(\d.\d.\d)"' Cargo.toml | cut -d'"' -f2)

INSTALL = install
INSTALL_PROGRAM = ${INSTALL} -D -m 0755
INSTALL_DATA = ${INSTALL} -D -m 0644

prefix = /usr
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
datarootdir = $(prefix)/share
libdir = $(exec_prefix)/lib

BIN_ROG := rog-control-center

SRC := Cargo.toml Cargo.lock Makefile $(shell find -type f -wholename '**/src/*.rs')

DEBUG ?= 0
ifeq ($(DEBUG),0)
	ARGS += --release
	TARGET = release
endif

VENDORED ?= 0
ifeq ($(VENDORED),1)
	ARGS += --frozen
endif

all: build

clean:
	cargo clean

distclean:
	rm -rf .cargo vendor vendor.tar.xz

install:
	$(INSTALL_PROGRAM) "./target/release/$(BIN_ROG)" "$(DESTDIR)$(bindir)/$(BIN_ROG)"
	$(INSTALL_DATA) "./data/$(BIN_ROG).desktop" "$(DESTDIR)$(datarootdir)/applications/$(BIN_ROG).desktop"
	$(INSTALL_DATA) "./data/$(BIN_ROG).png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/$(BIN_ROG).png"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/$(BIN_ROG)"
	rm -r "$(DESTDIR)$(datarootdir)/applications/$(BIN_ROG).desktop"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/$(BIN_ROG).png"

update:
	cargo update

vendor:
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	mv .cargo/config ./cargo-config
	rm -rf .cargo
	tar pcfJ vendor_asusctl_$(VERSION).tar.xz vendor
	rm -rf vendor

build:
ifeq ($(VENDORED),1)
	@echo "version = $(VERSION)"
	tar pxf vendor_asusctl_$(VERSION).tar.xz
endif
	cargo build $(ARGS)

.PHONY: all clean distclean install uninstall update build
