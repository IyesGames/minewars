#!/bin/bash

# Generate gfx2d game asset files from source SVGs

mkdir -p assets-tmp

s=128
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
    inkscape -w $s10 -h $s10 sprites.svg -o assets-tmp/sprites-mip$level.png

    level=$(($level + 1))
    s=$(($s / 2))
done

KTXOPTS="--mipmap --levels $level --zcmp 19 --threads 1 --t2"
toktx $KTXOPTS ../assets/tilemap/sprites.ktx2 assets-tmp/sprites-mip*.png
toktx $KTXOPTS ../assets/tilemap/roads6.ktx2 assets-tmp/roads6-mip*.png
toktx $KTXOPTS ../assets/tilemap/roads4.ktx2 assets-tmp/roads4-mip*.png

rm -rf assets-tmp
