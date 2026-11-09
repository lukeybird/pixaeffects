#!/bin/bash
# Run on the publisher's Mac. Credentials stay in its login Keychain.
set -euo pipefail
if [[ $(uname -s) != Darwin || $# != 1 || ! -f $1 ]]; then
  echo 'Usage on Mac: bash sign_update_macos.sh /path/to/pixaeffects-0.1.0-macos-universal.dmg' >&2
  exit 1
fi
identity='Developer ID Application: Luke Barger (945M5X4DY3)'
profile='PixaShop-Notary'
source_dmg=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
work=$(mktemp -d "${TMPDIR:-/tmp}/pixaeffects-release.XXXXXXXX")
mount="$work/mounted"
mkdir "$mount"
cleanup() {
  hdiutil detach "$mount" >/dev/null 2>&1 || true
  rm -rf "$work"
}
trap cleanup EXIT
hdiutil attach -readonly -nobrowse -mountpoint "$mount" "$source_dmg"
ditto "$mount/PixaEffects.app" "$work/PixaEffects.app"
hdiutil detach "$mount"
app="$work/PixaEffects.app"
version=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$app/Contents/Info.plist")
bundle=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$app/Contents/Info.plist")
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ && "$bundle" == com.aiwebdesignfirm.pixaeffects ]] || { echo 'Unexpected app identity' >&2; exit 1; }
# This Rust app packages one Mach-O executable and no embedded frameworks.
codesign --force --options runtime --timestamp --sign "$identity" "$app"
codesign --verify --deep --strict "$app"
ditto -c -k --keepParent "$app" "$work/notarize.zip"
xcrun notarytool submit "$work/notarize.zip" --keychain-profile "$profile" --wait --output-format plist > "$work/notary.plist"
[[ $(/usr/libexec/PlistBuddy -c 'Print :status' "$work/notary.plist") == Accepted ]] || { echo 'Apple did not accept the app. Do not publish.' >&2; exit 1; }
xcrun stapler staple "$app"
xcrun stapler validate "$app"
spctl --assess --type execute "$app"
# Preserve the existing drag-to-Applications layout while replacing its app bundle.
hdiutil convert "$source_dmg" -format UDRW -o "$work/editable.dmg"
hdiutil attach -nobrowse -mountpoint "$mount" "$work/editable.dmg"
rm -rf "$mount/PixaEffects.app"
ditto "$app" "$mount/PixaEffects.app"
hdiutil detach "$mount"
out="$HOME/Documents/PixaEffects-Release-$version"
mkdir -p "$out"
asset="$out/pixaeffects-$version-macos-universal.dmg"
[[ ! -e "$asset" ]] || { echo "Release file already exists: $asset" >&2; exit 1; }
hdiutil convert "$work/editable.dmg" -format UDZO -o "$asset"
codesign --timestamp --sign "$identity" "$asset"
xcrun notarytool submit "$asset" --keychain-profile "$profile" --wait --output-format plist > "$work/dmg-notary.plist"
[[ $(/usr/libexec/PlistBuddy -c 'Print :status' "$work/dmg-notary.plist") == Accepted ]] || { echo 'Apple did not accept the DMG. Do not publish.' >&2; exit 1; }
xcrun stapler staple "$asset"
xcrun stapler validate "$asset"
hdiutil attach -readonly -nobrowse -mountpoint "$mount" "$asset"
codesign --verify --deep --strict "$mount/PixaEffects.app"
spctl --assess --type execute "$mount/PixaEffects.app"
hdiutil detach "$mount"
shasum -a 256 "$asset" > "$asset.sha256"
echo "Signed, notarized release: $asset"
open "$out"
