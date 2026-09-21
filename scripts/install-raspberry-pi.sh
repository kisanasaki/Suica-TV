#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/suica-tv"
data_root="${XDG_DATA_HOME:-$HOME/.local/share}"
data_dir="$data_root/suica-tv"
systemd_dir="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
bin_dir="$HOME/.local/bin"

command -v cargo >/dev/null || {
  echo "Rust is missing. Install rustup first: https://rustup.rs" >&2
  exit 1
}
command -v npm >/dev/null || {
  echo "npm is missing. Install Node.js 18 or newer first." >&2
  exit 1
}

cd "$repo_dir/apps/tv"
npm ci
npm test
npm run build

cd "$repo_dir"
cargo test --workspace
cargo build --release --workspace

install -d "$bin_dir" "$config_dir" "$data_dir" "$systemd_dir"
install -m 0755 "$repo_dir/target/release/suica-core" "$bin_dir/suica-core"
sed "s|WorkingDirectory=%h/suica-tv|WorkingDirectory=$repo_dir|" \
  "$repo_dir/deploy/suica-core.service" > "$systemd_dir/suica-core.service"
chmod 0644 "$systemd_dir/suica-core.service"

if [[ ! -f "$config_dir/suica-core.toml" ]]; then
  sed "s|/home/pi/suica-tv|$repo_dir|g; s|/home/pi/.local/share|$data_root|g" \
    "$repo_dir/config/suica-core.example.toml" > "$config_dir/suica-core.toml"
  chmod 0600 "$config_dir/suica-core.toml"
  echo "Created $config_dir/suica-core.toml"
  echo "Set pairing_code to a private six-digit value before enabling the service."
else
  echo "Kept existing $config_dir/suica-core.toml"
fi

systemctl --user daemon-reload
echo "Installation complete. Review the config, then run:"
echo "  systemctl --user enable --now suica-core"
