# Structures

There are several structures that can be built by players to give them a stategic
advantage. Structures are built on land, using the production points of the city
whose region the land is part of. The player must own the city and the land.

When a structure is placed on a tile, the underlying tile kind is changed to
**Foundation**.

Structures can be bulldozed to reclaim part of their value. After bulldozing,
the underlying tile kind is changed to **Destroyed Land**.

Structures are destructable. They take **1** damage from adjacent explosions and
**2** damage from direct explosions (on the same tile).

**Strike** can be peformed on structures, causing direct damage. Adjacent damage
can be caused by nearby mine explosions or strikes.

## Roads

HP: **3**.

Roads serve to connect cities. An active road connection brings numerous benefits.

Roads do not have a *direction*, they are simply a kind of grid tile; each
tile either has road built on it, or not. The "path" / pathfinding algorithm
simply follows grid tiles. That is, a road tile is considered connected to any
adjacent road tiles. (think Civ-like road behavior)

Two cities are considered to have an active road connection iff:
 - there is an uninterrupted path of roads between them
   (the pathfinding algo must be able to reach one city from the other by following road tiles)
 - both cities are owned by the same player
 - all tiles along the path must be owned by the player
 - the path may not cross other regions / all tiles along the path must be part of either city's region

Roads allow gameplay to still occur on the tile. The tile is playable like a
land tile. The tile can be captured like land, Items can be placed, Digits are
displayed, etc.

Therefore, it is possible for a mine to explode directly on a road. This
is another possible way to damage roads, as well as a **Strike**.

Note that this means an enemy can interrupt a road connection between two cities
simply by capturing or destroying a single road tile along it. Roads are very
vulnerable. This encourages players to invest in redundant road networks with
backup paths.

## Mountain Barricades

HP: **8**.

Mountain Barricades block mobility by closing gaps/chokepoints between mountains.

The player must own the land tile to build on, the city of the tile's region, and
also the adjacent mountains.

The barricade effectively prevents the opponent from capturing ownership of
either mountain cluster, unless they surround everything and capture both
mountain clusters + the barricade all at once, or destroy the barricade.

A barricade longer than one tile is counted as a single structure. It is
constructed all at once and destroyed all at once. Cost adds up, but HP does
not. Therefore, longer barricades become prohibitive to build and vulnerable /
easier to destroy.

## Watch Towers

HP: **5**.

Watch towers give "Full Visibility" of the surrounding area (radius of **5**
tiles).

They can be built on any player-owned land tile, as long as the player owns the
city of the region. If adjacent to a mountain or forest, the player must also
own the mountain or forest.

They can be captured in a way similar to cities: the opponent must surround them
by capturing the adjacent tiles.

If built adjacent to mountains/forest, they need to be captured with the
mountain cluster.

Note: towers can effecively be used in place of barricades. A tower, if built
between two mountain clusters, will effectively behave like a barricade that
also gives visibility.

Hence, balance must be kept to keep both structures viable. Towers should be
more expensive to build and easier to destroy (in that scenario, not too easy to
destroy on open land either).

## Bridges

HP: **4**.

Bridges can be built on **Water** tiles, as long as there is **Land** on two
opposing adjacent tiles. Effectively, this limits them to straight sections
of single-tile-thick rivers.

Bridges allow land to be captured and roads to be built across the water,
thus reducing the impact of the rivers as a geographical feature to segment
the map and impede movement.

