Create a chore organizer among multiple people. It should keep track of various chores which have to be done. There are multiple categories:

- one-time
- recurring after some time of last completion
- recurring at specific time
- with deadlines

Some chores have dependencies on other chores or external events.

Some chores are limited to certain users.

**Backend**

Is this a better job for SQL or NoSQL? Use either sqlite or CBOR-backed storage, with complete chore database in memory. Once an hour, dump into a special git repo and commit if changes happened.

**Frontend**

Web app and android app. One codebase preferred. Default interface: listing of pending chores, button to add my chore, button to add common chore. Do not ask for details, all chores are by default "remind me in 30 minutes". Will fix later.

**Auth**

Create users by a terminal script on the server. No passwords, login by token, renewed every day, expired in a week, init token passed as a link from the admin script. Reset token by admin script.
