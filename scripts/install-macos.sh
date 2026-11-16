#!/bin/zsh
# Compile Prisma, l'installe dans /Applications et retire toute autre copie connue de macOS.
set -euo pipefail
cd "$(dirname "$0")/.."

NAME="Prisma"
BUNDLE_ID="io.github.larsylan.prisma"
BUILT="target/release/bundle/macos/$NAME.app"
INSTALLED="/Applications/$NAME.app"
LSREGISTER=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister

npm ci
npm run tauri build -- --bundles app

osascript -e "quit app \"$NAME\"" 2>/dev/null || true
rm -rf "$INSTALLED"
cp -R "$BUILT" /Applications/

# La copie de travail resterait visible dans Spotlight et Launchpad.
"$LSREGISTER" -u "$BUILT" 2>/dev/null || true
rm -rf "$BUILT"
mdfind "kMDItemCFBundleIdentifier == '$BUNDLE_ID'" 2>/dev/null | while read -r copy; do
    [[ "$copy" == "$INSTALLED" ]] && continue
    echo "🧹 Ancienne copie retirée : $copy"
    "$LSREGISTER" -u "$copy" 2>/dev/null || true
done
"$LSREGISTER" -f -R "$INSTALLED"

open "$INSTALLED"
echo "✅ Installé dans $INSTALLED"
