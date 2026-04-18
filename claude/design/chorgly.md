Create a chore organizer among multiple people. It should keep track of various chores which have to be done. There are multiple categories:

- one-time
- recurring after some time of last completion
- recurring at specific time
- with deadlines

Some chores have dependencies on other chores or external events.

Some chores are limited to certain users.

**Backend**

CBOR-backed storage, complete chore database in memory. Once an hour, dump into a special git repo and commit if changes happened. The data repo path is configurable; initialize the repo if absent. The default path is `./data`.

**Frontend**

Web app and android app. One codebase preferred (PWA is the preferred Android approach — near-zero extra work). Default interface: listing of pending chores, button to add my chore, button to add common chore. Do not ask for details, all chores are by default "remind me in 30 minutes". Will fix later.

Auth tokens are passed as URL query parameters (`?token=<value>`) on first load. The app reads the token, stores it in localStorage, redirects to the clean URL, then connects silently.

**Auth**

Create users by a terminal script on the server. No passwords, login by token, renewed every day, expired in a week, init token passed as a link from the admin script. Reset token by admin script.

**Chore permissions**

Every chore has three permission fields:
- `visible_to`: who can see the chore (None = everyone)
- `assignee`: single primary assignee (None = no specific assignee)
- `can_complete`: who can tick the chore off (None = everyone)

Defaults for common chores: `visible_to=None`, `assignee=None`, `can_complete=None` (all).
Defaults for personal chores: `visible_to=[me]`, `assignee=me`, `can_complete=[me]`.
All three fields can be changed by the creator.

**Recurring chores with multiple assignees**

Completing a recurring chore resets the timer for everyone — one completion from any user resets the whole group. A future issue may add a variant where N individual per-user sub-chores must each be completed separately.

**External events**

A list of named external events is maintained. Each event is either untriggered (pending) or triggered. The user explicitly ticks off that an event has happened. Chores may declare `depends_on_events: Vec<EventId>`; they stay blocked until all referenced events are triggered. Future work may add automated watchers (website scrapers, etc.).
