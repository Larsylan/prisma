#!/bin/bash
# Enregistre la démo animée du README (docs/demo.gif) dans un Linux virtuel : il faut Docker.
set -euo pipefail
cd "$(dirname "$0")/../.."
OUT=target/demo
mkdir -p "$OUT"
# GIF de 960 px de large à 15 images par seconde, avec une palette calculée sur la vidéo.
GIF_FILTER="fps=15,scale=960:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=192[p];[b][p]paletteuse=dither=sierra2_4a"

npm run build
docker build -q -t prisma-demo scripts/demo
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --disable-gpu --hide-scrollbars \
    --window-size=1600,900 --screenshot="$PWD/$OUT/game.png" "file://$PWD/scripts/demo/game.html" 2>/dev/null

docker run --rm -v "$PWD":/src -v prisma-demo-target:/target -v prisma-demo-cargo:/root/.cargo/registry \
    -e CARGO_TARGET_DIR=/target -w /src prisma-demo bash -c "
        cargo build -q --locked -p prisma --features tauri/custom-protocol &&
        cargo build -q --locked -p prisma-core --features test-support --example steam_fixture &&
        dbus-run-session -- bash scripts/demo/scene.sh &&
        ffmpeg -loglevel error -y -i $OUT/demo.mp4 -vf '$GIF_FILTER' docs/demo.gif"
echo "✅ docs/demo.gif"
