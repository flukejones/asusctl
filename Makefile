VERSION := $(shell /usr/bin/grep -Pm1 'version = "(\d.\d.\d)"' Cargo.toml | cut -d'"' -f2)

INSTALL = install
INSTALL_PROGRAM = ${INSTALL} -D -m 0755
INSTALL_DATA = ${INSTALL} -D -m 0644

prefix = /usr
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
datarootdir = $(prefix)/share
libdir = $(exec_prefix)/lib
zshcpl = $(datarootdir)/zsh/site-functions

BIN_ROG := rog-control-center
BIN_C := asusctl
BIN_D := asusd
BIN_U := asusd-user
LEDCFG := aura_support.ron

SRC := Cargo.toml Cargo.lock Makefile $(shell find -type f -wholename '**/src/*.rs')

STRIP_BINARIES ?= 0

DEBUG ?= 0
ifeq ($(DEBUG),0)
	ARGS += --release
	TARGET = release
else
	ARGS += --profile dev
	TARGET = debug
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

install-program:
	$(INSTALL_PROGRAM) "./target/$(TARGET)/$(BIN_ROG)" "$(DESTDIR)$(bindir)/$(BIN_ROG)"

	$(INSTALL_PROGRAM) "./target/$(TARGET)/$(BIN_C)" "$(DESTDIR)$(bindir)/$(BIN_C)"
	$(INSTALL_PROGRAM) "./target/$(TARGET)/$(BIN_D)" "$(DESTDIR)$(bindir)/$(BIN_D)"
	$(INSTALL_PROGRAM) "./target/$(TARGET)/$(BIN_U)" "$(DESTDIR)$(bindir)/$(BIN_U)"

install-data:
	$(INSTALL_DATA) "./rog-control-center/data/$(BIN_ROG).desktop" "$(DESTDIR)$(datarootdir)/applications/$(BIN_ROG).desktop"
	$(INSTALL_DATA) "./rog-control-center/data/$(BIN_ROG).png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/$(BIN_ROG).png"
	cd rog-aura/data/layouts && find . -type f -name "*.ron" -exec $(INSTALL_DATA) "{}" "$(DESTDIR)$(datarootdir)/rog-gui/layouts/{}" \;

	$(INSTALL_DATA) "./data/$(BIN_D).rules" "$(DESTDIR)$(libdir)/udev/rules.d/99-$(BIN_D).rules"
	$(INSTALL_DATA) "./rog-aura/data/$(LEDCFG)" "$(DESTDIR)$(datarootdir)/asusd/$(LEDCFG)"
	$(INSTALL_DATA) "./data/$(BIN_D).conf" "$(DESTDIR)$(datarootdir)/dbus-1/system.d/$(BIN_D).conf"

	$(INSTALL_DATA) "./data/$(BIN_D).service" "$(DESTDIR)$(libdir)/systemd/system/$(BIN_D).service"
	$(INSTALL_DATA) "./data/$(BIN_U).service" "$(DESTDIR)$(libdir)/systemd/user/$(BIN_U).service"

	$(INSTALL_DATA) "./data/icons/asus_notif_yellow.png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_yellow.png"
	$(INSTALL_DATA) "./data/icons/asus_notif_green.png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_green.png"
	$(INSTALL_DATA) "./data/icons/asus_notif_blue.png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_blue.png"
	$(INSTALL_DATA) "./data/icons/asus_notif_red.png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_red.png"
	$(INSTALL_DATA) "./data/icons/asus_notif_orange.png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_orange.png"
	$(INSTALL_DATA) "./data/icons/asus_notif_white.png" "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_white.png"

	$(INSTALL_DATA) "./data/icons/scalable/gpu-compute.svg" "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/gpu-compute.svg"
	$(INSTALL_DATA) "./data/icons/scalable/gpu-hybrid.svg" "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/gpu-hybrid.svg"
	$(INSTALL_DATA) "./data/icons/scalable/gpu-integrated.svg" "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/gpu-integrated.svg"
	$(INSTALL_DATA) "./data/icons/scalable/gpu-nvidia.svg" "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/gpu-nvidia.svg"
	$(INSTALL_DATA) "./data/icons/scalable/gpu-vfio.svg" "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/gpu-vfio.svg"
	$(INSTALL_DATA) "./data/icons/scalable/notification-reboot.svg" "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/notification-reboot.svg"

	cd rog-anime/data && find "./anime" -type f -exec $(INSTALL_DATA) "{}" "$(DESTDIR)$(datarootdir)/asusd/{}" \;

install: install-program install-data

uninstall:
	rm -f "$(DESTDIR)$(bindir)/$(BIN_ROG)"
	rm -r "$(DESTDIR)$(datarootdir)/applications/$(BIN_ROG).desktop"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/$(BIN_ROG).png"

	rm -f "$(DESTDIR)$(bindir)/$(BIN_C)"
	rm -f "$(DESTDIR)$(bindir)/$(BIN_D)"
	rm -f "$(DESTDIR)$(libdir)/udev/rules.d/99-$(BIN_D).rules"
	rm -f "$(DESTDIR)/etc/asusd/$(LEDCFG)"
	rm -f "$(DESTDIR)$(datarootdir)/dbus-1/system.d/$(BIN_D).conf"
	rm -f "$(DESTDIR)$(libdir)/systemd/system/$(BIN_D).service"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_yellow.png"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_green.png"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/512x512/apps/asus_notif_red.png"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/gpu-compute.svg"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/gpu-hybrid.svg"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/gpu-integrated.svg"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/gpu-nvidia.svg"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/gpu-vfio.svg"
	rm -r "$(DESTDIR)$(datarootdir)/icons/hicolor/scalable/status/notification-reboot.svg"
	rm -rf "$(DESTDIR)$(datarootdir)/asusd"
	rm -rf "$(DESTDIR)$(datarootdir)/rog-gui"

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

bindings:
	typeshare ./rog-anime/src/ --lang=typescript --output-file=bindings/ts/anime.ts
	typeshare ./rog-aura/src/ --lang=typescript --output-file=bindings/ts/aura.ts
	typeshare ./rog-profiles/src/ --lang=typescript --output-file=bindings/ts/profiles.ts
	typeshare ./rog-platform/src/ --lang=typescript --output-file=bindings/ts/platform.ts

introspect:
	gdbus introspect --system -d org.asuslinux.Daemon -o /org/asuslinux/Aura -x > bindings/dbus-xml/org-asuslinux-aura-4.xml
	gdbus introspect --system -d org.asuslinux.Daemon -o /org/asuslinux/Anime -x > bindings/dbus-xml/org-asuslinux-anime-4.xml
	gdbus introspect --system -d org.asuslinux.Daemon -o /org/asuslinux/Platform -x > bindings/dbus-xml/org-asuslinux-platform-4.xml
	gdbus introspect --system -d org.asuslinux.Daemon -o /org/asuslinux/Power -x > bindings/dbus-xml/org-asuslinux-power-4.xml
	gdbus introspect --system -d org.asuslinux.Daemon -o /org/asuslinux/Profile -x > bindings/dbus-xml/org-asuslinux-profile-4.xml
	gdbus introspect --system -d org.asuslinux.Daemon -o /org/asuslinux/Supported -x > bindings/dbus-xml/org-asuslinux-supported-4.xml
	xmlstarlet ed -L -O -d '//interface[@name="org.freedesktop.DBus.Introspectable"]' bindings/dbus-xml/org-asuslinux-*
	xmlstarlet ed -L -O -d '//interface[@name="org.freedesktop.DBus.Properties"]' bindings/dbus-xml/org-asuslinux-*
	xmlstarlet ed -L -O -d '//interface[@name="org.freedesktop.DBus.Peer"]' bindings/dbus-xml/org-asuslinux-*

build:
ifeq ($(VENDORED),1)
	cargo vendor-filterer --platform x86_64-unknown-linux-gnu vendor
	#cargo vendor
	@echo "version = $(VERSION)"
	tar pxf vendor_asusctl_$(VERSION).tar.xz
endif
	cargo build $(ARGS)
ifneq ($(STRIP_BINARIES),0)
	strip -s ./target/$(TARGET)/$(BIN_C)
	strip -s ./target/$(TARGET)/$(BIN_D)
	strip -s ./target/$(TARGET)/$(BIN_U)
	strip -s ./target/$(TARGET)/$(BIN_ROG)
endif


.PHONY: all clean distclean install uninstall update build bindings
