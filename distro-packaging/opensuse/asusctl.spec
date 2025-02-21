#
# spec file for package asusctl
#
# Copyright (c) 2020-2021 Luke Jones <luke@ljones.dev>
#
# All modifications and additions to the file contributed by third parties
# remain the property of their copyright owners, unless otherwise agreed
# upon. The license for this file, and modifications and additions to the
# file, is the same license as for the pristine package itself (unless the
# license for the pristine package is not an Open Source License, in which
# case the license is the MIT License). An "Open Source License" is a
# license that conforms to the Open Source Definition (Version 1.9)
# published by the Open Source Initiative.

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#


%if %{defined fedora}
%global debug_package %{nil}
%endif

%define version 6.1.7
%define specrelease %{?dist}
%define pkg_release 8%{specrelease}

# Use hardening ldflags.
%global rustflags -Clink-arg=-Wl,-z,relro,-z,now
Name:    asusctl
Version: %{version}
Release: %{pkg_release}
Summary:        Control fan speeds, LEDs, graphics modes, and charge levels for ASUS notebooks
License:        MPL-2.0

URL:     https://gitlab.com/asus-linux/asusctl
Source0: %{name}-%{version}.tar.gz

%if %{defined fedora}
BuildRequires:  rust-packaging
BuildRequires:  systemd-rpm-macros
%else
BuildRequires:  cargo-packaging
%endif
BuildRequires:  git
BuildRequires:  clang-devel
BuildRequires:  cargo
BuildRequires:  cmake
BuildRequires:  rust
BuildRequires:  rust-std-static
BuildRequires:  pkgconfig(gbm)
BuildRequires:  pkgconfig(libinput)
BuildRequires:  pkgconfig(libseat)
BuildRequires:  pkgconfig(libudev)
BuildRequires:  pkgconfig(xkbcommon)
BuildRequires:  pkgconfig(libzstd)
BuildRequires:  desktop-file-utils

Requires:       libappindicator-gtk3

%description
asusctl is a utility for Linux to control many aspects of various ASUS laptops
but can also be used with non-Asus laptops with reduced features.

It provides an interface for rootless control of some system functions such as
fan speeds, keyboard LEDs, battery charge level, and graphics modes.
asusctl enables third-party apps to use the above with dbus methods.

%package rog-gui
Summary:        An experimental GUI for %{name}

%description rog-gui
A one-stop-shop GUI tool for asusd/asusctl. It aims to provide most controls,
a notification service, and ability to run in the background.

%prep
%autosetup
%if %{defined fedora}
%cargo_prep
%endif
sed -i 's|offline = true|offline = false|' .cargo/config.toml
sed -i 's|source.crates-io|source.ignore_this|' .cargo/config.toml

%build
export RUSTFLAGS="%{rustflags}"
%cargo_build

%install
export RUSTFLAGS="%{rustflags}"
mkdir -p "%{buildroot}/%{_bindir}" "%{buildroot}%{_docdir}"
%make_install

install -D -m 0644 README.md %{buildroot}/%{_docdir}/%{name}/README.md
install -D -m 0644 rog-anime/README.md %{buildroot}/%{_docdir}/%{name}/README-anime.md
install -D -m 0644 rog-anime/data/diagonal-template.png %{buildroot}/%{_docdir}/%{name}/diagonal-template.png

desktop-file-validate %{buildroot}/%{_datadir}/applications/rog-control-center.desktop

%pre
%service_add_pre asusd.service

%post
%service_add_post asusd.service

%preun
%service_del_preun asusd.service

%postun
%service_del_postun asusd.service

%files
%license LICENSE
%{_bindir}/asusd
%{_bindir}/asusd-user
%{_bindir}/asusctl
%{_unitdir}/asusd.service
%{_userunitdir}/asusd-user.service
%{_udevrulesdir}/99-asusd.rules
#%dir %{_sysconfdir}/asusd/
%{_datadir}/asusd/aura_support.ron
%{_datadir}/dbus-1/system.d/asusd.conf
%{_datadir}/icons/hicolor/512x512/apps/asus_notif_yellow.png
%{_datadir}/icons/hicolor/512x512/apps/asus_notif_green.png
%{_datadir}/icons/hicolor/512x512/apps/asus_notif_red.png
%{_datadir}/icons/hicolor/512x512/apps/asus_notif_blue.png
%{_datadir}/icons/hicolor/512x512/apps/asus_notif_orange.png
%{_datadir}/icons/hicolor/512x512/apps/asus_notif_white.png
%{_datadir}/icons/hicolor/scalable/status/gpu-compute.svg
%{_datadir}/icons/hicolor/scalable/status/gpu-hybrid.svg
%{_datadir}/icons/hicolor/scalable/status/gpu-integrated.svg
%{_datadir}/icons/hicolor/scalable/status/gpu-nvidia.svg
%{_datadir}/icons/hicolor/scalable/status/gpu-vfio.svg
%{_datadir}/icons/hicolor/scalable/status/notification-reboot.svg
%{_docdir}/%{name}/
%{_datadir}/asusd/

%files rog-gui
%{_bindir}/rog-control-center
%{_datadir}/applications/rog-control-center.desktop
%{_datadir}/icons/hicolor/512x512/apps/rog-control-center.png
%{_datadir}/rog-gui

%changelog
