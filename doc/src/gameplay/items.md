# Items

Items are defensive things that you can **deploy** onto **land** tiles that you own.
If another player attempts to capture the land, they will suffer the effect of the item.

Once deployed, you cannot remove your own items from the tiles they are on.

Every player sees **Digits** on tiles they own, indicating the total number of items
on adjacent tiles they do not own.

## Mines

**Mines** will **explode** when stepped on, resulting in the offending player
receiving a **Stun**. This adds to the "Death" count of the offending player and
to your "Kill" count.

All players in the match will be notified of this event. The explosion effect
and location will be seen by all players. The tile where the explosion occurs
will be upgraded to "Limited Visibility" status for all players for a duration
of **2.0 sec**. Note: this means players can see ownership, etc.

A skull decal will be left behind to commemorate the occasion.

The **Stun** is a timeout during which a player cannot perform any game actions.
All of their cities pause production for the duration of the stun.

The stun duration is **5** seconds.

The tile will be converted to **Destroyed Land**, thereby providing no more
resources for the remainder of the game.

## Decoys

**Decoys** are harmless and will **break** when stepped on. The other player
will successfully capture the tile.

The purpose of decoys is to confuse the other player and make the Minesweeper
digits puzzle more difficult for them to solve.

The presence of a decoy will add an asterisk (*) to the digits other players see
on adjacent tiles. This provides a hint to them, without directly revealing the
actual location of the decoy.

## Flashbang Traps

The offending player will get a "full white" screen flash, which fades out over
**0.36** seconds. For accessibility reasons, the color of the flash should be
customizable, but it cannot be disabled completely.

All tiles adjacent to the flashbang's tile, that are owned by the offending
player, will have their digits obscured (similar to smoke).

Similar to a Decoy, a Flashbang will add an asterisk (*) to adjacent digits.
This will trick players to fall into the trap.
