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
BIN_N=asus-notify
LEDCFG=asusd-ledmodes.toml
X11CFG=90-nvidia-screen-G05.conf
PMRULES=90-asusd-nvidia-pm.rules
VERSION:=$(shell grep -Pm1 'version = "(\d.\d.\d)"' asus-nb-ctrl/Cargo.toml | cut -d'"' -f2)

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
	install -D -m 0755 "target/release/$(BIN_N)" "$(DESTDIR)$(bindir)/$(BIN_N)"
	install -D -m 0644 "data/$(PMRULES)" "$(DESTDIR)/lib/udev/rules.d/$(PMRULES)"
	install -D -m 0644 "data/$(BIN_D).rules" "$(DESTDIR)/lib/udev/rules.d/99-$(BIN_D).rules"
	install -D -m 0644 "data/$(LEDCFG)" "$(DESTDIR)$(sysconfdir)/asusd/$(LEDCFG)"
	install -D -m 0644 "data/$(BIN_D).conf" "$(DESTDIR)$(sysconfdir)/dbus-1/system.d/$(BIN_D).conf"
	install -D -m 0644 "data/$(X11CFG)" "$(DESTDIR)$(sysconfdir)/X11/xorg.conf.d/$(X11CFG)"
	install -D -m 0644 "data/$(BIN_D).service" "$(DESTDIR)/lib/systemd/system/$(BIN_D).service"
	install -D -m 0644 "data/$(BIN_N).service" "$(DESTDIR)/lib/systemd/user/$(BIN_N).service"
	install -D -m 0644 "data/icons/asus_notif_yellow.png" "$(DESTDIR)/usr/share/icons/hicolor/512x512/apps/asus_notif_yellow.png"
	install -D -m 0644 "data/icons/asus_notif_green.png" "$(DESTDIR)/usr/share/icons/hicolor/512x512/apps/asus_notif_green.png"
	install -D -m 0644 "data/icons/asus_notif_red.png" "$(DESTDIR)/usr/share/icons/hicolor/512x512/apps/asus_notif_red.png"
	install -D -m 0644 "data/_asusctl" "$(DESTDIR)/usr/share/zsh/site-functions/_asusctl"

uninstall:
	rm -f "$(DESTDIR)$(bindir)/$(BIN_C)"
	rm -f "$(DESTDIR)$(bindir)/$(BIN_D)"
	rm -f "$(DESTDIR)$(bindir)/$(BIN_N)"
	rm -f "$(DESTDIR)/lib/udev/rules.d/$(PMRULES)"
	rm -f "$(DESTDIR)/lib/udev/rules.d/99-$(BIN_D).rules"
	rm -f "$(DESTDIR)$(sysconfdir)/dbus-1/system.d/$(BIN_D).conf"
	rm -f "$(DESTDIR)$(sysconfdir)/X11/xorg.conf.d/$(X11CFG)"
	rm -f "$(DESTDIR)/lib/systemd/system/$(BIN_D).service"
	rm -r "$(DESTDIR)/lib/systemd/user/$(BIN_N).service"
	rm -r "$(DESTDIR)/usr/share/icons/hicolor/512x512/apps/asus_notif_*"
	rm -f "$(DESTDIR)/usr/share/zsh/site-functions/_asusctl"

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

target/release/$(BIN_D): $(SRC)
ifeq ($(VENDORED),1)
	@echo "version = $(VERSION)"
	tar pxf vendor_asus-nb-ctrl_$(VERSION).tar.xz
endif
	cargo build $(ARGS)
