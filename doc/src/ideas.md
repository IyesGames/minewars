# More Ideas to Explore

This is a list of additional gameplay ideas to be explored after we have a playable demo/MVP.

## Grid Style

Compare whether the game plays better on a hexagonal grid or a square grid.

Strong personal preference for hexagonal, but maybe we should play-test both?

## Alliances

Temporary friendly relations between players.

During the game, a player can propose an **alliance** to another player,
which the other player can accept or reject.

The following **extra rules** apply to **allied players**:
 - Full visibility of each others' territory
   - Digits are *not* revealed
 - Protection from offenses
   - Cannot capture each other's land
   - Cannot use Smoke
   - Cannot Strike in each other's territory or adjacent to the border
 - Trade
   - Can connect each others' cities with roads
 - (maybe?) in-game chat / comms

The alliance can be canceled by either player at any time.

If all other players are eliminated, the alliance automatically breaks. They
must then fight each other to determine the final winner of the game.

## Premade Maps

Perhaps it could be interesting to play on a handcrafted map, instead of
procedurally generated?

There is a replayability argument to be made.

Procedurally generated maps make the game replayable, because every match is
unique. Player skill comes from adaptability -- being able to figure out the
best strategy on an unknown map.

Fixed maps make the game replayable, because players can learn them, and apply
better strategies the next time they play. Similar to games like Counter-Strike,
this raises the skill ceiling, as players can learn a map and rely on their
familiarity and knowledge of how to play it most effectively.

For this to work at its best, we need a map/scenario editor.

As a cheap alternative, we can also provide the ability to specify a seed for
the usual procedural generation algorithm. However, that would obviously only
allow replaying the same randomly generated map multiple times. It would not
enable completely custom handcrafted maps.

## City Population / Sustenance Cost

This is just a reserve game design idea that should *only* be added if the game
otherwise proves to be very challenging to balance (particularly in the
late-game when players have a large and very productive empire).

Introduce an additional variable on cities, can be called "population" or "sustenance".
This is a base number of Resources that is subtracted from the city's total resources,
to slow down the production rate.

As the game advances, its sustenance cost can rise, preventing it from becoming
*too* productive in the late-game.

Additional things that this number could be used for:

### Building progression

Require a minimum population number for certain structures, preventing them from
being constructed too early in the game.

### Starvation

If the city fails to cover its sustenance cost for an extended period of time
("starvation"), it could be destroyed / removed from the game.

For example:
if the sustenance cost is currently X, and the city has less than X total resources,
a timer (say, one minute) starts. If the timer elapses, the city is turned to ruin.
If the player manages to recover some land within the timer, as soon as the city
has at least X resources, it leaves "starving" state and the timer resets.

This would make cities non-permanent and allow the total number of cities on the
map to be reduced in the late-game.

When a player is eating away at the opponent's territory, the oppenent's city's
resources fall drastically. The risk of starvation puts extra pressure on the
defender to defend the territory in the region of the city.

This also puts pressure on the attacker: if they want the city, they must commit
to reaching and capturing it as quickly as possible, before it has starved.
Alternatively, if they want the city gone, they could deliberately starve the
city by playing slowly.
