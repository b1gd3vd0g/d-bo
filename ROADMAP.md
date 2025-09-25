# D-Bo Roadmap to Completion

The goal is to have this project finished by Halloween. I have already made progress on player registration and account sign-up, as well as planning out processes, database schemas, and features that I hope to include.

The Halloween deadline gives me approximately six weeks to finish the project. In this document, I will assign tasks to complete for each week, approximately in the order I should complete them.

First, however, I will note important things that must be handled as I progress through the project.

## Tasks to keep up with

As I continue to add new features, it is very important that I maintain the integrity of the codebase. I should be documenting the code thoroughly; every module, every function, every class/struct needs to have a documentation comment. I want the purpose of my code to be very clear. In-line comments are less necessary, as I should strive to keep my code readable with helpful variable/function names, using spacing to group related functionality, etc.; however, it can still be included when it feels needed.

On that same note, my API documentation as well as the WebSocket documentation should not fall behind my code. It is ideal to do the documentation for a specific endpoint before starting to program it. If that does not happen, then the documentation at the very least should be no more than one endpoint behind the codebase; that is, if I code the functionality for the refresh endpoint, I should finish the API documentation for the refresh endpoint _before_ starting on building functionality for the password change endpoint.

Finally, it is important to test the code for errors as much as is necessary. This includes unit testing and integration testing as progress is made.

On a related note, a basic CI/CD pipeline should be made as soon as possible. Maybe it should simply ensure that the project can compile and run unit tests at first, but later should be improved upon to create Docker images, deploy a test environment using Docker Compose, and eventually update a production environment upon a push to the master branch.

## Weekly schedule

As previously mentioned, I have a bit less than six weeks to complete this project. The following is my plan for what should be completed week by week. This schedule can be adjusted as new requirements are discovered, or if progress is slower or faster than originally expected. However, all efforts should be made to have an acceptable, functional deliverable produced by October 31.

---

### Sprint 1: September 23 - 28 (6 days)

**(Backend) Finish core authentication flows and account management functionality**

- Create a basic CI/CD pipeline for the backend
  - For now, it can simply run unit tests and build the application.
- Complete email confirmation flow
- Implement refresh token endpoint
- Add account management endpoints
  - Change email (with confirmation)
  - Change username
  - Change password (cannot match last 5)
- Test functionality
  - Register -> Confirm email -> Log in -> Refresh -> Update account info

---

### Sprint 2: September 29 - October 4 (6 days)

**(Backend) Implement friendship systems between players**

- Search for players by username
- Friend requests
  - Send/cancel requests to other players
  - Accept/reject requests from other players
- Retrieve a list of your friends

---

### Sprint 3: October 5 - 11 (7 days)

**(Backend) Finish game creation and gameplay, and implement WebSocket functionality.**

- Create a game lobby; set rules and invite friends to join it.
- Create a struct to manage game state; implement functions to alter the game state only in allowed ways, taking into account the custom rulesets.
- Create WebSocket functionality
  - Join/leave a game lobby
  - Take your turns to alter the game state
- Unit test socket events and state transitions

---

After Sprint 3, we begin to focus mostly on frontend development. It is important to note at this point that this application will most commonly be used on mobile devices; therefore, it is important to ensure that the UI looks good on a smartphone screen **and** computer screens.

---

### Sprint 4: October 12 - 18 (7 days)

**(Frontend) Finish frontend authentication and account management functionality**

- Make login/register screens actually connect to the backend
- Create email confirmation/rejection page
- Implement account settings screen to include:
  - Change email
    - Must verify a new email address before it is actually used.
  - Change password
    - Must not match the last five passwords used
    - May not be done while an email address change is pending
    - Sends an email to the player confirming that they have changed their password.
  - Change username
  - Change preferred language (English/Spanish)
- Test end-to-end functionality in browser.
- _(If possible)_ Deploy the application to production, so that friends can create accounts and assist in testing as player-to-player interactions become available.

---

### Sprint 5: October 19 - 25 (7 days)

**(Frontend) Finish social functionality, and implement real-time gameplay between players**

- Deploy the application to production, so that friends can create accounts and assist in testing as player-to-player interactions become available.
  - The CI/CD pipeline should be updated by this point as well, so that pushes to the master branch are immediately reflected on the frontend.
- Build friend search/friend list UI
- Friend requests:
  - Ensure requests to other players can be sent/cancelled.
  - Ensure requests from other players can be accepted/rejected.
- Build lobby creation/invite functionality
  - Allow invitees to accept/reject invitations
- Implement WebSocket functionality for gameplay
- Show game state changes on-screen
  - Ensure current game state can accurately be shown
  - Implement transitions - so when Player 1 adds a card to a pile, the card will be shown moving from their hand to the pile and flipping over.

---

### Sprint 6: October 26 - 31 (6 days)

**Ensure Spanish translation is functional and acceptable, polish appearances, and test thoroughly**

- Ensure that the frontend supports both English and Spanish
  - On screens where no account is logged in, the preference should be found from navigator.language, and then stored in a LocalStorage cookie.
  - On screens where an account _is_ logged in, the preference should be the one stored in the account; the cookie should be ignored.
- Ensure Spanish translations are accurate
  - Ask friends to explore the app and verify
- Ensure the frontend styling is attractive, consistent, and intuitive.
- Verify that email templates are consistent and branded; ensure they are always sent in the player's preferred language.
- End-to-end QA
  - Create account -> Confirm account -> Log in -> Change account details -> Add friend -> Accept requests -> Create game and invite friends -> Play -> End game
- Fix bugs
- Finalize deployment

## Extras

After these six weeks are completed, the application should be fully functional, and players should be able to connect and play with each other. However, there are features that may not be **necessary** but will certainly add a lot of value to the application. These should most likely be completed _after_ the previously mentioned functionality is implemented, but could also be finished at other points throughout development. These features include the following:

- Allow players to create a custom avatar to represent them. The avatar will be a pixel-art sprite created from presets - they may have different hair styles, skin tones, eye shapes, etc.; however, they do not just draw their own sprites, nor will they upload their own photos.
- Enable player-to-player messaging within game lobbies; _possibly_ enable messaging between friends outside of game lobbies (but that's not as important).
- Keep track of player statistics:
  - Between two players, keep track of wins/losses/draws/dropouts **specifically** between the two players
  - As a whole, keep track of **all** wins/losses/draws/dropouts for a single player.
- Enable matchmaking for players who wish to be matched up with random players who they are not friends with
  - This matchmaking should try to find players with similar statistics
  - It should also try to find **active** players
    - I'm not sure what I mean by this yet; players who have played many games recently, players who have signed on recently, players who are **currently seeking connections**, players who are **currently signed in**... TBD.