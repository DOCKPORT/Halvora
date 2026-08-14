#!/bin/bash
set -e

# The container mounts the host project root at /build. Everything below is
# relative to that working directory: the source tree, the assets that cargo
# embeds, and the binary/ dir that receives the output AppImage.
cd /build

echo "================================"
echo " Building Halvora (release)..."
echo "================================"
cargo build --release

echo "================================"
echo " Assembling AppDir..."
echo "================================"
APPDIR=/build/binary/Halvora.AppDir
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
cp target/release/halvora "$APPDIR/usr/bin/halvora"

# AppRun wrapper - suppresses harmless GTK warnings and bridge errors.
# GIO/GTK module unsetting is not needed: we ship no bundled libraries.
cat > "$APPDIR/AppRun" << 'APPRUNEOF'
#!/bin/bash
export NO_AT_BRIDGE=1
# Suppress harmless GTK module loading failures (e.g. xapp-gtk3-module missing)
exec 2> >(grep -v 'Failed to load module' >&2)
exec "${APPDIR}/usr/bin/halvora"
APPRUNEOF
chmod +x "$APPDIR/AppRun"

# Icon and .desktop file required by appimagetool
cp /build/binary/halvoralogo.svg "$APPDIR/halvoralogo.svg"
cat > "$APPDIR/halvora.desktop" << DESKEOF
[Desktop Entry]
Name=Halvora
Comment=Bitcoin halving tracker
Exec=halvora
Icon=halvoralogo
Type=Application
Categories=Office;Finance;Chart;
Terminal=false
DESKEOF

# AppStream metadata and screenshot so desktop environments and app stores
# can show a description and preview image. The metadata lives at the project
# root (metainfo/), mounted here at /build/metainfo/.
mkdir -p "$APPDIR/usr/share/metainfo"
cp /build/metainfo/halvora.appdata.xml "$APPDIR/usr/share/metainfo/halvora.appdata.xml"
cp /build/metainfo/halvora-screenshot.png "$APPDIR/usr/share/metainfo/halvora-screenshot.png"

echo "================================"
echo " Running appimagetool..."
echo "================================"
APPDIR="$APPDIR"
if [ ! -f /build/binary/appimagetool ]; then
    wget -q https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage -O /build/binary/appimagetool
    chmod +x /build/binary/appimagetool
fi

cd /build/binary
# Ignore the exit status of appimagetool: it exits non-zero on AppStream
# validation warnings, which are non-fatal. The real success check below
# confirms the AppImage was produced.
ARCH=x86_64 ./appimagetool --appimage-extract-and-run "$APPDIR" || true

# Fail if the AppImage was not produced (e.g. a real packaging error).
if [ ! -f "Halvora-x86_64.AppImage" ] && ! ls -1 Halvora*x86_64.AppImage >/dev/null 2>&1; then
    echo "Error: appimagetool did not produce an AppImage."
    exit 1
fi

# Rename the auto-generated output to our preferred convention.
# appimagetool names the output from the .desktop Name (Halvora -> Halvora-x86_64.AppImage).
NEW_OUTPUT="Halvora-x86_64.AppImage"
if [ ! -f "$NEW_OUTPUT" ]; then
    # Fallback in case appimagetool derived a different name.
    MATCHED=$(ls -1 Halvora*x86_64.AppImage 2>/dev/null | head -n 1)
    if [ -n "$MATCHED" ] && [ "$MATCHED" != "$NEW_OUTPUT" ]; then
        rm -f "$NEW_OUTPUT"
        mv "$MATCHED" "$NEW_OUTPUT"
    fi
fi

echo "================================"
echo " Done!"
echo " AppImage: /build/binary/$NEW_OUTPUT"
echo "================================"