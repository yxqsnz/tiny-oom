#!/usr/bin/env bash
(sudo -v)
echo '[info] installing: '
printf '\t(Rust) compile project ... '
cargo build --release -q && echo 'ok' || (echo "error" && exit 1)
printf '\t(Misc) install project ... '
sudo install -Dm755 target/release/tiny-oom -t "/usr/bin/" && echo 'ok'
printf '\t(Misc) install service ... '
sudo install -Dm644 tiny-oom.service -t "/etc/systemd/system/" && echo 'ok'
