#!/usr/bin/env python3
# Generates tests/testdata/db.cbor with known users for integration tests.
#
# Users created:
#   valid_user   — has a valid session token (expires far in the future)
#   expired_user — has an expired session token
#   init_user    — has an unused init_token, no valid session token
#
# Token values are written to tests/testdata/tokens.yaml so the test
# script can read them without hardcoding.

import cbor2
import uuid
import os
import datetime
import yaml

OUTDIR = os.path.join(os.path.dirname(__file__), "testdata")
os.makedirs(OUTDIR, exist_ok=True)

now = datetime.datetime.now(datetime.timezone.utc)

# --- user definitions ---

valid_uid        = uuid.UUID("00000000-0000-0000-0000-000000000001")
expired_uid      = uuid.UUID("00000000-0000-0000-0000-000000000002")
init_uid         = uuid.UUID("00000000-0000-0000-0000-000000000003")
second_init_uid  = uuid.UUID("00000000-0000-0000-0000-000000000004")

valid_token        = "valid-session-token-0000000000000000000000000000000000000000000000"
expired_token      = "expired-session-token-000000000000000000000000000000000000000000000"
init_token         = "init-token-00000000000000000000000000000000000000000000000000000000"
second_init_token  = "second-init-token-00000000000000000000000000000000000000000000000000"

def ts(dt):
    # ciborium serialises DateTime<Utc> as an RFC3339 string
    return dt.strftime("%Y-%m-%dT%H:%M:%S.%fZ")

users = {
    valid_uid.bytes: {
        "id": valid_uid.bytes,
        "name": "alice",
        "token": valid_token,
        "token_issued_at": ts(now - datetime.timedelta(hours=1)),
        "token_expires_at": ts(now + datetime.timedelta(days=7)),
        "init_token": None,
    },
    expired_uid.bytes: {
        "id": expired_uid.bytes,
        "name": "bob",
        "token": expired_token,
        "token_issued_at": ts(now - datetime.timedelta(days=14)),
        "token_expires_at": ts(now - datetime.timedelta(days=7)),
        "init_token": None,
    },
    init_uid.bytes: {
        "id": init_uid.bytes,
        "name": "carol",
        # session token also expired so only init_token can be used
        "token": "placeholder-expired-token-00000000000000000000000000000000000000000",
        "token_issued_at": ts(now - datetime.timedelta(days=14)),
        "token_expires_at": ts(now - datetime.timedelta(days=7)),
        "init_token": init_token,
    },
    # dave: used to test that init_token survives a dropped connection
    second_init_uid.bytes: {
        "id": second_init_uid.bytes,
        "name": "dave",
        "token": "placeholder-expired-token2-0000000000000000000000000000000000000000000",
        "token_issued_at": ts(now - datetime.timedelta(days=14)),
        "token_expires_at": ts(now - datetime.timedelta(days=7)),
        "init_token": second_init_token,
    },
}

db = {"users": users, "chores": {}, "events": {}}

with open(os.path.join(OUTDIR, "db.cbor"), "wb") as f:
    cbor2.dump(db, f)

tokens = {
    "valid_token":       valid_token,
    "expired_token":     expired_token,
    "init_token":        init_token,
    "second_init_token": second_init_token,
}
with open(os.path.join(OUTDIR, "tokens.yaml"), "w") as f:
    yaml.dump(tokens, f)

print(f"Generated {OUTDIR}/db.cbor and tokens.yaml")
