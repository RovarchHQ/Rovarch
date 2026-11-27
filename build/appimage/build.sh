#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$(realpath "$0")")"

tools="$PWD/tools"
appdir="$PWD/AppDir"
gtk_hook="$appdir/apprun-hooks/linuxdeploy-plugin-gtk.sh"

download_tool() {
    local name="$1" url="$2"
    if [ ! -x "$tools/$name" ]; then
        curl -fsSL -o "$tools/$name" "$url"
        chmod +x "$tools/$name"
    fi
}

mkdir -p "$tools"
download_tool linuxdeploy-x86_64.AppImage \
    https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
download_tool linuxdeploy-plugin-gtk.sh \
    https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh

cargo build --release --manifest-path ../../Cargo.toml

export PATH="$tools:$PATH"
export DEPLOY_GTK_VERSION=4

icon_files=()
for icon in ../../data/icons/hicolor/*/apps/io.rovarch.app.png; do
    icon_files+=(--icon-file "$icon")
done

rm -rf "$appdir"
linuxdeploy-x86_64.AppImage \
    --appdir "$appdir" \
    --executable ../../target/release/rovarch \
    --desktop-file ../../data/io.rovarch.app.desktop \
    "${icon_files[@]}" \
    --plugin gtk

sed -i '/^export GTK_THEME=/d; /^export GDK_BACKEND=/d' "$gtk_hook"

bundled_icons="$appdir/usr/share/icons/Adwaita"
mkdir -p "$bundled_icons"
cp -r /usr/share/icons/Adwaita/{index.theme,scalable,symbolic} "$bundled_icons/"
gtk4-update-icon-cache --force --quiet "$bundled_icons"

linuxdeploy-x86_64.AppImage --appdir "$appdir" --output appimage

generated=$(find "$PWD" -maxdepth 1 -name "Rovarch-*.AppImage" -print -quit)
chmod +x "$generated"
echo "AppImage: $generated"
