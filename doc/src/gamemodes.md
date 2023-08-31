# Game Modes

The game client can offer a variety of different modes to offer gameplay
experiences for different users' tastes.

These are just interesting ideas to try out. I honestly don't know what game
modes we will end up with after playtesting. :)

## MineWars Game Modes

These are variants of the MineWars real-time competitive multiplayer game,
as described in [gameplay](./gameplay.md).

### Elimination (classic)

 - Players: **4**.
 - Cities: **8**.
 - Map size: **64**.

Balanced experience. The goal is to balance strategy and action.

Plenty of breathing room for players to grow their empires and establish their
strategies. The map is big, plenty of space and resources. There are more
cities than players, meaning neutral cities are available for taking.

Leads to distinct "early game", "mid game", and "late game" gameplay.

Early game is the initial exploration, to establish a presence on the map,
before the player has encountered another player. This feels a bit like singleplayer
Minesweeper -- you try to expand into neutral territory while uncovering the random
pre-seeded mines.

Mid game starts from your first encounter with another player. You now need to worry
about your border / battlefront, offense and defense. The midgame is a battle for
survival, your goal is to eliminate weaker players and prepare for the final duel.

Late game is when there are only two empires left on the map and the other
players have been eliminated. The two remaining players are likely the
strongest, with big empires, and need to battle to determine the final winner.

### Mayhem

 - Players: **6**
 - Cities: **6**
 - Map size: **48**
 - Reduced production costs
 - No Ban stage

Game mode designed for frantic action and less room for strategy. The
progression of the game effectively moves faster. The idea is to lead to a
short early game and frantic mid game.

There are more players than in a classic game, and there are exactly as many
cities as there are players, so players are forced into PvP confrontations
almost immediately. The map is smaller and more claustrophobic.

### Duel

 - Players: **2**
 - Cities: **5**
 - Map size: **32**

With only two players, effectively skips from early game to late game, skipping
the mid game. The map is smaller and tighter, to keep the game time shorter and
prevent it from becoming boring from the early game taking too long. There are plenty
of cities for the players to battle over.

### Duos

 - Two players control one empire
 - Empires: **3** (6 players)
 - Cities: **6**
 - Map size: **48**

Adds some teamwork and cooperation to the mix. Reliant on having communication
between the teammates.

## Turn-Based MineWars

GAME DESIGN WIP!! This will need a lot of revision!

Alternative turn-based Duel, for those who prefer a slower-paced methodical
game, instead of the frantic action of regular MineWars.

 - Players: **2**
 - Cities: **5**
 - Map size: **24**

Map must be tiny, due to the slow-paced action of turn-based gameplay.

There is no fog of war, all tiles have a Visibility state of at least Limited.

Turn time limit: **10 seconds**.

Within each turn, players have a quota of actions they may perform:

 - Capture up to **3** land tiles.
 - Deploy up to **3** items.
 - Queue up production items for their cities.

... blah blah blah ... todo todo

## Singleplayer MineWars-like

Game modes that can be played offline, sharing some mechanics with the
regular multiplayer MineWars.

### MineWars vs. AI/bots

(considered out of scope for now)

### MineWars Playground

For experimentation: allow one client to control all views/empires.

### MineWars-like Minesweeper

Like Minesweeper, the goal is to capture all safe tiles on the map.

## Classic Minesweeper

Game modes without any of the fancy MineWars mechanics. Just simply
click on tiles and avoid mines. The goal is to claim all safe tiles
in the quickest time possible. May be made more forgiving by giving
players multiple lives.

We can offer some diversity in terms of map styles. Choose between:
square grid (like good ol' Minesweeper), hex grid, MineWars map.

### Singleplayer

Just play by yourself.

### Co-op

Multiplayer session, where all players share the same color/territory and lives.

### PvP

Multiplayer session, but every client gets their own color/territory, lives, and
stats for how much of the map they have explored. At the end, players are ranked
on who has explored the most.


