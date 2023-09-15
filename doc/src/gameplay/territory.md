# Territory and Ownership

MineWars is a game of territory. Players must expand to claim resources and
progress through the game.

## Visibility

Areas away from the player's territory are covered by "fog of war". The
player can see **3** tiles away from tiles they own.

The entire map's geography is shown to all players at all times, there are no
areas that are completely "black". Fog of war simply means that the player does
not get updates on the status of the tile / any gameplay that happens there.

Any changes to the tile kind behind fog of war are not revealed, so a player's
view of the map geography can be outdated.

Previous ownership is not shown behind fog of war. Tiles should have a neutral
color.

Mountain tiles are always treated as a cluster. If you can see one tile of
the cluster, you see them all.

There are technically 3 levels of visibility, showing increasing amounts of info:
 - Fog: any tile further than the visibility radius away from a player-owned tile
 - Limited: any tile not owned by the player, within the visibility radius
 - Full: tiles owned by the player, tiles within the range of a **Watch Tower**, tiles where a **City** is located

Within "Fog Visibility" tiles, the player sees:
 - Last known tile kind / geography (may be outdated)
 - Skulls

Within "Limited Visibility" tiles, the player also sees:
 - Current tile kind (if changed due to harvesting, explosions, etc.)
 - The ownership of the tile (displayed using the owner's color)
 - Completed Structures
 - Smokes
 - Explosions

Within "Full Visibility" tiles, the player also sees:
 - Items
 - Pending Structures

## Ownership of Territory

Each tile on the map can have an owner: one of the players in the game. The
owner is indicated by the tile's color. Every player has a color. For
accessibility reasons, the colors should be customizable client-side. Players do
not get to pick the exact colors displayed to others.

At the start of the game, all tiles are neutral, except for the starting cities
of the players and all tiles adjacent to them.

Owned tiles provide **Resources** to the city of the region they are associated
with, if the player owns the city.

On owned **Land**, the player gets:
 - the ability to deploy **Items** on the tile
 - the ability to perform **offensive actions** on adjacent unowned tiles
 - the ability to **build structures**, if the player owns the region's city
 - the ability to **Harvest** the tile, if the player owns the region's city
 - visibility of surrounding tiles
 - a digit (like Minesweeper) indicating how many (if any) items exist on adjacent unowned tiles

("unowned" here means: neutral or owned by another player)

## Capturing Territory

### Land

Land is captured simply by clicking on it. You can only do it if you own at
least one adjacent tile.

After being captured, the land is **protected** for **2.5** sec, meaning it
cannot be captured by another player. This gives the new owner a chance to
deploy items to defend it, preventing the game from turning into a clickfest.

If the tile is known empty (there is no adjacent digit), the game will
autoexpand. All safe land (up to any tiles with digits) will be recursively
captured instantly, like in Minesweeper.

If there is an adjacent digit, that indicates the tile is potentially unsafe --
a risk to the player.

If the tile contains any **Items**, their respective effects will be triggered
when the player attempts to capture the tile (see their respective docs for more detail):
 - **Mines** will **explode**, causing the player to be **stunned**
 - **Decoys** will **break** / do nothing, and the tile will be captured as if it was empty
 - **Flashbang Traps** will "blind" the player

Player strategy: if they think the tile is empty or a decoy, they should capture
it, breaking any decoy. If they think the tile contains a dangerous item, they
can either **Strike the tile** to destroy it, or try to safely surround and
**capture the item**.

### Clusters

**Mountains** and **Forests** are always owned as a cluster. If there are multiple
adjacent tiles of one of these types, the player cannot own them individually;
they are captured all at once and lost all at once.

To capture a cluster, surround it on all sides by capturing all adjacent land
tiles. If there are multiple adjacent clusters (say, a forest next to a
mountain), or if any of the adjacent land contains **Items** or **Structures**,
all of those tiles are grouped together for the purpose of capturing. The player
must surround the entire group (capture all adjacent safe tiles), gaining
control of all the clusters, items, and structures, at once.

A player only maintains ownership of a mountain or forest cluster *while* they
maintain ownership of *all* adjacent land tiles. You immediately lose ownership
of the entire cluster if another player captures even a single adjacent land
tile.

Note that only land tiles are counted. If there are adjacent clusters (forest
next to a mountain), it is possible to lose one while keeping the other.

### Items

When there are **Items** deployed on **Land**, you can capture the items by
surrounding them. Capture all safe tiles around them.

If there are multiple adjacent tiles with items on them, they are treated
as a Cluster, similar to mountains/forests.

When the items have been successfully surrounded, all the land tiles are
automatically captured by the player. The items are **sold** for money.
See the Economy section for more info.

Player strategy implications: This rewards the player for playing the game
carefully and taking the time to treat foreign items as a puzzle to be solved,
rather than simply striking the tiles to destroy the items. Striking is quick and
easy, but costs money, and destroys the land's resources. Capturing the items
is time-consuming and requires mental effort to solve the minesweeper digits
puzzle, but rewards the player with money and keeps the land intact.

### Cities

To capture a city, the player must surround the city, by capturing every land
tile adjacent to the city's tile. The player must then hold all adjacent tiles
for a duration of **5.0 sec**.

The city remains owned by the player until another player does the same
(captures every adjacent land tile and maintains ownership for duration).

See the **Economy** section for further implications of capturing cities.

### Structures

#### Roads

Road tiles are treated as land tiles for the purpose of capturing.

However, note that cities only count as connected, if there is a path
that is fully owned by the player. Upon losing even a single tile,
the connection is broken.

#### Barricades

Given that barricades connect mountain clusters, they have implications
for the ownership of those clusters. They make it effectively impossible
to capture each of the mountain clusters individually. However, each
cluster can still be lost individually.

When a player surrounds both clusters together with the barricade between
them, they capture the barricade along with both clusters.

#### Watch Towers

Watch Towers are captured by surrounding them, similar to cities. If they
are built adjacent to a cluster, they need to be captured together with
that cluster. If they are built between clusters, they effectively work
as a Barricade.

#### Bridges

Bridges are built on water, and therefore have no ownership. However,
they allow a player to capture tiles on the opposite side of the water,
if they own tiles on one side.

