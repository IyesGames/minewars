#!/bin/bash

# Generate game asset files from source SVGs

mkdir -p assets-tmp

s=64
s_min=16

level=0
while [ $s -ge $s_min ]
do
    s2=$(($s * 2))
    s4=$(($s * 4))
    s6=$(($s * 6))
    s8=$(($s * 8))
    s10=$(($s * 10))

    inkscape -w $s2 -h $s10 assets-src/digits.svg -o assets-tmp/digits-mip$level.png
    inkscape -w $s -h $s8 assets-src/gents.svg -o assets-tmp/gents-mip$level.png
    inkscape -w $s -h $s6 assets-src/flags.svg -o assets-tmp/flags-mip$level.png
    inkscape -w $s -h $s8 assets-src/tiles6.svg -o assets-tmp/tiles6-mip$level.png
    inkscape -w $s8 -h $s8 assets-src/roads6.svg -o assets-tmp/roads6-mip$level.png
    inkscape -w $s -h $s8 assets-src/tiles4.svg -o assets-tmp/tiles4-mip$level.png
    inkscape -w $s4 -h $s4 assets-src/roads4.svg -o assets-tmp/roads4-mip$level.png

    magick convert assets-tmp/digits-mip$level.png -crop 2x10@ +repage +adjoin PNG32:assets-tmp/out-digits-sprite%d-mip$level.png
    magick convert assets-tmp/gents-mip$level.png -crop 1x8@ +repage +adjoin PNG32:assets-tmp/out-gents-sprite%d-mip$level.png
    magick convert assets-tmp/flags-mip$level.png -crop 1x6@ +repage +adjoin PNG32:assets-tmp/out-flags-sprite%d-mip$level.png
    magick convert assets-tmp/tiles6-mip$level.png -crop 1x8@ +repage +adjoin PNG32:assets-tmp/out-tiles6-sprite%d-mip$level.png
    magick convert assets-tmp/roads6-mip$level.png -crop 8x8@ +repage +adjoin PNG32:assets-tmp/out-roads6-sprite%d-mip$level.png
    magick convert assets-tmp/tiles4-mip$level.png -crop 1x8@ +repage +adjoin PNG32:assets-tmp/out-tiles4-sprite%d-mip$level.png
    magick convert assets-tmp/roads4-mip$level.png -crop 4x4@ +repage +adjoin PNG32:assets-tmp/out-roads4-sprite%d-mip$level.png

    level=$(($level + 1))
    s=$(($s / 2))
done

KTXOPTS="--mipmap --levels $level --zcmp 19 --threads 1 --t2"
toktx --layers 20 $KTXOPTS assets/tilemap/digits.ktx2 assets-tmp/out-digits-*.png
toktx --layers 8 $KTXOPTS assets/tilemap/gents.ktx2 assets-tmp/out-gents-*.png
toktx --layers 6 $KTXOPTS assets/tilemap/flags.ktx2 assets-tmp/out-flags-*.png
toktx --layers 8 $KTXOPTS assets/tilemap/tiles6.ktx2 assets-tmp/out-tiles6-*.png
toktx --layers 64 $KTXOPTS assets/tilemap/roads6.ktx2 assets-tmp/out-roads6-*.png
toktx --layers 8 $KTXOPTS assets/tilemap/tiles4.ktx2 assets-tmp/out-tiles4-*.png
toktx --layers 16 $KTXOPTS assets/tilemap/roads4.ktx2 assets-tmp/out-roads4-*.png

rm -rf assets-tmp
