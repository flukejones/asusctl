%if %{defined fedora}
%global debug_package %{nil}
%endif

# Use hardening ldflags.
%global rustflags -Clink-arg=-Wl,-z,relro,-z,now
Name:           asus-nb-ctrl
Version:        1.0.0
Release:        0
Summary:        Text editor for terminal
License:        MPLv2
Group:          Productivity/Text/Editors
URL:            https://gitlab.com/asus-linux/asus-nb-ctrl
Source:         %{name}-%{version}.tar.gz
# cargo vendor &&
# tar cfJ vendor.tar.xz vendor
Source1:        vendor.tar.xz
BuildRequires:  clang-devel
BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  rust-std-static
BuildRequires:  pkgconfig(dbus-1)
BuildRequires:  pkgconfig(libudev)

%description
ASUS Laptop control

%prep
%setup -q -n %name-next
%setup -q -n %name-next -D -T -a 1

mkdir .cargo
cat >.cargo/config <<EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

%build
export RUSTFLAGS="%{rustflags}"
RUST_BACKTRACE=1 cargo build --release

%install
export RUSTFLAGS="%{rustflags}"

mkdir -p "%{buildroot}%{_bindir}"
install -D -m 0755 target/release/asusd %{buildroot}%{_bindir}/asusd
install -D -m 0755 target/release/asusctl %{buildroot}%{_bindir}/asusctl
install -D -m 0644 data/asusd.rules %{buildroot}%{_udevrulesdir}/90-asusd.rules
install -D -m 0644 data/asusd.conf  %{buildroot}%{_sysconfdir}/dbus-1/system.d/asusd.conf
install -D -m 0644 data/asusd.service %{buildroot}%{_unitdir}/asusd.service
install -D -m 0644 data/asusd-ledmodes.toml  %{buildroot}%{_sysconfdir}/asusd/asusd-ledmodes.toml

mkdir -p "%{buildroot}%{_datadir}/licenses/%{name}"
cp LICENSE "%{buildroot}%{_datadir}/licenses/%{name}/"

mkdir -p "%{buildroot}/bin"

%files
%license LICENSE
%{_bindir}/asusd
%{_bindir}/asusctl
%{_unitdir}/asusd.service
%{_udevrulesdir}/90-asusd.rules
%{_sysconfdir}/dbus-1/system.d/asusd.conf
%{_sysconfdir}/asusd/asusd-ledmodes.toml

%changelog
