# Data Collections

There are several different collections of data. Currently these are stored in MongoDB. In this document, I will describe all the different data collections; I will give an overview of the data's purpose, I will define its shape, and I will describe the indices that should be placed on their collections in order to speed up queries.

## Table of Contents

**Confirmed Collections**:

- [Players](#players)
- [Confirmation Tokens](#confirmation-tokens)
- [Refresh Tokens](#refresh-tokens)
- [Friend Requests](#friend-requests)
- [Friendships](#friendships)
- [Counters](#counters)
- [Games](#games)

## Players

The `players` collection holds player accounts. The information stored within is all the info which directly has to do with a _single_ player account. Information linking two accounts together, like friendships, are not stored here.

### Models

| Field                   | Data Type            | Notes                                                    |
| ----------------------- | -------------------- | -------------------------------------------------------- |
| `player_id`             | `String`             | Random UUID v4 converted to string; unique               |
| `username`              | `String`             | Case-insensitively unique                                |
| `email`                 | `String`             | Case-insensitively unique                                |
| `password`              | `String`             | Hashed using Argon2                                      |
| `created`               | `bson::Date`         | Unconfirmed accounts will be deleted after two days      |
| `confirmed`             | `bool`               | Whether an email address has _ever_ been confirmed       |
| `proposed_email`        | `Option<String>`     | A proposed email address which has not yet been verified |
| `last_passwords`        | `[String;4]`         | The last 4 hashed passwords used by this account         |
| `gender`                | `String`             | `"male"` \|\| `"female"` \|\| `"other"`                  |
| `preferred_language`    | `String`             | `"en"` \|\| `"es"`                                       |
| `pronoun` **\***        | `String`             | `"masculine"` \|\| `"feminine"` \|\| `"neutral"`         |
| `stats`                 | `PlayerStats`        | See `PlayerStats` model below.                           |
| `last_login`            | `bson::Date`         | Last **successful** login                                |
| `failed_logins`         | `u8`                 | Number of consecutive failed login attempts              |
| `locked_until` **\*\*** | `Option<bson::Date>` | When a login can be attempted again                      |

> **\*** The `pronoun` field is important for inclusivity when translating the application into Spanish. When a player chooses `gender == "other" && preferred_language == "es"`, we ask them how they would like to be treated, as the `-e` ending for gender-neutral terms is not universally accepted.
>
> Below is an example of text specifically catered to spanish-speaking players of different pronouns:
>
> | Pronoun     | "Welcome, esteemed player!"       |
> | ----------- | --------------------------------- |
> | `masculine` | "¡Bienvenido, estimado jugador!"  |
> | `feminine`  | "¡Bienvenida, estimada jugadora!" |
> | `neutral`   | "¡Bienvenide, estimade jugador!"  |

---

> **\*\*** After five failed logins, an account is **locked** for 15 minutes. Each consecutive failed login will result in the account being locked for the previous lockout time plus 15 more minutes; i.e. 6 failed attempts = 30 minute lockout, 7 failed attempts = 45 minute lockout...

#### PlayerStats

| Field      | Data Type | Notes                                                                                                                                   |
| ---------- | --------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `wins`     | `u64`     | How many games have been won. Includes wins by all other players dropping out, as well as shared wins by the **last chance** house rule |
| `losses`   | `u64`     | How many games have been finished, but lost                                                                                             |
| `dropouts` | `u64`     | How many games the player has dropped out of                                                                                            |

### Indices

| Field(s)    | Indices                   | Condition            |
| ----------- | ------------------------- | -------------------- |
| `player_id` | Unique                    |                      |
| `username`  | Case-insensitively unique |                      |
| `email`     | Case-insensitively unique |                      |
| `created`   | TTL (2 days)              | `confirmed == false` |

## Confirmation Tokens

The `confirmation-tokens` collection holds tokens that can be used to confirm a player's email address. These tokens can be used both for **initial account confirmation** as well as **verifying a new email address** for an account which has already been established.

There may only ever be **one token per `player_id`** in this collection.

### Model

| Field       | Data Type    | Notes                                      |
| ----------- | ------------ | ------------------------------------------ |
| `token_id`  | `String`     | Random UUID v4 converted to string; unique |
| `player_id` | `String`     | The player this token represents; unique   |
| `created`   | `bson::Date` | These disappear after two days             |

### Indices

| Field(s)   | Index        |
| ---------- | ------------ |
| `token_id` | Unique       |
| `created`  | TTL (2 days) |

## Refresh Tokens

The `refresh-tokens` collection holds tokens that can be used to identify a player once their **access token** has expired, saving players from having to log-in again every 15 minutes.

A single player can have up to **3** refresh tokens associated with their account; this means that a player can stay signed in on **up to 3 devices**.

Refresh tokens disappear every 30 days, however, each time a token is used, it is replaced by a fresh token, with a new `token_id` and a new `secret`, so a player will only need to sign in again on their device if they have not been active on their account for 30 days straight; a player who uses the app every day on the same device could use the application without logging in **indefinitely**.

### Model

| Field       | Data Type    | Notes                                                          |
| ----------- | ------------ | -------------------------------------------------------------- |
| `token_id`  | `String`     | Random UUID v4 converted into string; unique                   |
| `player_id` | `String`     | The player this token represents                               |
| `secret`    | `String`     | Hashed using Argon2                                            |
| `created`   | `bson::Date` | These disappear after 30 days                                  |
| `revoked`   | `bool`       | Attempted use of a revoked token indicates suspicious activity |

### Indices

| Field       | Index    |
| ----------- | -------- |
| `token_id`  | Unique   |
| `player_id` | Standard |

## Friend Requests

The `friend-requests` collection keeps track of pending friend requests between two players. When a friend request is accepted, the document is deleted and transferred to the `friendships` collection. When a friend request is rejected, it is simply deleted.

### Model

| Field         | Data Type    | Notes                                        |
| ------------- | ------------ | -------------------------------------------- |
| `request_id`  | `String`     | Random UUID v4 converted into string; unique |
| `sender_id`   | `String`     | The player who sent the request              |
| `receiver_id` | `String`     | The player who receives the request          |
| `created`     | `bson::Date` |                                              |

### Indexes

| Field         | Index    |
| ------------- | -------- |
| `request_id`  | Unique   |
| `sender_id`   | Standard |
| `receiver_id` | Standard |

## Friendships

The `friendships` collection keeps track of active friendships between two players. When a friendship is terminated, the document is simply removed from the database.

### Model

| Field           | Data Type    | Notes                                                      |
| --------------- | ------------ | ---------------------------------------------------------- |
| `friendship_id` | `String`     | Random UUID v4 converted into string; unique               |
| `friends`       | `[String;2]` | The first string is the sender; the second is the receiver |
| `since`         | `bson::Date` |                                                            |

### Indices

| Field(s)             | Index    |
| -------------------- | -------- |
| `friendship_id`      | Unique   |
| `friends` -> `since` | Compound |

## Counters

The `counters` collection is unique from our other data collections, as there will not be a potentially endless number of them. Counters is used for metadata related to the application, so that I (the developer) can track how my application is being used. A counter has simply a unique ID representing the statistic that is being counted, and a value.

### Model

| Field   | Data Type | Notes                                       |
| ------- | --------- | ------------------------------------------- |
| `id`    | `String`  | Indicates what this counter tracks          |
| `value` | `u64`     | How many times an action has been performed |

### Indices

| Field | Index  |
| ----- | ------ |
| `id`  | Unique |

### Known Counters

The following list is not complete, but indicates some of the counters that will be used in the application:

| ID                   | Purpose                                                                                                                                                                     |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `pings`              | Counts how many times the backend has started up and connected to the database successfully                                                                                 |
| `accounts_created`   | Counts how many accounts have been initially registered                                                                                                                     |
| `accounts_confirmed` | Counts how many times an account has been confirmed                                                                                                                         |
| `accounts_deleted`   | Counts how many times an account has been deleted _after_ having been initially confirmed                                                                                   |
| `games_started`      | Counts how many times a new game of D-Bo has been started                                                                                                                   |
| `games_finished`     | Counts how many games of D-Bo have been _finished_ - meaning they ended with a single winner or a draw                                                                      |
| `games_terminated`   | Counts how many games of D-Bo have been _prematurely terminated_ - indicating a player has forfeited the game, or didn't take their turn within a reasonable amount of time |
| `friendships`        | Counts how many friend requests have been accepted                                                                                                                          |

## Games

The `games` collection stores game states. This collection is far more complex than all other collections, and the `Game` model includes several sub-models which will be described below. These models are stored within the same collection, as nested objects of a `Game`.

### Models

#### Game

| Field     | Data Type          | Notes                                                                                                   |
| --------- | ------------------ | ------------------------------------------------------------------------------------------------------- |
| `game_id` | `String`           | Random UUID v4 converted into string; unique                                                            |
| `players` | `Vec<PlayerState>` | All players who participate in this game. The order indicates turn-order. See `PlayerState` model below |
| `turn`    | `usize`            | Indicates the array position of the player whose turn it is                                             |
| `rules`   | `Ruleset`          | See `Ruleset` model below                                                                               |
| `deck`    | `Vec<Card>`        | Indicates the cards stored in the deck from which all players draw their cards. See `Card` model below  |

#### Card

This model is very simple, but deserves some explanation. Its **data type** when stored in the database is simply a `u8`; in fact, it will always be a number between `0` and `12` (inclusive).

The value `0` represents a "D-Bo" card - which is essentially a wild card that can be used to represent any value. All other values represent a card of their own value.

#### PlayerState

The `PlayerState` model represents the cards belonging to a single player.

| Field         | Data Type        | Notes                                                             |
| ------------- | ---------------- | ----------------------------------------------------------------- |
| `active`      | `bool`           | Indicates whether a player is currently present in the game lobby |
| `hand`        | `Vec<Card>`      | The cards in the player's hand                                    |
| `discard`     | `[Vec<Card>;4]`  | The player's four discard piles.                                  |
| `stock`       | `Vec<Card>`      | The player's stock pile.                                          |
| `final_state` | `Option<String>` | `"winner"` \|\| `"loser"` \|\| `"quitter"`                        |

#### Ruleset

The `Ruleset` model represents the custom rules for a game. This is set at the beginning of the game and cannot be changed afterward.

For a detailed description of the rules of **D-Bo**, including house rules, check out the [Rules Document](RULES.md).

| Field          | Data Type | Default                                    |
| -------------- | --------- | ------------------------------------------ |
| `deck_count`   | `u8`      | `1`                                        |
| `stock_size`   | `u8`      | `30` for 2-4 players; `20` for 5-6 players |
| `random_start` | `bool`    | `false`                                    |
| `dump`         | `bool`    | `false`                                    |
| `last_chance`  | `bool`    | `false`                                    |

### Indices

| Field     | Index    |
| --------- | -------- |
| `game_id` | Unique   |
| `players` | Standard |
