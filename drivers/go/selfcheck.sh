#!/bin/sh
# Build the driver and feed it `hello` and the spec's direct-hpke-base vector
# through `open`, over stdin/stdout exactly as the runner does.
set -eu
cd "$(dirname "$0")"
go build -o tsp-driver .
python3 - <<'PY' | ./tsp-driver
import base64, json
f = json.load(open("../../fixtures/spec-vectors.json"))
v = f["vectors"]["direct-hpke-base"]
def ident(name, private):
    i = f["identifiers"][name]
    keys = ["id", "sigKeyType", "encKeyType", "pkS", "pkE"] + (["skS", "skE"] if private else [])
    return {k: i[k] for k in keys}
msg = base64.urlsafe_b64encode(base64.urlsafe_b64decode(v["message"] + "==")).decode().rstrip("=")
print(json.dumps({"id": 1, "op": "hello"}))
print(json.dumps({"id": 2, "op": "open", "receiver": ident(v["receiver"], True),
                  "sender": ident(v["sender"], False), "message": msg}))
PY
