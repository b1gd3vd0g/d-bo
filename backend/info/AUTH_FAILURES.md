# Authentication Failures

Obviously, **MOST** requests require some sort of authentication. First, I will list the types of authentication that are required, describe why and what types of requests will require that type of authentication. Then, I will describe all of the reasons that authentication can fail. This will make a conclusive list of all types of authentication failure.

> **NOTE**: These authentication failures are only valid for our REST API endpoints; there will be some sort of authentication needed for the WebSocket interactions as well, but as I'm not sure how that will work out specifically at this point, we're not gonna get into it yet.

## Table of Contents

- [Authentication Types](#authentication-types)
  - [Login Credentials](#login-credentials)
  - [Authentication Token](#authentication-token)
    - [Authentication Token _and_ Password](#authentication-token-and-current-password)
  - [Cookie Authentication](#cookie-authentication)
- [Response Codes](#response-codes)

## Authentication Types

There are **four** different types of authentication that may be needed by different types of requests - [login credentials](#login-credentials), [authentication token](#authentication-token), [authentication token _and_ password](#authentication-token-and-current-password), and [cookie authentication](#cookie-authentication). These types of authentication can all fail in _distinct_ ways; often, multiple types of failures are possible for the same authentication type.

Following are descriptions of the four authentication tokens, and the reasons that they may fail.

### Login Credentials

**Login Credentials** require two things in order to authenticate a player: an **identifier** and a **password**.

The **identifier** is _either_ the player's **username** _or_ their **email address**. The **identifier** is **case-insensitive**, _but_ must be **complete**.

The **password**, conversely, is **case-sensitive** as well as **complete**. The player will provide their **raw-text password**, which will be compared to the stored **password hash** in the database. The player must use their **most recent password** in order to identify themselves.

> **Login Credentials** is only required in the **login** endpoint; _all_ other endpoints require at least a slightly different authentication type.

**Login Credentials** can fail for technically _two_ different reasons, but the distinction should **never** be shared with the client, in order to better protect our data.

- The player account could not be found with the provided **identifier**
- The **password** does not match the player's currently stored **hash**

> **Note**: Either one of these failures should simply return **BAD LOGIN CREDENTIALS** to the client.

### Authentication Token

Both the **login** and **refresh** endpoints will provide the client with an **authentication token** - a **JSON Web Token** which can identify a player for the next **15 minutes**.

This **authentication token** is _probably_ the most common form of authentication for our application. It is used to identify the player who is making the request.

The player's **authentication token** should be provided following the guidelines of [Bearer Authentication](https://swagger.io/docs/specification/v3_0/authentication/bearer-authentication/)

**Authentication Tokens** have _several_ different reasons for failure:

- The authentication token was not provided, following the **Bearer Authentication** guidelines.
  - The `Authorization` header is missing
  - The `"Bearer "` prefix is missing from the `Authorization` header
  - There is no value _following_ the `"Bearer "` prefix.
- The authentication token could not be parsed by the system.
  - _This most likely means that the token is forged!_
- The authentication token is **expired** - it was created _more than_ 15 minutes ago
  - An **expired token** can _usually_ be refreshed by visiting the **refresh** endpoint.
- The authentication token is **premature** - it was created _before_ the player's **sessions were invalidated**.
  - A player's sessions become "invalidated" following a change to their **login credentials**.
  - This response means that the player will have to **login** again, using their **login credentials**.
- The authentication token does **not represent an existing player account**.
  - This will usually happen if a player **deletes their account**, and later tries to use a valid authentication token to make a request.

> **NOTE**: All of these possibilities should be considered and handled properly - however, when **benign** users are using the application as intended, by using a **frontend** that is created correctly, the only errors that **should** happen are **expired token** and **premature token**. All other errors indicate either **(a)** bad frontend design, or **(b)** malicious intent

#### Authentication Token _and_ Current Password

There are a handful of requests that could be **too disruptive** to a player's account to require _just_ a valid authentication token. These requests could end up **deleting** the player's account entirely, or otherwise **locking the player out** of their own account, and allowing a malicious user to take control of the account and impersonate another player.

These requests include **account deletion**, and **changes to login credentials**.

These requests can fail if the **authentication token** is invalid for _any_ of the reasons listed above, **or** if the provided password is incorrect.

> **Note**: Unlike **login credentials**, the code returned to the client should specifically indicate **WRONG PASSWORD**, if the token was valid and identified a player, but the password did not match.

### Cookie Authentication

This method of authentication is used only for the **refresh** endpoint.

Both the **login** endpoint _and_ the **refresh** endpoint itself will set an **HTTP-Only Cookie** for requests sent to the **refresh** endpoint. This cookie will contain both an **identifier** (a unique UUID V4) and a **secret**.

These endpoints also create **persistent refresh tokens** in the database, with the unique identifier, the secret (hashed in the same way as player passwords), the date it was created, and the player it represents. These refresh tokens are valid for up to **30 days** after their creation.

There are several reasons why this method of authentication could fail:

- The cookie is not set
  - The required key is `refresh_token`
- The cookie value cannot be parsed
  - The cookie value should be formatted in the following way: `"{id}:{secret}"`. If the string cannot be split into **two** pieces, by splitting it at the colon, then the value is considered unparseable.
- The **identifier** does not represent an existing refresh token in the database.
  - This could mean _either_ that the token has expired and has been wiped out by an automatic TTL index, **or** that the token never existed in the first place.
- The refresh token was found, but is **expired**.
  - As the TTL index is the same as the expiration time, it is unlikely that this will ever happen, but it is **possible** and should be accounted for.
- The refresh token was found, but its **secret** does not match the hash stored in the database.
- The refresh token was found, its secret matches, but it does not represent an **existing player account**.
  - This will happen if a player **deletes their account** and later visits the **refresh endpoint**.

## Response Codes

Whenever authentication fails for any of the reasons described above, a `401 Unauthorized` HTTP response will be returned to the client.

In order to distinguish between the different **reasons**, the HTTP response body will contain a **code** indicating what happened.

The following table indicates the **code** which will be returned within the response body, and what they mean.

| Code                                         | Meaning                            |
| -------------------------------------------- | ---------------------------------- |
| [`BLC`](#blc-bad-login-credentials)          | **Bad Login Credentials**          |
| [`MAT`](#mat-missing-authentication-token)   | **Missing Authentication Token**   |
| [`BAT`](#bat-bad-authentication-token)       | **Bad Authentication Token**       |
| [`EAT`](#eat-expired-authentication-token)   | **Expired Authentication Token**   |
| [`PAT`](#pat-premature-authentication-token) | **Premature Authentication Token** |
| [`BPW`](#bpw-bad-password)                   | **Bad Password**                   |
| [`CNS`](#cns-cookie-not-set)                 | **Cookie Not Set**                 |
| [`NPC`](#npc-non-parseable-cookie)           | **Non-Parseable Cookie**           |
| [`BCC`](#bcc-bad-cookie-credentials)         | **Bad Cookie Credentials**         |
| [`ERT`](#ert-expired-refresh-token)          | **Expired Refresh Token**          |
| [`PNF`](#pnf-player-not-found)               | **Player Not Found**               |

### `BLC` Bad Login Credentials

This is the only returned from the **login** endpoint, and indicates that _either_ the **identifier** (username or email) _or_ the **password** does not match the records in our database.

### `MAT` Missing Authentication Token

This can be returned from **any** endpoint requiring an authentication JWT. It means that **no value** was provided for the authentication token; either:

- The `Authorization` header is missing
- The `"Bearer "` prefix is missing from the `Authorization` header
- There is no value _following_ the `"Bearer "` prefix

### `BAT` Bad Authentication Token

This can be returned from **any** endpoint requiring an authentication JWT. It means that the value provided as the authentication token **cannot be parsed**.

### `EAT` Expired Authentication Token

This can be returned from **any** endpoint requiring an authentication JWT. It means that the authentication token is **expired** after 15 minutes.

### `PAT` Premature Authentication Token

This can be returned from **any** endpoint requiring an authentication JWT. It means that the authentication token was created **before** a player's sessions were invalidated.

### `BPW` Bad Password

This can be returned from endpoints requiring **both** an authentication JWT _and_ a password. This indicates that the authentication token passed all checks, but the **password** provided in the request body does not correspond with the player's current password.

### `CNS` Cookie Not Set

This can be returned only from the **refresh** endpoint. It indicates that the `refresh_token` cookie was not set in the request.

### `NPC` Non-Parseable Cookie

This can be returned only from the **refresh** endpoint. It indicates that value stored for the `refresh_token` cookie could not be parsed into an **id** and a **secret**.

### `BCC` Bad Cookie Credentials

This can be returned only from the **refresh** endpoint. It indicates that the **id** and/or **secret** provided in the `refresh_token` cookie do _not_ correspond with an existing refresh token in the database.

### `ERT` Expired Refresh Token

This can be returned only from the **refresh** endpoint. It indicates that the refresh token was found in the database, but has since **expired** after 30 days.

> **Note**: This code is unlikely, as the **expiration** for refresh tokens is set for the same amount of time as the **TTL index**, which will wipe it from the database automatically. So _usually_, even if the token _did_ once exist but has expired, the `BCC` code will be returned instead.

### `PNF` Player Not Found

This can be returned from **any** endpoint requiring _either_ an authentication JWT _or_ cookie credentials for a refresh token.

It reflects that the token (either authentication JWT _or_ refresh token) was valid, _but_ the `player_id` stored within that token does **not** correspond with an existing player account.
