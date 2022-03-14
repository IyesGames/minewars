# Gameplay and Rules

Note: any exact values (like numbers for costs, etc) are tentative and
subject to change, as we play-test and balance the game.

Note: many of the concepts are interconnected; by necessity, this document
cannot introduce them in isolation, and each section may refer to concepts
not yet explained (if you are reading in order).

## Gameplay Objective

In MineWars, a number of players compete for ownership over **cities**
and **territory**.

There are a number of **cities** in various locations on the map. Each
player begins the game with one randomly-assigned **starting city**.

A player must own *at least one city* at all times; otherwise is eliminated
from the game. The objective of the game is to eliminate other players.

## The Map

The game map is a procedurally-generated fully-contiguous island/continent.

There are 4 types of tiles (geographical features).

**Water** is the outer part of the map, outside of the playable area. Non-interactive.

**Land** is an interactive tile, forming the playable area.

All land within **2** tiles of water is **Fertile Land**, which gives extra
resources. Besides the resources, this tile behaves identically to **Land**.

**Mountains** are generated in random clusters of multiple adjacent tiles. They give
large amounts of resources and can be owned by players, but are otherwise non-interactive.

## Cities

At map generation time, a number of **cities** are placed. The generation algorithm
aims to make them roughly uniformly spread out.

All tiles adjacent to the city tile must be playable **land**, cities cannot
be located next to **mountains** or **water**.

There will always be more cities than the number of players in the session,
meaning that at least some cities start neutral.

The location of cities is unknown until discovered, the player has to explore to find them.

## Regions

Each tile on the map is associated with a city. All tiles that are associated
with a given city are collectively known as that city's **region**.

The region is the city's "area of influence". If a given player owns the city,
resources from any tiles within the region that are owned by that player are
counted towards the city, and the player may order the city to construct
things on tiles within the region.

Hence, the size of the region and the geography within it, determine the
**power** of the city. There will always be a power imbalance in every map
generated: some cities will be more powerful, with access to more land and
resources, than others.

The regions are determined as follows (at map generation time):
 - **land** tiles belong to the nearest city (Voronoi-like)
 - **mountain** tiles are assigned by cluster
   - all tiles in a cluster are given to the region that would have the majority of the cluster if the tiles were land
 - Any disputed tiles are given to the city with the larger region (to increase the power imbalance)

## Starting Cities

The **starting cities** at the beginning of the game are chosen, so that
they are among the most powerful cities on the map (calculated as the sum
of **local resources** and **import resources**).

(sort by city power and randomly assign the best N cities to players)

## Production

Every city is always working on something, known as the **current production
item**. The default, unless overriden by the player, is to produce a random
mix of **mines** and **decoys**. The player may assign the city to work on
something specific instead.

Neutral cities do not perform any production.

## Resources

Each city's rate of production is determined by how many **resources** it
has available (sum of Local + Import + Base). The numbers indicate points
accumulated per second.

There are 4 kinds of resources:
 - **Local resources**: counted for the local city, not counted towards other cities
 - **Export resources**: do not count towards the local city; instead, count towards each neighboring city, if there is an active road connection
   - (these are the import resources of connected cities)
 - **Import resources**: count towards the local city, if there is an active road connection
   - (these are the export resources of connected cities)
 - **Base resources**: each city starts with **25** resource points, regardless of owned territory

Local and Export Resources come from all the tiles within the city's region,
that are owned by the player that owns the city. Import resources come from
any connected neighboring cities.

**Note:** this means, that:
 - Export resources are useless/ignored if there are no active road connections
 - Export resources are *multiplied* with every road connection; each connected city gets the exact same full amount of import resources, as the local city has export resources; this makes them *very powerful*

Resources given by each tile owned:
 - **Land** gives **1** local resource
 - **Fertile Land** gives **1** local resource + **1** export resource
 - **Mountain** gives **5** local resources + **3** export resources

## Production Costs

The cost of everything is determined using a multiplier applied to a **Base
Unit**; this helps negate the unpredictability of the random procedural world
generation. This is calculated once at the start of the session, based on the
generated map, and remains fixed throughout the game.

The **Base Unit** is roughly representative of how powerful cities could
get in a given game session, and is calculated as follows:

 1. The maximum accessible resources of any given city are calculated as the sum of its maximum **local resources** and maximum **import resources** (assuming road connections to all neighbors).
 2. Take the highest value of any city on the map
 3. Divide by the total number of cities on the map

This should help give relatively well-balanced costs with varying map configurations.

Cost sheet (TODO playtest and balance):

 - Mines: 5.0 * BaseUnit
 - Decoys: 5.0 * BaseUnit
 - Roads (per tile): 1.5 * BaseUnit

(dev note): balancing rationale:
 - mines/decoys are equal for now, playtesting will show if one or the other is more important
 - players should be getting approx 1 new mine/decoy every 5-15 seconds late-game if they own a large portion of the map
 - roads should be expensive enough to make them a tough sacrifice in terms of opportunity cost of producing mines instead
 - roads should be relatively cheap, as you need to build a lot of them for them to be useful

## Roads

Roads greatly boost the production rate of cities, by giving them extra
resources. An active road connection between two cities adds each city's
**export resources** to the other's total **import resources**.

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

Note that this means an enemy can interrupt a player's road connection simply
by capturing a single road tile along it. This encourages players to invest
in redundant road networks with backup paths.

Roads can be built with a "road build tool", where the player can use the
mouse to select all the tiles to build roads on. When confirmed by the player,
the pending tiles in each region will be added to the respective city's queue,
replacing any other currently active production item. This means that a city
cannot produce mines (or anything else) while it is building roads.

Roads can be built on any land tile.

Any tile with a road on it loses its resources (gives 0 local and export resources).

Roads cannot be removed, once built.

## Visibility

Areas away from the player's territory are covered by "fog of war". The
player can only see **2** tiles away from their territory.

The entire map's geography is visible to all players at all times, there
are no areas that are completely "black". Fog of war simply means that the
player does not get updates on the status of the tile, like its ownership,
or any game-entities that reside there.

Mountain tiles are always treated as a cluster. If you can see one tile of
the cluster, you see them all.

## Ownership

Tiles, along with anything on them, can be owned by a player.

They give resources to the city of the region they are associated with, iff the player owns the city.

On owned **Land**, the player gets:
 - the ability to place mines and decoys there
 - the ability to activate a mine there and cause an explosion
 - visibility of surrounding tiles
 - a digit indicating how many (if any) mines and decoys exist on adjacent non-player-owned tiles (like Minesweeper)
 - the ability to modify the tile (with roads) using production from the region's city
 - the ability to capture adjacent land tiles

## Capturing Land

Land is captured simply by clicking on it. You can only do it if you own at least one adjacent tile.

You are taking a risk – if there is a [**mine**](#mines) on the tile,
it will [explode](#explosions), and you will be [**stunned**](#stun).

If there is a [**decoy**](#decoy), it will break.

For ergonomics, if there is no digit on a tile you own (indicating no adjacent
mines, safe on all sides), you may click on that tile and instantly capture
adjacent land tiles with one click.

## Capturing Cities

To capture a city, the player must surround the city, by capturing every
land tile adjacent to the city's tile.

The city remains owned by the player until another player does the same
(captures every adjacent land tile).

When a city is captured, its current production item is set to the default.

## Capturing Mines

Mines from adjacent territories can be captured. If you surround the mine(s)
by capturing all adjacent land tiles, you will be awarded those mines and
the land they reside on. The mines are added to your inventory. You gain
ownership of the land tiles where the mines were located.

Mines on adjacent tiles are grouped together, also along with mountain
clusters. If there are multiple adjacent tiles containing mines and/or
mountains, you can only capture them all together as a group, by capturing
the surronding adjacent land around the whole group.

## Capturing Mountains

Mountains are owned as a cluster.

You only own the mountain cluster *while* you maintain ownership of *all*
adjacent land. To capture a mountain cluster, surround it on all sides by
capturing all adjacent land tiles. You immediately lose ownership of the
entire cluster if another player captures even a single adjacent land tile.

Mountain clusters are grouped together with mines for capturing purposes. If
there are multiple adjacent tiles containing mines and/or mountains, you
can only capture them all together as a group, by capturing the surronding
adjacent land around the whole group.

## Stun

If you step on a **mine**, you get a **stun**. This is a cooldown during
which you cannot perform any game actions. All your cities pause production
for the duration of the stun.

The stun will increase in duration if you get stunned repeatedly in a short period of time.

The base stun duration is **5** seconds.

## Inventory

You have an **inventory**, which is your arsenal of mines and decoys that
are not currently deployed anywhere on the map.

When your cities produce a new mine or decoy, it is added to your inventory.
When you capture mines from the world, they are added to your inventory.

You can deploy mines and decoys from your inventory to any map tile, or pick
back up any deployed ones, with a single click.

## Mines

Mines are your main offensive and defensive weapon.

If you suspect there is a mine on an adjacent tile you do not own, your possible courses of action are:
 - [Sacrifice mines from your inventory to blow it up](#offense)
 - [Try to surround and capture it by capturing adjacent safe tiles](#capturing-mines)
 - [Risk yourself and step on the tile anyway; possibly causing an explosion and getting stunned](#capturing-land)
   - You may just get lucky and it turns out to be a [decoy](#decoy)
 - Ignore it and go do something else ;)

### Defense

You can use mines defensively, by placing them on land tiles you own.

If another player tries to capture your territory, they will have to be
careful to not step on them and stun themselves. They will see digits on
their tiles, hinting them about the mines.

Learn to make different formations of mines + decoys, taking geography into
account, for an effective defense.

### Offense

You can use a mine from your inventory to **activate** another mine placed
somewhere on your territory, near your border.

The activated mine will explode after a short **1.0** second fuse time. During
this time, it will be visible to any player with visibility of the land tile
it is on (likely including your enemy).

The explosion will chain – it will also explode any mines on adjacent
tiles, allowing you to take out mines across the border, that do not belong
to you. (and also exploding any of your own adjacent mines)

Note: this means you are sacrificing 2 mines to cause an explosion.

This is useful for taking out enemy mines. This is also useful during the
early-game exploration stage – if there is a neutral cluster of mines that
is difficult to surround and capture, you may choose to blow it up rather
than capturing it, if you value your time more than you value the mines.

## Decoys

Decoys can be placed on tiles, just like mines, but are safe to step on. They
do not explode; instead they simply break.

They count towards the digits shown to players, just like mines, causing confusion.

They are very valuable in defensive formations, when mixed with mines, to confuse
the enemy with digits.

As they are non-explosive, they also serve to break up explosion chains. An
adjacent exploding mine will break/destroy the decoy, but the explosion will
not propagate further.

## Starting Mines and Decoys

On world generation, a large number of mines and decoys are randomly scattered
around the map, in all the neutral unexplored areas.

This serves to impede the initial expansion of players' territories, and to
provide players with a source of mines during the early-game, while their
cities' production is still rather slow due to a lack of resources.

The early-game stage of MineWars, before encountering another player, is
hence more like a glorified single-player minesweeper experience, as each
player tries to form some initial territory and prepare for their first enemy
encounter, while being mindful and trying to figure out the randomly-placed
initial mines.

## Explosions

All explosions in the game chain. Whenever a mine explodes, it also
simultaneously causes all adjacent mines to explode, recursively, allowing
for long chains. Explosions also break any adjacent decoys.

Explosions are caused when a player tries to **capture a land tile that is
mined** (awarding the player a **stun**), or intentionally by [**activating
mines**](#offense).

## Losing Condition

If a player loses the last city they own, they are out of the game.

All territory they own becomes neutral.

Their inventory contents are randomly scattered across all neutral territory on the map.

They may continue to spectate the game.

## Winning Condition

If a player is the last one standing, they are the winner of the game.
