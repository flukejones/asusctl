VERSION := $(shell grep -Pm1 'version = "(\d.\d.\d)"' daemon/Cargo.toml | cut -d'"' -f2)

INSTALL = install
INSTALL_PROGRAM = ${INSTALL} -D -m 0755
INSTALL_DATA = ${INSTALL} -D -m 0644

prefix = /usr
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
datarootdir = $(prefix)/share
libdir = $(exec_prefix)/lib

BIN_SD := supergfxd
BIN_SC := supergfxctl
X11CFG := 90-nvidia-screen-G05.conf
PMRULES := 90-asusd-nvidia-pm.rules

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
	$(INSTALL_PROGRAM) "./target/release/$(BIN_SD)" "$(DESTDIR)$(bindir)/$(BIN_SD)"
	$(INSTALL_PROGRAM) "./target/release/$(BIN_SC)" "$(DESTDIR)$(bindir)/$(BIN_SC)"
	$(INSTALL_DATA) "./data/$(BIN_SD).service" "$(DESTDIR)$(libdir)/systemd/system/$(BIN_SD).service"
	$(INSTALL_DATA) "./data/org.supergfxctl.Daemon.conf" "$(DESTDIR)$(datarootdir)/dbus-1/system.d/org.supergfxctl.Daemon.conf"
	$(INSTALL_DATA) "./data/$(X11CFG)" "$(DESTDIR)$(datarootdir)/X11/xorg.conf.d/$(X11CFG)"
	$(INSTALL_DATA) "./data/$(PMRULES)" "$(DESTDIR)$(libdir)/udev/rules.d/$(PMRULES)"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/$(BIN_SC)"
	rm -f "$(DESTDIR)$(bindir)/$(BIN_SD)"
	rm -f "$(DESTDIR)$(libdir)/systemd/system/$(BIN_SD).service"
	rm -f "$(DESTDIR)$(datarootdir)/dbus-1/system.d/org.supergfxctl.Daemon.conf"
	rm -f "$(DESTDIR)$(datarootdir)/X11/xorg.conf.d/$(X11CFG)"
	rm -f "$(DESTDIR)$(libdir)/udev/rules.d/$(PMRULES)"

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
