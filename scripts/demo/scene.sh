#!/bin/bash
# Joue la scène dans le conteneur : un jeu à l'écran, un succès débloqué, puis la bibliothèque.
set -euo pipefail
ulimit -c 0
OUT=/src/target/demo
WIDTH=1600 HEIGHT=900
APP=/target/debug/prisma
FIXTURE=/target/debug/examples/steam_fixture
WORK=$(mktemp -d)

Xvfb :99 -screen 0 ${WIDTH}x${HEIGHT}x24 +extension Composite &
export DISPLAY=:99
sleep 1
xcompmgr -n &
feh --no-fehbg --bg-fill "$OUT/game.png"

"$FIXTURE" "$WORK/steam"
USER=Alex PRISMA_DATA_DIR="$WORK/data" PRISMA_STEAM_ROOT="$WORK/steam" "$APP" > "$OUT/prisma.log" 2>&1 &

# L'app est prête quand elle a enregistré les trois jeux du faux client Steam.
until grep -q "Low Orbit" "$WORK/data/library.json" 2>/dev/null; do sleep 0.5; done
sleep 3
MAIN=$(xdotool search --onlyvisible --name '^Prisma$' | head -1)
xdotool windowunmap "$MAIN"
sleep 1

ffmpeg -loglevel error -y -f x11grab -draw_mouse 0 -framerate 30 -video_size ${WIDTH}x${HEIGHT} -i :99 -t 20 "$OUT/demo.mp4" &
RECORDER=$!
sleep 2
"$FIXTURE" "$WORK/steam" unlock
# Le succès, puis le platine qu'il complète : deux cartes de 6 s.
sleep 13.5
xdotool windowmap "$MAIN"
xdotool windowmove "$MAIN" $(( (WIDTH - 1120) / 2 )) $(( (HEIGHT - 760) / 2 ))
wait "$RECORDER"
