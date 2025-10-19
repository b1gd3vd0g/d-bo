# TO DO

The following is a list of tasks to complete on the day assigned. The list of tasks for each day should ideally be completed **in order** - therefore, when creating the list, the more important tasks should be first on the list.

This document should turn into a checklist of tasks completed on a day-by-day basis. If a task which is already assigned for a _future date_ is completed early, the task should be **moved to the bottom** of the date on which it was completed. If a task is _not completed_ on the day which it was assigned, it should be **left unmarked** on its assigned completion date, and **moved to the top** of the date on which it is to be completed. It should be completed before moving on to the current day's tasks.

Days on which no progress is made, and on which no tasks were previously assigned, should not be included in this list at all.

## Table of Contents

- [Note on Priorities](#note-on-priorities)
  - [Fix Compilation Errors](#fix-compilation-errors)
  - [Fix Bugs](#fix-bugs)
  - [Ensure Existing Code Quality](#ensure-existing-code-quality)
  - [Document Existing API Endpoints](#document-existing-api-endpoints)
  - [Unit Testing](#unit-testing)
  - [Document Existing Simple Code](#document-existing-simple-code)
- [19/10/2025](#19-october-2025)
- [20/10/2025](#20-october-2025)
- [21/10/2025](#21-october-2025)

## Note On Priorities

The priorities of assigning tasks should more or less follow the guidelines below. Guidelines are listed in **decreasing** order of priority.

### Fix Compilation Errors

The goal for commits to this project is that **every commit should compile properly** - if a compilation error is present upon the start of the day, the **first priority** should be to fix it.

### Fix bugs

After the code can compile, we want to make sure that the code is functioning as expected. If a newly created functionality requires a time zone within the player account, we need to work our way backward to ensure that the time zone is provided upon account registration. If we add parameters to a function, and fix their calls with temporary literals, we should ensure that the placeholder is replaced by functional code as soon as possible.

### Ensure Existing Code Quality

Before implementing new features, we should ensure that our existing code is of the highest quality. In order, I'd say that we should ensure quality in the following order:

1. Ensure functionality is found in the proper _layer_. Our current architecture consists of four layers: **handlers** -> **services** -> **repositories** -> **models**.
   - The **handler layer** should parse HTTP requests, call the **service layer** to perform actions, and use that result in order to send a meaningful HTTP response back to the client.
   - The **service layer** should make calls to the **repository layer** in order to query and make changes to the database, perform any side effects, and return a meaningful result to the **handler layer**.
   - The **repository layer** should define all allowed database queries. Each repository affects a _single_ collection of **models** within the database. Each function in this layer should perform do _one thing_ very simply - the **service layer** can make multiple calls to the repository layer in order to chain actions according to business rules.
   - The **model layer** defines the shape of documents within a single **repository**. It provides **constructors**, **getters**, and **validation functions** for a single model.
2. Ensure that functions and their parameters are named clearly - the function name should clearly explain **what** the function does, and the required parameters should be obvious based on the combination of **name** and **type**.
3. Ensure that code **within** functions is not only _functional_, but **legible**. In-line comments should be seldomly used, as functionality should be clear based on variable names, return types, and spacing.
4. Ensure that existing code is thoroughly documented. This is more important in some places, and less so in others. For example, the **service layer** should be thoroughly documented - _every_ side effect that the function performs should be listed in the description, and _every_ possible error should be listed - these functions can be very complex, as they have a tendency to perform several actions. Conversely, the purpose of structs, fields, and most functions within the model layer are usually quite clear based on naming and return types. While these should still be documented, their priority is lesser.

### Document Existing API Endpoints

We don't want to complete our REST API without completing documentation for a single endpoint - this will cause two problems: `1` When we finally reach the stage where we have to make documentation, it will become _extremely_ tedious (which will likely reduce documentation quality), and `2` We will likely discover that our API has inconsistencies, causing us to refactor the API anyway.

The _goal_ is to never be more than one endpoint ahead of the documentation within the actual codebase. Sometimes, it will be simple to implement several related endpoints at once - but before moving on to unrelated endpoints, the documentation should be finished for the current ones.

### Implement New Features/Endpoints

We want to keep moving forward, making consistent progress toward completion of the application - we don't want to get stuck refactoring existing features if it is not necessary.

### Unit Testing

Unit testing is complex for some of the functionalities of this codebase. Truthfully, I am not shooting for 100% code coverage with tests. However, most **adapter functions** that perform a single thing should be tested - if this functionality is not working right, then it is obvious that higher-level functionality will not work right either.

For **database queries**, **email sending**, and other adapter functionalities that require an internet connection, it is simpler to test the functionality by calling the APIs and later checking external sources (the database, my email inbox) to ensure that the desired actions have been performed properly. These should be tested in this manner **as soon as functionality is complete**, to ensure the codebase is working as desired early on, and so as not to compound multiple errors and over-complicate debugging.

### Document Existing Simple Code

As mentioned in step 4 of **Ensure Existing Code Quality**, some codebase documentation is less important than others. At the end of the project (or whenever I feel "in the mood"), I should work on ensuring that **all** code is documented thoroughly, so that running `cargo doc` would be sufficient for a _stranger to the codebase_ to fully understand what is going on in every layer.

---

## 19 October 2025

- `[X]` Implement Time Zones in the player registration function, so that all new documents created include a time zone.
- `[ ]` Fix the `PlayerService::login(..)`, `send_lockout_email(..)`, and `format_date_time(..)` functions to work as desired.
- `[ ]` Unit test the `format_date_time(..)` function thoroughly. Include:
  - `[ ]` Tests in English for several dates, including several months, and both AM and PM hours
  - `[ ]` Tests in Spanish for several dates, including several months, both AM and PM hours, ensuring to test that 1:00 AM says "a la" instead of "a las".
- `[ ]` Fix API documentation (where TODO comments are already placed) to ensure compliance with the `RESPONSES.md` document.
- `[ ]` Fix the actual API endpoints to agree with the documentation
- `[ ]` Thoroughly read the API documentation to ensure **all endpoints** return proper **response codes** (in alignment with the `RESPONSES.md` document) - add response bodies to differentiate different errors which return the same response code.
- `[ ]` Document the four endpoints which **update (proposed) login credentials**
- `[ ]` Implement the handler functions to return the proper error responses, in alignment with the updated API documentation.

## 20 October 2025

- `[ ]` Implement the endpoint for "undo password change", allowing the player to change their password via the secure link in the email, following a password change that was (probably) not actually authorized by the player
- `[ ]` Implement the endpoint for "undo pending email change", allowing the player to remove the proposed email address, following a proposed email address change that was (probably) not actually authorized by the player
- `[ ]` Document these two endpoints in the API documentation
- `[ ]` Implement the following **simple** player account changes:
  - `[ ]` Update preferred language
  - `[ ]` Update player gender
    - `[ ]` This should allow the player to choose their preferred _pronouns_ as well, **if** their preferred language is Spanish and their gender is Other. If the preferred pronoun is not included, default to **Other**, resulting in the "-e" endings.
- `[ ]` Document the **simple** player account changes in the API documentation

## 21 October 2025

- `[ ]` Implement an endpoint for **requesting a password reset**, for when a player cannot remember their login credentials.
  - `[ ]` This should send an email to the player, providing them with a secure link to reset their password.
  - `[ ]` This should **probably** require a new type of persistent token - a **password reset token**, valid for 15 minutes. This function will **create** the token, and provide its info in the secure link.
- `[ ]` Implement an endpoint for **resetting a password via reset token**.
  - `[ ]` This should provide the player ID and the reset token ID, ensure it is valid, matches, and unexpired, and ensure that the new password does not match the last 5 passwords.
  - `[ ]` This should **NOT** be available while the account is **locked**!

> ---
>
> After this point, a small celebration should be had, because **finally** we have reached the finish line for **player account management**!
>
> The next big step is **inter-player interactivity** - searching for other players by filters, sending/cancelling friend requests, accepting/rejecting friend requests, and listing a player's existing friendships.
>
> ---
