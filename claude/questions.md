# Questions / Action items for guardian

## From issue #2 prototype work

**Q1. Android target**
The design says "web app and android app, one codebase preferred".
For the web side, Rust→WASM works. For Android, what is the intended approach?
Options:
- Progressive Web App (works in Android Chrome with home-screen install, near-zero extra work)
- Native Android app using the Rust core via the Android NDK + a thin Kotlin/Compose shell
- Something else (Tauri Mobile, etc.)?

*Which of these will start the fastest? Efficiency / latency is the key here.*

**Q2. Data git repo for hourly snapshots**
The design says "dump into a special git repo and commit if changes happened".
Where does this repo live? Should `chorgly-backend` initialise it if absent, or must the admin
create it beforehand? What path/URL should be configurable?

*Initialize if absent, at configurable local path.*

**Q3. External event dependencies**
Chores can depend on "external events". How are these defined and triggered?
Options:
- Admin posts to a special WebSocket message / HTTP endpoint
- A named event is declared in the DB and a script fires it
- Something else?

*A list of external events is maintained, which the user needs to watch for.
The user explicitly ticks off that something has happened. These may be
later augmented by e.g. specific website watchers, but that's not a problem
for today.*

**Q4. Personal chore semantics (`assigned_to: []`)**
When a user taps "Add my chore", the prototype sends `assigned_to: []` (empty list) and relies
on the server to substitute the current user's ID. Should `assigned_to: []` mean "only the
creator can see/complete this", or should the client always send `assigned_to: [<my-id>]`
explicitly? (The WASM frontend can expose the user ID, but it needs a clear convention.)

*Let's make it like this: Every chore has a property of who can see it, who is
the primary assignee, and who can tick it off. Default for common chores: All
can see it, no assignee, everybody tick off. Default for private chores: Only
me see it, me assignee, me gonna tick off. All these things should be possible
to change.*

**Q5. Recurring chore with multiple assignees — who resets it?**
If a chore is assigned to a group (e.g. Alice and Bob) and Bob marks it done, does the
completion reset the timer for both? Or does Alice still need to complete it independently?

*It always resets the timer for all. But it may be handy to have a special issue
later which would be actually a block of N similar issues specific for each
user separately.*

**Q6. Token delivery mechanism**
The admin script prints a login URL. Should the app read the token from the URL query string on
first load, store it in localStorage, and then redirect to the clean URL? (Current prototype
requires manual paste.)

*Yes. Read, store, redirect, use invisibly.*
