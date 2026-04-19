#!/usr/bin/env python3
# Integration tests for WebSocket authentication.
#
# Expects a running chorgly backend at WS_URL (default ws://[::1]:8080/ws).
# Reads token values from tests/testdata/tokens.yaml.
#
# Test cases (all from issue #4):
#   1. access denied without prior Auth message
#   2. access denied with a wrong (unknown) token
#   3. access denied with an expired token
#   4. access allowed with a valid stored session token
#   5. access allowed with the first-access (init) token
#   6. first-access token still works if the first connection dropped before AuthOk
#   7. first-access token denied after a successful first login

import asyncio
import os
import sys
import cbor2
import yaml
import websockets


WS_URL = os.environ.get("WS_URL", "ws://[::1]:8080/ws")

TESTDATA_DIR = os.path.join(os.path.dirname(__file__), "testdata")


def load_tokens():
    with open(os.path.join(TESTDATA_DIR, "tokens.yaml")) as f:
        return yaml.safe_load(f)


def cbor_encode(msg: dict) -> bytes:
    return cbor2.dumps(msg)


def cbor_decode(data: bytes) -> dict:
    return cbor2.loads(data)


async def send_auth(ws, token: str) -> dict:
    await ws.send(cbor_encode({"Auth": {"token": token}}))
    raw = await ws.recv()
    return cbor_decode(raw)


async def send_list_all(ws) -> dict:
    await ws.send(cbor_encode("ListAll"))
    raw = await ws.recv()
    return cbor_decode(raw)


PASS = "\033[32mPASS\033[0m"
FAIL = "\033[31mFAIL\033[0m"
failures = []


def check(name: str, cond: bool, detail: str = ""):
    if cond:
        print(f"  {PASS}  {name}")
    else:
        print(f"  {FAIL}  {name}" + (f": {detail}" if detail else ""))
        failures.append(name)


async def run_tests():
    tokens = load_tokens()
    valid_token   = tokens["valid_token"]
    expired_token = tokens["expired_token"]
    init_token    = tokens["init_token"]

    print(f"Connecting to {WS_URL}")

    # 1. Access denied without prior Auth message.
    async with websockets.connect(WS_URL) as ws:
        resp = await send_list_all(ws)
        check(
            "access denied without token",
            "Error" in resp,
            repr(resp),
        )

    # 2. Access denied with wrong token.
    async with websockets.connect(WS_URL) as ws:
        resp = await send_auth(ws, "completely-wrong-token")
        check(
            "access denied with wrong token",
            "AuthFail" in resp,
            repr(resp),
        )

    # 3. Access denied with expired token.
    async with websockets.connect(WS_URL) as ws:
        resp = await send_auth(ws, expired_token)
        check(
            "access denied with expired token",
            "AuthFail" in resp,
            repr(resp),
        )

    # 4. Access allowed with valid session token; ListAll returns Snapshot.
    async with websockets.connect(WS_URL) as ws:
        resp = await send_auth(ws, valid_token)
        check(
            "access allowed with valid session token",
            "AuthOk" in resp,
            repr(resp),
        )
        resp = await send_list_all(ws)
        check(
            "ListAll succeeds after valid auth",
            "Snapshot" in resp,
            repr(resp),
        )

    # 4b. ListAll denied without prior auth.
    async with websockets.connect(WS_URL) as ws:
        resp = await send_auth(ws, "completely-wrong-token-for-listall-test")
        check(
            "auth fails for ListAll-denial test (precondition)",
            "AuthFail" in resp,
            repr(resp),
        )
        resp = await send_list_all(ws)
        check(
            "ListAll denied after failed auth",
            "Error" in resp,
            repr(resp),
        )

    # 5. Access allowed with first-access (init) token.
    async with websockets.connect(WS_URL) as ws:
        resp = await send_auth(ws, init_token)
        check(
            "access allowed with first-access token",
            "AuthOk" in resp,
            repr(resp),
        )

    # 6. First-access token still works if the first access failed (AuthFail on
    #    wrong token, then retry with the real init_token on a new connection).
    #    Uses "second_init_token" — a fresh token not yet consumed.
    second_init_token = tokens.get("second_init_token")
    if second_init_token:
      # First attempt: wrong token → AuthFail (must not affect second_init_token).
      async with websockets.connect(WS_URL) as ws:
        resp = await send_auth(ws, "wrong-token-on-first-attempt")
        check(
          "first attempt fails cleanly (precondition for test 6)",
          "AuthFail" in resp,
          repr(resp),
        )

      # Second attempt with the real init_token — must succeed.
      async with websockets.connect(WS_URL) as ws:
        resp = await send_auth(ws, second_init_token)
        check(
          "first-access token still works after a failed first attempt",
          "AuthOk" in resp,
          repr(resp),
        )
    else:
      print("  NOTE  'second_init_token' not in tokens.yaml; skipping test 6")

    # 7. First-access token denied after successful first login.
    #    Test 5 already consumed the init_token; try it again.
    async with websockets.connect(WS_URL) as ws:
        resp = await send_auth(ws, init_token)
        check(
            "first-access token denied after first successful login",
            "AuthFail" in resp,
            repr(resp),
        )

    return len(failures) == 0


def main():
    ok = asyncio.run(run_tests())
    if failures:
        print(f"\n{len(failures)} test(s) failed: {', '.join(failures)}")
        sys.exit(1)
    else:
        print("\nAll tests passed.")
        sys.exit(0)


if __name__ == "__main__":
    main()
