# All Gameplay Actions

This page describes all gameplay actions that a player can perform.

Most actions have a **Cooldown** to prevent abuse. This is the amount of time
that must pass between successive invocations of the same action.

Some actions also have a **Delay**. This is the amount of time between when
the action is initiated and when it is completed.

The game client should display a progress indicator to visualize Cooldowns and
Delays. Cooldown indicators should be displayed within the game UI/HUD, where
the action is triggered. Delay indicators should be displayed on the tile where
the action is being performed.

## Capture Land

Cooldown: **0.125 sec**.
Delay: **0.25 sec**.

Attempt to capture a foreign land tile. Any items on the tile will be triggered
and their affects applied to the player. If the tile has a digit value of zero,
adjacent tiles will be automatically captured, recursively.

## Strike

Cooldown: **1.0 sec**.
Delay: **1.0 sec**.

Cause an **Explosion** on the given tiles. Destroys any **Items** on the
marked tiles. Converts the tile kind to **Destroyed Land**.

Costs money.

Can be applied to any tile that has a Visibility state above "Fog".

The game client should allow the user to first "mark"/"flag" the tiles
selected for a strike. This displays **Flags** on the map, on the marked
tiles. This happens entirely client-side. Then, the user can confirm
the selections, to launch the strike. The flags disappear and all the
marked tiles are submitted as a single action to the server.

## Reveal

Cooldown: **1.0 sec**.
Delay: **0.25 sec**.

Grants "Full Visibility" status to a single foreign tile, thus revealing any
items there.

The increased visibility state remains until any of the following occurs on the tile:
 - Explosion
 - Change of Ownership
 - Tile enters "Fog" state due to being outside of the visibility radius

Costs money.

Can be applied to any tile that has a Visibility state of "Limited".

## Smoke

Cooldown: **1.0 sec**.
Delay: **0.5 sec**.

Places a **Smoke** on an enemy tile, concealing the digit on that tile. The
other player cannot see any digit on that tile for the duration of the smoke.

This gives the player an opportunity deploy items on their territory, while
limiting how much information is revealed to the opponent.

The smoke lasts **5.0 sec**.

Costs money.

Can be applied to any tile that is owned by another player and has a Visibility
state above "Fog".

If an explosion occurs on a smoked tile, the smoke is instantly cleared.

## Deploy Item

Cooldown: **1.0 sec**.
Delay: **0.5 sec**.

Place an **Item** on a tile that is owned by the player.

The item is purchased using money from the city's region (if owned), or
the nearest city by distance (if unowned). The cost may be spread out
between multiple cities, if there are active road connections.

## Initiate Construction

Cooldown: **0.5 sec**.
Delay: **0.0 sec**.

Place a **Structure** on a tile that is owned by the player.

The player must own the city of the region.

The tile kind is immediately changed to **Foundation**.

The Structure is not ready immediately. When placed, it is in "pending"
mode, and the city is set to "construction" mode.

While a structure is "pending", it will not perform its intended effects.
Instead, it will accumulate "construction points" based on the city's resources.
The player should be able to see the progress.

While a city is in "construction" mode, it does not generate income.
Instead, its resources are counted towards the pending structures.

If there are multiple "pending" structures in a given region, they are processed
one-by-one, in a queue, in the order they were submitted by the player.

When building roads, all selected tiles are submitted as a single action.

## Bulldoze Structure

Cooldown: **2.0 sec**.
Delay: **5.0 sec**.

Remove a **Structure** from a tile, recovering part of its value.

## Harvest Tile

Cooldown: **1.0 sec**.
Delay: **2.0 sec**.

Instatly claim a sum of resources (counted either towards income or
construction, based on the current state of the region's city).

The player must own the region's city.

The tile kind is changed to **Destroyed Land**.

The amount gained depends on the tile kind.

The tile is converted to **Destroyed Land**, meaning it will have no resources
for the rest of the game.
