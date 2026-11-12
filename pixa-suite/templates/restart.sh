#!/bin/sh
set -eu
pid=$1
root=$2
target=$3
staged="$root/@NAME@.app"
backup="$root/previous.app"
# Wait for a normal app exit; never kill the editor or bypass its save confirmation.
while kill -0 "$pid" 2>/dev/null; do /bin/sleep 1; done
# Revalidate the staged signature immediately before replacement.
/usr/bin/codesign --verify --deep --strict "$staged"
/usr/bin/codesign --display --verbose=4 "$staged" 2>&1 | /usr/bin/grep -qx 'TeamIdentifier=945M5X4DY3'
/usr/bin/codesign --display --verbose=4 "$staged" 2>&1 | /usr/bin/grep -qx 'Identifier=@BUNDLE@'
/usr/sbin/spctl --assess --type execute "$staged"
/bin/mv "$target" "$backup"
if ! /bin/mv "$staged" "$target"; then
    /bin/mv "$backup" "$target"
    /usr/bin/open "$target"
    exit 1
fi
if ! /usr/bin/open "$target"; then
    /bin/mv "$target" "$root/failed.app"
    /bin/mv "$backup" "$target"
    /usr/bin/open "$target"
    exit 1
fi
/bin/rm -rf "$root"
