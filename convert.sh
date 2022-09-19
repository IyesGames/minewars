#!/bin/bash

# Generate game asset files from source SVGs

# TODO: when Bevy(_ecs_tilemap) supports it,
# generate texture arrays instead of spritesheets

for s in 16 24 32 40 48 56 64 72 80 96 128 192 256
do
    mkdir -p assets/sprites/$s

    s8=$(($s * 8))
    s4=$(($s * 4))
    s6=$(($s * 6))

    inkscape -w $s -h $s8 assets-src/digits.svg -o assets/sprites/$s/digits.png
    inkscape -w $s -h $s8 assets-src/gents.svg -o assets/sprites/$s/gents.png
    inkscape -w $s -h $s6 assets-src/flags.svg -o assets/sprites/$s/flags.png
    inkscape -w $s -h $s8 assets-src/tiles4.svg -o assets/sprites/$s/tiles4.png
    inkscape -w $s -h $s8 assets-src/tiles6.svg -o assets/sprites/$s/tiles6.png
    inkscape -w $s4 -h $s4 assets-src/roads4.svg -o assets/sprites/$s/roads4.png
    inkscape -w $s8 -h $s8 assets-src/roads6.svg -o assets/sprites/$s/roads6.png
done
