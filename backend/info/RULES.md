# D-Bo Rules

**D-Bo** is based on a familiar card game, and by default, it will follow the rules exactly. The application also supports a number of **house rules**, to modify the default ruleset in order to let them play the way that they want.

This document will first outline the default rules of the game, and in the following section, will describe the available house rules and explain how they modify gameplay.

## Default Rules

**Players**: 2 - 6\
**Deck**: 162 cards; 12 cards of each number 1-12, and 18 "D-Bo" cards

### Key terminology

The **draw pile** is a facedown pile of cards from which all players may draw new cards.

A **stock pile** is a pile of cards dealt to each player at the start of the game. The cards are placed face-down, and only the top card may be flipped. _Using all cards in your stock pile is how you win!_

A **discard pile** is a stack of cards, placed face-up, which belongs to a single player. Each player is allowed _four_ discard piles. The top card of any discard pile may be used during a player's turn.

A **building pile** is a stack of cards, placed face-up, which is shared between all players. There are _four_ building piles on which players can lay their cards. A building pile must start with a 1, and cards can be placed on top of the pile in _incremental increasing order_. A player may only place cards on a building pile; when the pile is _complete_, its cards are removed and placed into the _scrap pile_.

A **scrap pile** includes cards which have already been played or otherwise forfeited; when the _draw pile_ becomes empty, the scrap pile is shuffled, then replaces the _draw pile_.

### Gameplay

When a game is generated, the system determine a random turn order for all players. Each player will be dealt their **stock pile**. The amount of cards in each player's stock pile is reflected in the table below:

| Players | Stock pile size |
| ------- | --------------- |
| 2 - 4   | 30              |
| 5 - 6   | 20              |

The top card of each player's **stock pile** will be flipped over.

At the beginning of each player's turn, they will draw cards from the **draw pile** until they have five cards in their hand.

Players may use cards from their **hand**, from their **discard piles**, or from their **stock piles**, and place them on any **building pile**. A player can place as many cards as possible onto the building pile. If a player happens to empty their **hand** while their turn is still going, they may draw 5 more cards from the **draw pile** and continue play.

To end their turn, a player must place a card from their hand into one of their **discard piles**. Any card may be placed into any discard pile; however, only the card on the _top_ of any discard pile may be used in gameplay.

The player who empties their **stock pile** first is declared the winner, and gameplay is finished.

### Table setup

A drawing of a game of **D-Bo** between two players is provided below.

```
                           Player 1
+--------------------------------------------------------------+
| (Player 1's Zone)                                            |
|                                                              |
|        Stock             Discard                             |
|        Pile               Piles                              |
|        +--+       +--+  +--+  +--+  +--+                     |
|        |::|       |::|  |::|  |::|  |::|                     |
|        +--+       +--+  +--+  +--+  +--+                     |
|                                                              |
| - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -|
| (Communal Zone)                                              |
|                                                              |
|        Draw              Building               Scrap        |
|        Pile               Piles                 Pile         |
|        +--+        +--+  +--+  +--+  +--+        +--+        |
|        |  |        |##|  |##|  |##|  |##|        |##|        |
|        +--+        +--+  +--+  +--+  +--+        +--+        |
|                                                              |
| - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -|
| (Player 2's Zone)                                            |
|                                                              |
|                   +--+  +--+  +--+  +--+         +--+        |
|                   |##|  |##|  |##|  |##|         |##|        |
|                   +--+  +--+  +--+  +--+         +--+        |
|                          Discard                 Stock       |
|                           Piles                  Pile        |
+--------------------------------------------------------------+
                           Player 2
```

## House rules

During game initiation, any combination of the following house rules can be combined to create a custom ruleset, so that players can play the way they prefer.

### Multiple decks

**Multiple Decks** is a house rule which allows players to use multiple decks of cards in their game. This house rule is accompanied by a positive integer value between **1 and 5** indicating the number of decks; default is 1.

### Custom stock

**Custom stock** is a house rule which lets a player dictate the starting size of their stock pile. This house rule is accompanied by a positive integer value between **10 and 100** indicating the number of cards initially in each stock pile.

> **Custom stock** and **Multiple decks** are restricted in the following way: the total number of cards in all players' stock piles at the start of the game may _not_ exceed **75%** of the total deck size.
>
> In a game with six players and two decks, the maximum **custom stock** is **40** - there are `324` total cards, 75% of which is `243`; and as `41 * 6 > 243`, this would break the rule.
>
> In a game with four players and a custom stock of 75, the minimum number of decks is **3** - there are 300 cards initially in stock; if there were 2 decks, the total amount of cards would be `324`; and as `300 / 324 > 0.75`, this would break the rule.

### Random start

**Random start** is a house rule in which, at the start of a game, after dealing the **stock piles**, instead of keeping the building piles empty, a card is placed in each space. If a **D-Bo** card happens to be placed in a build pile in this way, it is treated as a 12 - it is placed in the **scrap pile**, and that building pile is left empty, so that only a 1 can be played.

### Dump

**Dump** is a house rule that can be toggled either _on_ or _off_.

With **dumping** enabled, a player may choose after drawing their cards at the beginning of their turn to **dump** their entire hand into the scrap pile instead of taking their turn. Once a player has played a card (whether from their stock pile, discard piles, or hand), they may no longer choose to **dump**.

### Last chance

**Last chance** is a house rule which allows a game to have multiple winners. After a player wins by playing the ultimate card in their stock pile, they will lay down the cards in their hand **face-up**. Their discard piles, as well as the cards from their hands, become open to play by any subsequent player. Each player will get their **last chance** - they can use their own cards, as well as the **hands** and **discard piles** of any player who has finished their final turn. If they manage to empty their stock pile, they will be considered a winner.

During their **last chance**, whether the player wins or not, their hand and discard piles become available to all subsequent players.
