VERSION := $(shell grep -Pm1 'version = "(\d.\d.\d)"' asus-nb-ctrl/Cargo.toml | cut -d'"' -f2)

INSTALL = install
INSTALL_PROGRAM = ${INSTALL} -D -m 0755
INSTALL_DATA = ${INSTALL} -D -m 0644

prefix = /usr
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
datarootdir = $(prefix)/share
libdir = $(exec_prefix)/lib
zshcpl = $(datarootdir)/zsh/site-functions

BIN_C := asusctl
BIN_D := asusd
BIN_N := asus-notify
LEDCFG := asusd-ledmodes.toml
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
	$(INSTALL_PROGRAM) "./target/release/$(BIN_C)" "$(DESTDIR)$(bindir)/$(BIN_C)"
	$(INSTALL_PROGRAM) "./target/release/$(BIN_D)" "$(DESTDIR)$(bindir)/$(BIN_D)"
	$(INSTALL_PROGRAM) "./target/release/$(BIN_N)" "$(DESTDIR)$(bindir)/$(BIN_N)"
	$(INSTALL_DATA) "./data/$(PMRULES)" "$(DESTDIR)$(libdir)/udev/rules.d/$(PMRULES)"
	$(INSTALL_DATA) "./data/$(BIN_D).rules" "$(DESTDIR)$(libdir)/udev/rules.d/99-$(BIN_D).rules"
	$(INSTALL_DATA) "./data/$(LEDCFG)" "$(DESTDIR)/etc/asusd/$(LEDCFG)"
	$(INSTALL_DATA) "./data/$(BIN_D).conf" "$(DESTDIR)$(datarootdir)/dbus-1/system.d/$(BIN_D).conf"
	$(INSTALL_DATA) "./data/$(X11CFG)" "$(DESTDIR)$(datarootdir)/X11/xorg.conf.d/$(X11CFG)"
	$(INSTALL_DATA) "./data/$(BIN_D).service" "$(DESTDIR)$(libdir)/systemd/system/$(BIN_D).service"
	$(INSTALL_DATA) "./data/$(BIN_N).service" "$(DESTDIR)$(libdir)/systemd/user/$(BIN_N).service"
	$(INSTALL_DATA) "./data/icons/asus_notif_yellow.png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_yellow.png"
	$(INSTALL_DATA) "./data/icons/asus_notif_green.png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_green.png"
	$(INSTALL_DATA) "./data/icons/asus_notif_red.png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_red.png"
	$(INSTALL_DATA) "./data/_asusctl" "$(DESTDIR)$(zshcpl)/_asusctl"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/$(BIN_C)"
	rm -f "$(DESTDIR)$(bindir)/$(BIN_D)"
	rm -f "$(DESTDIR)$(bindir)/$(BIN_N)"
	rm -f "$(DESTDIR)$(libdir)/udev/rules.d/$(PMRULES)"
	rm -f "$(DESTDIR)$(libdir)/udev/rules.d/99-$(BIN_D).rules"
	rm -f "$(DESTDIR)/etc/asusd/$(LEDCFG)"
	rm -f "$(DESTDIR)$(datarootdir)/dbus-1/system.d/$(BIN_D).conf"
	rm -f "$(DESTDIR)$(datarootdir)/X11/xorg.conf.d/$(X11CFG)"
	rm -f "$(DESTDIR)$(libdir)/systemd/system/$(BIN_D).service"
	rm -r "$(DESTDIR)$(libdir)/systemd/user/$(BIN_N).service"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_yellow.png"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_green.png"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_red.png"
	rm -f "$(DESTDIR)$(zshcpl)/_asusctl"

update:
	cargo update

vendor:
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	mv .cargo/config ./cargo-config
	rm -rf .cargo
	tar pcfJ vendor_asus-nb-ctrl_$(VERSION).tar.xz vendor
	rm -rf vendor

build:
ifeq ($(VENDORED),1)
	@echo "version = $(VERSION)"
	tar pxf vendor_asus-nb-ctrl_$(VERSION).tar.xz
endif
	cargo build $(ARGS)

.PHONY: all clean distclean install uninstall update build
