# HTTP Responses

This document is made to ensure consistency is achieved in our REST API. The goal is to ensure that the same type of error does not result in a different response code for distinct endpoints.

## Table of Contents

- [Response Codes](#response-codes)
  - [Success Responses](#success-responses)
    - [`200 Ok`](#200-ok)
    - [`201 Created`](#201-created)
    - [`204 No Content`](#204-no-content)
  - [Client Error Responses](#client-error-responses)
    - [`400 Bad Request`](#400-bad-request)
    - [`401 Unauthorized`](#401-unauthorized)
    - [`403 Forbidden`](#403-forbidden)
    - [`404 Not Found`](#404-not-found)
    - [`409 Conflict`](#409-conflict)
    - [`410 Gone`](#410-gone)
    - [`415 Unsupported Media Type`](#415-unsupported-media-type)
    - [`422 Unprocessable Content`](#422-unprocessable-content)
  - [Server Error Responses](#server-error-responses)
    - [`500 Internal Server Error`](#500-internal-server-error)

## Response Codes

This section will list different response codes returned by our API, and the reasons that they will be returned. **The same reason should not be listed under any two response codes!**

There will be times when several reasons may return the same response code within the same HTTP endpoint. It is important that, in these cases, the _reason_ for failure is reflected within the HTTP response body.

---

### Success Responses

#### `200 Ok`

This response code should be used only when a request has been completed successfully _and_ the HTTP response body contains some sort of useful information for the client.

- This response may indicate that changes have been made to existing documents within the database, but no new documents have been created. In the case that a request has created new documents, a [`201 Created`](#201-created) response should be returned instead.

- If there is **not** any information to share within the response body, a [`204 No Content`](#204-no-content) response should be returned instead.

- In the case of **non-specific search queries** returning no results, this response should be returned (alongside an empty array), although some would say that a [`404 Not Found`](#404-not-found) response is more appropriate.
  - An example to clarify _non-specific search queries_ - a player searches for player accounts with a username matching the regular expression `/mr_avo/i`, but no documents exist which match.
  - This is not an error, but simply a successful search which yields no results.

#### `201 Created`

This response code indicates that the HTTP request has been completed successfully, and that new document(s) have been created within the database.

- If a new document has been created, it is best to return this HTTP response code, even if the HTTP response body is empty.

#### `204 No Content`

This response code indicates that the HTTP request has been completed successfully, but no information is passed along to the client within the HTTP response body.

---

### Client Error Responses

These response codes indicate that there was some sort of problem with the HTTP request (body, headers, etc.) that prevented the request from completing the desired action.

These codes should be the most common error codes sent out by our API - they do not indicate that there is a problem with our codebase, simply that the action could not be completed based on the state of the request.

#### `400 Bad Request`

This response code is the most general, and indicates that the response cannot be processed. These responses are generally returned fairly early. The reasons for returning this response are as follows:

- The HTTP Request body cannot be parsed into JSON.
  - This response is returned _automatically_ by the axum router.
  - Provides the client with a plaintext response body describing the error.
- Information provided within the HTTP request body does not pass validation checks (as described by business rules).
  - The most primary example of this is when a **username**, **password**, or **email address** is not valid.
- An authentication token could not be found on a request that requires it.
  - Requests requiring authentication via JWT follow the guidelines of [Bearer Authentication](https://swagger.io/docs/specification/v3_0/authentication/bearer-authentication/). This indicates that either:
    - The request is missing an `Authorization` header
    - The `Authorization` header is missing the `"Bearer "` prefix
    - There is no value following the `"Bearer "` prefix
  - This response is unlikely, if our frontend is designed correctly, and if the request is made by a **benign user**.

#### `401 Unauthorized`

This response code indicates that player authentication failed, and as such the request cannot be completed. This is a common response, as authentication is required for _most_ requests. The reasons for returning this response are as follows:

- **Invalid Login Credentials**

  - This is (possibly) only returned by the _login_ endpoint.
  - The request required a combination of (username _or_ email address) **and** password. The values provided within the HTTP request do not match the records in our database.
  - This could indicate _either_ that the username/email address do not exist in the database, **or** that the password does not match the found user.
    > _This distinction should **not** be passed along to the client!_

- **Incorrect Password**

  - This is similar to **invalid login credentials**, except that it occurs in requests which do not require the request to provide the username/email.
  - This is returned by requests proposing **changes to existing login credentials** (such as username, proposed email address, and password).
  - These requests usually require a **valid authentication token** to identify the player, then additionally their current password as a further security measure against malicious users.

- **Invalid Authentication Token**

  - This indicates that the provided authentication token cannot be parsed by our system.
  - It indicates that **something** was provided within the `Authorization` header, following [Bearer Authentication](https://swagger.io/docs/specification/v3_0/authentication/bearer-authentication/) guidelines, but that something was not a valid JWT for our system, for authentication purposes.
  - This response is unlikely, if the frontend is created correctly _and_ the request is being made by **benign users**.

- **Expired Authentication Token**

  - This indicates that the provided authentication token _was_ valid, but has since expired.
    > _Authentication JWTs have a lifetime of **15 minutes**!_
  - In this case, the **refresh endpoint** should be used to gain a new authentication token

- **Premature Authentication Token**

  - This indicates that the provided authentication token _is_ valid **and** unexpired; however, the player's sessions have been invalidated since the creation of the authentication token.
  - This indicates to the client that it is necessary to login (using login credentials) again, in order to regain valid authentication tokens.
    > _When this happens, the player's **refresh tokens** will also be deleted - therefore, the refresh endpoint is insufficient to correct this error._

#### `403 Forbidden`

This response code indicates that a player has been authenticated correctly, but does not have the permission to perform the desired request. The reasons for returning this response code are as follows:

- **Token Does Not Match Player**

  - This indicates that a **persistent token** (perhaps a _confirmation token_ or an _undo token_) exists, but does not correspond with the player identified by the authentication token.

- **Account Locked**

  - This is (probably) only returned by the **login endpoint**.
  - This indicates that either:
    - The login credentials were **incorrect**, and this login attempt resulted in a **lockout**.
    - The player was found (by username or email), but the account is **currently locked**.
  - The response body provides the UTC Date Time at which the account will become unlocked again.

- **Refresh Token Revoked**

  - This is only returned by the **refresh endpoint**.
  - This indicates that the refresh token was found, but has been revoked.
    - As refresh token revocation is not yet implemented (and may never be), it is possible that this reason may be obsolete.

#### `404 Not Found`

This indicates that a document (typically referenced by ID) cannot be found within the database; thus, the desired request cannot be performed. The reasons for this response code are as follows:

- **Document ID Not Found**

  - This indicates that a document referenced by ID could not be found within the database. Depending on the request, it could mean either:
    - The document has been **erased** (either due to _TTL expiration_ or _manually_ by a player's request), or
    - It never existed in the first place.

- **Valid authentication token references nonexistent account**

  - This indicates that a valid authentication token was provided, but the player account that it references no longer exists
    - Most likely, the player account was deleted.

> As described in [`200 Ok`](#200-ok), a _non-specific search query_ which yields no results should **not** return this error code, but rather a `200` response paired with an empty array.

#### `409 Conflict`

This response code indicates that a request cannot be completed, because it conflicts with the existing state of the database. The reasons for this request are as follows:

- **Action Has Already Been Performed**

  - This indicates that the requested action has already happened, and can't be performed again.
  - Some examples:
    - Attempting to **confirm** an account which has already been confirmed.
    - Attempting to **resend** a confirmation email when the account is already confirmed.

- **Action Conflicts With Current State of Document**

  - This indicates that the requested action may not be performed, due to the current state of that same document.
  - Some examples:
    - A player attempts to **reject** the account confirmation (thus deleting all related documents) after the account has already been confirmed
    - A player attempts to log in to an account which has not yet been confirmed
    -

- **A Provided Field Violates a Uniqueness Rule**

  - This indicates that a document cannot be created or updated because the proposed fields violate uniqueness rules.
  - Some examples of uniqueness rules that may be violated:
    - **Player usernames** must be _case-insensitively unique_
    - **Player email addresses** must be _case-insensitively unique_

#### `410 Gone`

The official description of this response code says "Resource once existed but is permanently removed". In the case of our application, it would know whether a resource has ever existed if it has been removed.

Instead, this code is used to indicate that a **persistent token** exists, but has expired. Some examples:

- A **confirmation token** (valid for 15 minutes) has expired.
- A **refresh token** (valid for 30 days) has expired.
- An **undo token** (valid for 24 hours) has expired.

> In the case of all but **confirmation tokens** (whose TTL index is 2 days), the TTL index will usually delete these tokens ~more or less~ at the same time they would expire - but it is always possible that the request is made _very shortly_ after expiration, before the TTL index has had time to take action; so checks should always be made.

#### `415 Unsupported Media Type`

This response code is used only in the case that an HTTP request body is not of the type required by our API.

The API **always** requires **JSON Request Bodies** (when any body is required at all) - therefore, this will only happen when the request fails to provide the `Content-Type: "application/json"` header.

This response code is handled automatically by the axum router, and provides the client with a plaintext HTTP response body describing the problem.

#### `422 Unprocessable Content`

This response code is used only in the case that the HTTP request body cannot be parsed into the data struct required by the HTTP router.

This response code is **only** ever returned _automatically_ by the axum router, and is accompanied by a plaintext HTTP response body describing the problem.

Some examples of what would cause this error:

- A _required_ field is missing from the request body
- A provided field is not of the correct type

---

### Server Error Responses

These response codes indicate that the request could not be completed due to a server-side error. Although these responses are possible at _nearly_ every single endpoint, they should be returned rarely.

These responses normally indicate that there is some sort of problem within our codebase that needs to be fixed. Whenever a server-error is returned, it should be **thoroughly logged**, and likely should be **reported** as soon as it occurs, so that action can be taken to prevent future instances.

#### `500 Internal Server Error`

This is the only server error response that gets returned from our API. It indicates that, due to some sort of problem with our codebase (or, possibly, with our database), the request could not be completed.

**All of these reasons are very unlikely**, however, some possible reasons for this error include:

- **Database Failure**

  - A query cannot be completed because the database connection cannot be made
  - A document within our database does not match the provided model, indicating data corruption.

- **Email Sending Error**

  - An email cannot be sent, because the sender credentials do not match, or the email body cannot be processed.
  - The receiver email address (which is validated by our application and then stored in our database) cannot be parsed into a `Mailbox`, indicating data corruption.

- **JWT Processing Error**

  - An authentication token cannot be parsed, not due to it being invalid, but due to a problem with our configuration.
  - An authentication token cannot be produced, due to a problem with our configuration, or its content being invalid.

- **Hashing Error**

  - A password (or refresh token secret) cannot be hashed and stored securely in the database.
  - A stored hash within our database cannot be parsed, indicating data corruption.

- **Time Zone Error**

  - A stored time zone within our database cannot be parsed into a `chrono_tz::Tz` struct, indicating data corruption.

## Response Bodies

// TODO: describe error response bodies (maybe)
