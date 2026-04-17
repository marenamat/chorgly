# chorgly

A chore organizer for multiple people.

Track chores that need doing — one-time tasks, recurring ones, deadline-driven ones, and those with dependencies on other chores or external events. Each chore can be restricted to specific users.

## Features

- **Chore types**: one-time, recurring (by elapsed time or fixed schedule), deadline-driven
- **Dependencies**: chores can depend on other chores or external events
- **Multi-user**: chores can be assigned to specific users or shared
- **Auth**: token-based login, no passwords; admin creates users via terminal script
- **Backend**: WebSocket, CBOR storage, hourly git snapshots
- **Frontend**: web app (Rust/WASM + JS glue)

## Status

Early development. See [open issues](https://github.com/marenamat/chorgly/issues) for planned work.

## Initialization

Built on [claude-base](https://github.com/marenamat/claude-base). See that project for clanker setup.

```
./clanker-setup
```
