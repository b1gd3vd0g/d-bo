# D-Bo Backend Roadmap

The following is a roadmap to completion of the D-Bo Backend crate. This document is going to be frequently updated - as I complete tasks, I will mark them off, and as new tasks become apparent to me, I will add them to the list wherever they seem most appropriate.

## Completed

- `[x]` Initialize project

- `[x]` Create rules for input validation (username, password, email) and create fucntions to ensure compliance
- `[x]` Create a connection to the MongoDB database
- `[x]` "Ping" the database at startup to ensure a connection can be made
- `[x]` Create the axum router and make it listen on port 60600
- `[x]` Create a "Player" struct to act as a model for the mongodb database
- `[x]` Test the validation functions
- `[x]` Define a function to validate all inputs at once, returning an `InputValidation` struct describing all problems with the input, which can be returned with a 400 response to the client
- `[x]` Create a template for the confirmation emails
- `[x]` Create a `DBoError` enum to help ease error-handling within the app
- `[x]` Create `send_confirmation_email(..)` function. This is currently incomplete, as all links go to my portfolio website instead of to an actual confirmation/rejection page.
- `[x]` Implement `Player::register(..)` function (incomplete; does not yet add the player to the database)
- `[x]` Create hashing functions for secure password verification/storage
- `[x]` Make `Player::register(..)` add the document to the database
- `[x]` Create `MessageResponse` and `ExistingFieldViolationResponse` structs to return as HTTP response bodies to the players
- `[x]` Create `SafePlayerInfo` struct to only expose public information for players to the client
- `[x]` Create the player registration handler function at `POST /player`
- `[x]` Create CORS configuration (must be made more restrictive in the future)
- `[x]` Make `ConfirmationToken` struct to act as a model for email confirmation tokens in the database
- `[x]` Use case-insenstive collations and database indexes to make queries faster and enforce case-insensitive uniqueness on email and username fields at the database level
- `[x]` Create a 2-day TTL index on confirmation tokens
- `[x]` Add `ConfirmationToken::expired(..) function to enforce a 15 minute window to confirm a player's email address
- `[x]` Create Player find functions to search the database for a single player
- `[x]` Create `ConfirmationToken::insert(..)` function to actually store the tokens in the database
- `[x]` Update `Player::register(..)` to add the confirmation token into the database
- `[x]` Derive `Debug` for `InputValidation` and `DBoError` structs for better error logging
- `[x]` Fix the confirmation email template and `send_confirmation_email(..)` function so that it links to the appropriate page
- `[x]` Implement `Player::delete(..)` function
- `[x]` Implement `Player::confirm(..)` function to mark their email address to confirmed.
- `[x]` Use my OWN bson dependency with feature "chrono-0_4" to enable conversions between `chrono::DateTime<Utc>` and `bson::DateTime`, actually enabling TTL index capabilities
- `[x]` Implement `ConfirmationToken::confirm(..)` and `ConfirmationTokem::reject(..)` to handle email confirmation/rejection
- `[x]` Create axum handler functions `handle_token_confirmation(..)` and `handle_token_rejection(..)`
- `[x]` Create HTTP routes `POST /confirmations/{token_id}` for token confirmation and `DELETE /confirmations/{token_id}` for token rejection
- `[x]` Create `ConfirmationToken::delete_by_player_id(..)` function to erase all tokens matching a player
- `[x]` Update `Player::find*` functions to return a DBoError instead of a MongoError
- `[x]` Implement `Player::resend_confirmation_email(..)` function
- `[x]` Make HTTP routes more REST compliant by pluralizing `players`
- `[x]` Add `email_verified` and `proposed_email` fields to player struct, so that if in the future players change their email address, their account won't be immediately deleted
- `[x]` Implement `Player::verify_new_email(..)` function
- `[x]` Implement `Player::find_by_email_or_username(..)` for use in login
- `[x]` Add `created` field to `Player`, and create a partial TTL index to players which will delete **unconfirmed** accounts after two days
- `[x]` Implement `Player::login(..)` function. Currently, this works to verify that an email and password match, but it needs to be extended once JWT functionality is enabled.
- `[x]` In order to enforce environment-related panicking at startup instead of after an HTTP endpoint is activated, export a static ENV variable that will be lazy loaded on startup, and can be used within any module.
- `[x]` Fix inconsistencies within player confirmation process - be able to distinguish between "player already confirmed", "player account missing", "token expired", etc.

## Incomplete

- `[ ]` Update (and finally commit to git) my API documentation
- `[ ]` Reconfigure currently written functionality to support layered functionality and further enforce the Single Responsibility Principle.
  - Database models should only handle database queries (and input validation on construction)
  - The service functions should perform all the operations necessary to complete the request
  - The handler functions should parse the HTTP request, call the service functions, and then map the result to a valid HTTP response.
- `[ ]` Enable JWT functionality for authentication tokens **and** refresh tokens
- `[ ]` Allow players to change their email address
- `[ ]` Allow players to change their passwords
- `[ ]` Expand unit testing
- `[ ]` Add CORS restrictions instead of allowing all origins and methods
- `[ ]` Create a `docker-compose.yaml` file for local dev environment including frontend, backend, and mongodb database
- `[ ]` Create `DBoGame` struct and functions to enable gameplay
- `[ ]` Allow players to invite one another to a game of D-Bo
- `[ ]` Enable basic web socket functionality to allow players to actually take turns and play together in real time
- `[ ]` Allow players to befriend one another
- `[ ]` Enable "house rules" which can be established upon game creation
- `[ ]` Track wins, losses, draws, and dropouts between individual players

- `[ ]` Deploy backend at `https://api.d-bo.bigdevdog.com`

## Optional

- `[ ]` Enable more than two players to join a single lobby.

- `[ ]` Enable in-game messaging between players
