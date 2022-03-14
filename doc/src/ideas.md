# More Ideas to Explore

This is a list of additional gameplay ideas to be explored after we have a playable demo/MVP.

## Grid Style

Compare whether the game plays better on a hexagonal grid or a square grid.

Strong personal preference for hexagonal, but we should play-test both.

## Game Modes and Configurations

It is possible to imagine multiple variants of the game:
 - Free-for-all: 3-6 players on a medium-sized map
 - Duos: same, but each territory is simultaneously/cooperatively controlled by 2 players
 - Duels: 2 players on a smaller map
 - (maybe?) Battle-Royale: huge map with shrinking playable area; 50, 100, or even more players

## Per-city inventories

Count mines/decoys separately based on which city produced them.

Give captured mines to the nearest city/region.

When the player wants to deploy mines/decoys, take from the nearest city/region
that has any available.

This would allow for "divide-and-conquer" strategies, where a player could
capture land to split another player's territory through the middle, forcing
them to defend each half separately, using only the supplies produced by the
cities there. This may allow for more interesting late-game "comebacks" where
the underdog turns the game around by splitting the winning player's territory.

TODO: think about UI/UX design to make this not confusing.

## Alliances

Temporary friendly relations between players.

During the game, a player can propose an **alliance** to another player,
which the other player can accept or reject.

The following **extra rules** apply to **allied players**:
 - Shared visibility
   - Reveals land ownership, cities, roads
   - Deployed mines are *not* revealed
   - Digits are *not* revealed
 - Protection from offenses
   - Cannot capture each other's land
   - Cannot activate/explode mines along the border
   - Mines do not count towards digits
 - Trade
   - Can benefit from each other's Export Resources by connecting each other's cities with roads
   - Can offer to transfer mines from one player's inventory to the other
 - (maybe?) in-game chat / comms

The alliance cannot be voluntarily broken by the players. Players remain
allied until they are the only ones left in the game. If all other players
are eliminated, the alliance automatically breaks. They must then fight each
other to determine the final winner of the game.

## Watch Towers

Watch towers give *permanent* visibility of the surrounding area (radius of
**2** tiles) for the entire duration of the game, to the player that initially
constructed them, even if the region's city or surrounding land is captured.

They are permanently owned by the player that initially constructed them
and cannot be captured.

Cities can be ordered to construct watch towers located at a player-chosen
land tile.

Watch towers must be expensive, much slower to produce than mines or roads.

(maybe?) limit to building at most one watch tower per region/city

Must be placed within the region of the city tasked to build the watch tower,
on a land tile owned by the player.

(?) to be decided:
 - additional requirement:
   - distance away from the border
 - during construction:
   - do we reserve the tile? (disallow it to be captured by another player)
   - do we allow another player to capture the tile and cancel the construction?
     - what happens to spent resources?

## Terraforming

Allow specific kinds of terraforming maneuvers:
 - Turning a **mountain** tile into a **land** tile
 - Turning a **water** tile into a **land** tile
   - only if there is adjacent **land**
 - Turning a **land** tile into a **water** tile
   - only if there is adjacent **water**
   - map must remain fully contiguous

These operations are performed as production by the city in the region where
they take place. That is, the player must spend that city's production and
resources on terraforming.

(?) to be decided: resource costs
