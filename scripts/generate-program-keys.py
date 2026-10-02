#!/usr/bin/env python3
"""Verify development program keypairs.

The private keys are not in git. This script checks that a local keypair, if
present, matches the public id committed in Anchor.toml. It does not mint a
replacement key. A new key would be a different program address.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

EXPECTED = {
    "oracle_core": "9CHifhFPRK8nJJpL2nTY17UfYC75aReR4qojVqNaLnoe",
    "oracle_registry": "8NuitXiyMGv6jE7WQq8mWaqGDyrDDRkgaNhroNnXkPXZ",
    "verification": "4f5BaSpKo64bUDEVSkn4h25b8bNspxi24FMfVa4sfurL",
}

ALPHABET = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"


def b58encode(data: bytes) -> str:
    number = int.from_bytes(data, "big")
    encoded = bytearray()
    while number:
        number, remainder = divmod(number, 58)
        encoded.append(ALPHABET[remainder])
    padding = 0
    for byte in data:
        if byte == 0:
            padding += 1
        else:
            break
    return (ALPHABET[:1] * padding + encoded[::-1]).decode()


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    failed = False
    for name, expected in EXPECTED.items():
        path = root / "keys" / "program" / f"{name}-keypair.json"
        if not path.exists():
            print(f"missing {path}", file=sys.stderr)
            print(f"expected public id {expected}", file=sys.stderr)
            failed = True
            continue
        raw = json.loads(path.read_text())
        if len(raw) != 64:
            print(f"{path} is not a 64-byte Solana keypair", file=sys.stderr)
            failed = True
            continue
        public = b58encode(bytes(raw[32:]))
        if public != expected:
            print(f"{name} local key is {public}, Anchor.toml has {expected}", file=sys.stderr)
            failed = True
            continue
        print(f"{name} {public}")
    if failed:
        print(
            "Refusing to generate a new keypair. Rotating a program id is a source change.",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
