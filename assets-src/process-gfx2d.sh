#!/bin/bash

# Generate gfx2d game asset files from source SVGs

mkdir -p assets-tmp

s=256
s_min=16

level=0
while [ $s -ge $s_min ]
do
    s2=$(($s * 2))
    s4=$(($s * 4))
    s6=$(($s * 6))
    s8=$(($s * 8))
    s10=$(($s * 10))

    inkscape -w $s8 -h $s8 roads6.svg -o assets-tmp/roads6-mip$level.png
    inkscape -w $s4 -h $s4 roads4.svg -o assets-tmp/roads4-mip$level.png
    inkscape -w $s10 -h $s10 numbers.svg -o assets-tmp/numbers-mip$level.png
    inkscape -w $s10 -h $s2 digits.svg -o assets-tmp/digits-mip$level.png
    inkscape -w $s10 -h $s2 tiles.svg -o assets-tmp/tiles-mip$level.png
    inkscape -w $s8 -h $s2 explosions.svg -o assets-tmp/explosions-mip$level.png
    inkscape -w $s4 -h $s4 gents.svg -o assets-tmp/gents-mip$level.png
    inkscape -w $s8 -h $s4 ui-icons.svg -o assets-tmp/ui-icons-mip$level.png
    inkscape -w $s4 -h $s flags.svg -o assets-tmp/flags-mip$level.png

    level=$(($level + 1))
    s=$(($s / 2))
done

KTXOPTS="--mipmap --levels $level --zcmp 19 --threads 1 --t2"

mkdir -p ../assets/sprites/
toktx $KTXOPTS ../assets/sprites/roads6.ktx2 assets-tmp/roads6-mip*.png
toktx $KTXOPTS ../assets/sprites/roads4.ktx2 assets-tmp/roads4-mip*.png
toktx $KTXOPTS ../assets/sprites/numbers.ktx2 assets-tmp/numbers-mip*.png
toktx $KTXOPTS ../assets/sprites/digits.ktx2 assets-tmp/digits-mip*.png
toktx $KTXOPTS ../assets/sprites/tiles.ktx2 assets-tmp/tiles-mip*.png
toktx $KTXOPTS ../assets/sprites/explosions.ktx2 assets-tmp/explosions-mip*.png
toktx $KTXOPTS ../assets/sprites/gents.ktx2 assets-tmp/gents-mip*.png
toktx $KTXOPTS ../assets/sprites/ui-icons.ktx2 assets-tmp/ui-icons-mip*.png
toktx $KTXOPTS ../assets/sprites/flags.ktx2 assets-tmp/flags-mip*.png

rm -rf assets-tmp
