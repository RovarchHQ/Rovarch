set -u
script=${1:-$(dirname "$0")/../install.sh}
srv=$(mktemp -d); home=$(mktemp -d); fakebin=$(mktemp -d)
case $home in "$HOME"|"$HOME"/*) echo "refusing to run: the temporary home is inside the real home"; exit 1;; esac
pass=0; failed=0
check() { if [ "$2" = ok ]; then pass=$((pass+1)); echo "  PASS  $1"; else failed=$((failed+1)); echo "  FAIL  $1  ($2)"; fi; }
tag=v9.9.9; name=Rovarch-$tag-x86_64.AppImage
mkdir -p "$srv/repos/Q/r" "$srv/Q/r/releases/download/$tag"
printf '[{"tag_name": "%s", "prerelease": true}, {"tag_name": "v1.0.0"}]' "$tag" > "$srv/repos/Q/r/releases"
printf 'fake appimage bytes' > "$srv/Q/r/releases/download/$tag/$name"
sum=$(sha256sum "$srv/Q/r/releases/download/$tag/$name" | cut -d' ' -f1)
printf '%s  %s\n' "$sum" "$name" > "$srv/Q/r/releases/download/$tag/SHA256SUMS"
port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')
(cd "$srv" && python3 -u -m http.server "$port" --bind 127.0.0.1 >"$srv.log" 2>&1 & echo $! > "$srv.pid")
for _ in 1 2 3 4 5 6 7 8 9 10; do curl -fs -o /dev/null "http://127.0.0.1:$port/" && break; sleep 0.5; done
run() { env -u XDG_DATA_HOME -u XDG_CONFIG_HOME -u XDG_CACHE_HOME -u XDG_BIN_HOME HOME="$home" ROVARCH_REPOSITORY=Q/r ROVARCH_API_URL=http://127.0.0.1:$port ROVARCH_DOWNLOAD_URL=http://127.0.0.1:$port "$@" 2>&1; }
installed="$home/.local/bin/Rovarch.AppImage"

out=$(run bash "$script"); [ -x "$installed" ] && [ "$(cat $installed)" = "fake appimage bytes" ] && echo "$out" | grep -q "v9.9.9" && check "installs the first (newest) release listed, executable" ok || check "installs newest" "$out"
out=$(run bash "$script"); [ -x "$installed" ] && check "running again updates in place" ok || check "rerun" "$out"

printf 'ORIGINAL' > "$installed"; echo "0000000000000000000000000000000000000000000000000000000000000000  $name" > "$srv/Q/r/releases/download/$tag/SHA256SUMS"
out=$(run bash "$script"); rc=$?; [ $rc -ne 0 ] && echo "$out" | grep -q "does not match its checksum" && [ "$(cat $installed)" = ORIGINAL ] && ! ls "$home/.local/bin" | grep -q "Rovarch.AppImage\." && check "bad checksum: refuses, keeps the old file, leaves no temp file" ok || check "bad checksum" "rc=$rc $out"

rm "$srv/Q/r/releases/download/$tag/SHA256SUMS"
out=$(run bash "$script"); [ $? -ne 0 ] && echo "$out" | grep -q "no SHA256SUMS" && check "missing SHA256SUMS: refuses" ok || check "missing sums" "$out"
printf '%s  other-file\n' "$sum" > "$srv/Q/r/releases/download/$tag/SHA256SUMS"
out=$(run bash "$script"); [ $? -ne 0 ] && echo "$out" | grep -q "do not list" && check "checksums that do not list the asset: refuses" ok || check "unlisted" "$out"
printf '%s  %s\n' "$sum" "$name" > "$srv/Q/r/releases/download/$tag/SHA256SUMS"

printf '[]' > "$srv/repos/Q/r/releases"
out=$(run bash "$script"); [ $? -ne 0 ] && echo "$out" | grep -q "no Rovarch release" && check "no releases: clear error" ok || check "no releases" "$out"
printf '[{"tag_name": "%s"}]' "$tag" > "$srv/repos/Q/r/releases"
out=$(run env ROVARCH_API_URL=http://127.0.0.1:1 bash "$script"); [ $? -ne 0 ] && echo "$out" | grep -q "could not look up" && check "unreachable API: clear error" ok || check "api down" "$out"

out=$(run bash "$script" --version 'v1; rm -rf /'); [ $? -ne 0 ] && echo "$out" | grep -q "not a release tag" && check "malicious --version is rejected before any network use" ok || check "bad tag" "$out"
out=$(run bash "$script" --version); [ $? -ne 0 ] && echo "$out" | grep -q "needs a release tag" && check "--version without a value" ok || check "version arg" "$out"
out=$(run bash "$script" --bogus); [ $? -ne 0 ] && echo "$out" | grep -q "unknown option" && check "unknown option" ok || check "unknown opt" "$out"
out=$(run bash "$script" --version v9.9.9); [ -x "$installed" ] && check "--version pins a release" ok || check "pin" "$out"

printf '#!/bin/sh\necho aarch64\n' > "$fakebin/uname"; chmod +x "$fakebin/uname"
out=$(run env PATH="$fakebin:$PATH" bash "$script"); [ $? -ne 0 ] && echo "$out" | grep -q "only has x86_64" && check "wrong architecture: refuses" ok || check "arch" "$out"

for t in git cc pkg-config; do printf '#!/bin/sh\n[ "$1" = "--atleast-version=4.18" ] && exit 1; [ "${1#--atleast}" != "$1" ] && exit 1; exit 0\n' > "$fakebin/$t"; chmod +x "$fakebin/$t"; done
printf '#!/bin/sh\necho "cargo 1.70.0 (x 2024-01-01)"\n' > "$fakebin/cargo"; chmod +x "$fakebin/cargo"; rm "$fakebin/uname"
out=$(run env PATH="$fakebin:$PATH" bash "$script" --native); [ $? -ne 0 ] && echo "$out" | grep -q "GTK 4.18" && echo "$out" | grep -q "libadwaita" && echo "$out" | grep -q "Rust 1.85" && check "--native lists every missing requirement and builds nothing" ok || check "native" "$out"

mkdir -p "$home/.local/share/rovarch/prefix" "$home/.config/rovarch" "$home/.local/share/applications"
echo x > "$home/.local/share/rovarch/prefix/cookie"; echo x > "$home/.config/rovarch/config.toml"
printf '[Desktop Entry]\nExec="%s" %%u\n' "$installed" > "$home/.local/share/applications/io.rovarch.app.RobloxLinks.desktop"
printf '[Desktop Entry]\nExec="%s" %%u\n' "$installed" > "$home/.local/share/applications/io.github.dratiux.Rovarch.RobloxLinks.desktop"
printf '[Desktop Entry]\nExec=firefox\n' > "$home/.local/share/applications/io.rovarch.app.Other.desktop"
run bash "$script" >/dev/null
out=$(run bash "$script" --uninstall); [ ! -e "$installed" ] && [ -f "$home/.local/share/rovarch/prefix/cookie" ] && [ ! -e "$home/.local/share/applications/io.rovarch.app.desktop" ] && [ ! -e "$home/.local/share/applications/io.rovarch.app.RobloxLinks.desktop" ] && [ ! -e "$home/.local/share/applications/io.github.dratiux.Rovarch.RobloxLinks.desktop" ] && [ -f "$home/.local/share/applications/io.rovarch.app.Other.desktop" ] && echo "$out" | grep -q "Kept" && check "--uninstall removes the app and its shortcuts, including the pre-io.rovarch.app ones, keeps data and unrelated entries" ok || check "uninstall" "$out | $(ls $home/.local/share/applications)"
out=$(run bash "$script" --uninstall --purge); [ ! -e "$home/.local/share/rovarch" ] && [ ! -e "$home/.config/rovarch" ] && check "--uninstall --purge deletes data and sign-in" ok || check "purge" "$out"
out=$(run bash "$script" --uninstall); [ $? -eq 0 ] && check "uninstalling when nothing is installed is not an error" ok || check "noop uninstall" "$out"

out=$(cat "$script" | run bash -s -- --help); echo "$out" | grep -q "Rovarch installer" && check "works when piped into bash (curl | bash)" ok || check "pipe" "$out"

kill "$(cat "$srv.pid")" 2>/dev/null
echo; echo "passed=$pass failed=$failed"
