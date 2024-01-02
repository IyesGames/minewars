# Economy

The **Economy** is a core part of the game. Each city generates income for
the player, which can be spent to perform gameplay actions.

### Money

Each city has a **Bank**. The Bank is the total **money** of the city.

The player can spend money to:
 - Build structures
 - Deploy items
 - Perform actions

The actions are funded by the city of the region of the tile where the action
takes place.

In foreign regions (where the player does not own the local city), actions
are funded by the nearest city by distance.

If a city has *active road connections* to other cities, then they will fund the
actions collectively. The cost of the actions is split between all connected
neighboring cities, proportionally to the total amount of money each city has in
its bank. The local city's contribution is weighted double.

If a city is captured by another player, **50%** of the money will stay in the
city and be usable by the new owner, and the remainder will be lost.

### Income

Each city constantly generates/accumulates money. The rate of income is determined
by the total **resources** owned by the city; that is, the sum of the resources
of each owned tile within the city's region. It can be reduced due to ongoing
construction. It can be affected by **Policy**.

Neutral cities (not owned by any player) do not generate any income.

### Resources

Each tile provides a certain number of resource points, counted towards the
city of the region where the tile is located, if the player owns both the
city and the tile.

Resources given by each tile owned:
 - **Regular Land**: **1** Res/sec
 - **Fertile Land**: **2** Res/sec
 - **Destroyed Land** and **Foundation**: **0** Res/sec
 - **Forest**: **5** Res/sec
 - **Mountain**: **7** Res/sec

Further, each city has **25** Base Res: the minimum amount of resources inherent
to the city, regardless of any owned tiles in the region.

### Policy

Players may tune a few different parameters that affect the resources of a city
and their usage:

 - Construction Rate: What % of resources goes towards any in-progress structures,
   instead of generating Income.
 - Export Rate: How much do we help (% of max) connected cities with their spending.
 - Import Rate: How much help (% of max) do we accept from connected cities.

### Harvesting Tiles

Players may **harvest* tiles (one tile at a time) to instantly claim a large sum
of resources. They are awarded to the city of the region the tile belongs to.

This is a one-time action. After harvesting, the tile is converted to
**Foundation** and produces no more resources for the remainder of the game.

The player must own both the tile and the city whose region it belongs to.

This mechanic exists to give players more options for survival in tense
situations. They can accelerate their current production one-time, at the cost
of foregoing all the potential resources they could have accumulated from the
tile over time and also potentially worsening the natural protection they get
from the map's geography.

Harvest bounties (resource given when harvesting a tile):
 - **Destroyed Land** and **Foundation**: not harvestable (**0** Res)
 - **Regular Land**: **100** Res
 - **Fertile Land**: **250** Res
 - **Forest**: **420** Res
 - **Mountain**: **600** Res

Harvest delay:
 - **Destroyed Land** and **Foundation**: not harvestable (**0** Res)
 - **Regular Land**: **5** sec
 - **Fertile Land**: **5** sec
 - **Forest**: **15** sec
 - **Mountain**: **30** sec

### Constructing Structures

The player may construct infrastructure on land that they own, if they own the
city in the region. When the player wants to build something, the city's
resources will go towards the structure and the city will not generate income
until construction is complete.

If construction is canceled before it is completed, **50%** of the spent
resources are instantly reclaimed and the remaining resources are lost.

### Bulldozing Structures

Any structures on land owned by the player can be bulldozed. This returns
**25%** of the resource cost of the structure back to the player.

The reward is counted towards the city of the tile's region, if owned by the
player, or the nearest city by distance otherwise.

### Capturing Foreign Items

If a player captures a tile that contains an item, the item is instantly sold,
giving the player:
 - Mine: 75% of the usual cost
 - Decoy: 50% of the usual cost
 - Smoke Trap: 0% of the usual cost

The reward is counted towards the city of the tile's region, if owned by the
player, or the nearest city by distance otherwise.

### Costs

The cost of everything is determined using a multiplier applied to a **Base
Unit**; this helps negate the unpredictability of the random procedural world
generation. This is calculated once at the start of the game, based on the
generated map, and remains fixed throughout the game.

The **Base Unit** is roughly representative of how powerful cities could
get in a given game session, and is calculated as follows:
 - The arithmetic mean of the available resources in each region

This should help give relatively well-balanced costs with varying map configurations.

The final amounts are rounded up to the nearest **100**.

Cost sheet (TODO playtest and balance):

Items:
 - Decoy: 2.0
 - Mine: 3.0
 - Mine (upgrade from decoy): 3.0
 - Flashbang: 8.0

Structures:
 - Road: 1.0
 - Barricade: 20.0
 - WatchTower: 30.0
 - Bridge: 5.0

Actions:
 - Strike: 1.0 + 2.0 * NTiles
 - Reveal: 5.0
 - Smoke: 0.5

Starting money:
 - If a player captures a neutral city: 10.0
 - The initial (spawn) city of each player starts with: 16.0
