# On Changing Player Login Credentials

This document will summarize the separate processes for updating the login credentials of an existing player account. These must be handled safely, as allowing malicious users to perform any of these actions could effectively hand over possession of the account to said malicious user.

Login credentials include **username**, **password**, and **email address**.

## Table of Contents

- [Universal Processes](#universal-processes)
- [Changing Username](#changing-username)
- [Changing Password](#changing-password)
- [Changing Email](#changing-email)

## Universal Processes

All requests made which change a player's login credentials have some consistent features among them. These are important, as they will make any malicious user's job significantly harder. The universal processes are as follows:

### Valid Access Token _and_ Password

Naturally, _any_ request requiring a player to be **logged in** requires a valid Access JWT to authenticate them following [Bearer Authentication](https://swagger.io/docs/specification/v3_0/authentication/bearer-authentication/) guidelines.

However, potentially dangerous requests (such as updating login credentials, as well as deleting a player account) require a player to enter their **current password** as well.

With a valid access token and a current password, we can be _fairly_ sure that the player is who they say they are.

### Email Notifications

Whenever a player's login credentials are updated, an email notification should be sent to the player, informing them of this change. This _warning email_ must be sent to the player's **current, confirmed** email address.

### Invalidate Existing Sessions

Following any change to a player's login credentials, all existing sessions associated with that player should be terminated. This means that both **access JWTs** _and_ **refresh tokens** identifying the player should be invalidated.

The processes for each token are as follows:

- **Access Tokens** are invalidated by adding a field to the `Player` model: a `bson::DateTime` called `session_valid_after`. Whenever an access token is used to validate a player, its `iat` field must be checked. If the token was initialized _before_ `session_valid_after`, it will be unable to retrieve a player account.

- **Refresh Tokens** are invalidated by simply wiping them all from the database. That way, when the refresh endpoint is used, it will be unable to identify a player account.

### Potential Future Processes

There are a couple extra security measures that could be taken in order to ensure account security. These include:

#### Credential Change Logs

Whenever a user credential is changed, it will create a document in a collection "credential-changes". These logs will have a TTL of 30 days. The model would be something like:

```
{
    change_id: String,
    player_id: String,
    credential: String              // "username" | "email" | "password"
    old_value: Option<String>,      // None for passwords
    new_value: Option<String>,      // None for passwords
    initialized: bson::DateTime,
    completed: bson::DateTime,      // Only different for email changes
    ip: String
}
```

#### Rate Limiting

Rate limiting would involve the Credential Change Logs. It would create some rules like:

- **No more than 3 credential changes in a 24 hour period**
- **No more than 1 credential change per field in a 24 hour period**

These rules are not concrete; maybe I would change them and give them more thought as I start to actually work on them.

If any rate limiting rules were broken, I would return a `429 TOO MANY REQUESTS` response.

## Changing Username

Changing a username will not require any additional security steps at this time, as it is the least dangerous of all the requests. If a player's username is changed, they would be able to login to their account via email address/password and change it back (if the hacker did not beat them to it).

## Changing Password

Changing a player's password is a very dangerous action, and requires stricter security measures.

### Unique new password

When a player resets their password, they will be unable to set it to any of their last 5 passwords.

### Reset password using `Undo Token`

The warning email sent to the player would provide a secure link for the player to reset their password, without having to verify their current password (because they likely would not know it).

This would involve creating a new collection called **undo-tokens**, which would be able to undo credential changes performed by malicious users. Undo tokens should be good for a 24 hour period. The `Undo Token` model would look something like this:

```
{
    token_id: String,
    player_id: String,
    credential: String,         // "email" | "password"
    created: bson::DateTime
}
```

After a player resets their password in this way, their sessions should be invalidated again, following the [same process](#invalidate-existing-sessions) as listed above.

## Changing Email

Changing a player's email address is especially dangerous, as it would invalidate all of the player's active sessions, **and** result in them not getting notified of any further changes to their account. We must do everything we can to avoid this.

> **NOTE**: When changing an email address, the sessions are not invalidated until the change is _finalized_ - either by confirming the new email address, or by undoing it via secure link in the email.

### Proposed Emails and Confirmation

When a request to change a player's email address is performed, it does not immediately replace the player's email address. What it does is set the `proposed_email` field of the player document to the new value. The warning email is sent to the old email address, and a confirmation email is sent to the new email address. A `Confirmation Token` is created, identical to the ones used during initial account registration. The new email address must be confirmed within 15 minutes of proposing the change; else, the token is "expired" and will not work - the confirmation email will need to be resent once more.

When a new email address is simply **proposed** (not confirmed), it will **not receive** any correspondences except for confirmation emails.

### Undo change using `Undo Token`

In the warning email to the current email address, a link will be sent allowing the player to "undo" the change - resetting the `proposed_email` field to `None` and maintaining their current email address.

This involves the same collection of `Undo Tokens` as [changing a password](#reset-password-using-undo-token) - the tokens will be good for 24 hours; after 24 hours, the undo token will be "expired" and will no longer work.

This **must** be done before the new email is confirmed! If the new email is confirmed, this request will fail because the damage has already been done.

### Undo change from within account

If a player can still log into their account, they may undo the change from within the settings menu. This does not require any token, nor will it invalidate any existing sessions.
