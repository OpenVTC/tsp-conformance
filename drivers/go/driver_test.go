package main

import (
	"bufio"
	"bytes"
	"encoding/base64"
	"encoding/json"
	"os"
	"strings"
	"testing"
)

// The self-check drives the real request loop (serve) with protocol lines,
// exactly as the runner does over stdin/stdout.

type fixture struct {
	Identifiers map[string]map[string]string `json:"identifiers"`
	Vectors     map[string]map[string]string `json:"vectors"`
}

func loadFixture(t *testing.T) *fixture {
	t.Helper()
	raw, err := os.ReadFile("../../fixtures/spec-vectors.json")
	if err != nil {
		t.Skipf("fixture not available: %v", err)
	}
	var f fixture
	if err := json.Unmarshal(raw, &f); err != nil {
		t.Fatal(err)
	}
	return &f
}

func identity(f *fixture, name string, private bool) map[string]any {
	id := f.Identifiers[name]
	out := map[string]any{
		"id": id["id"], "sigKeyType": id["sigKeyType"], "encKeyType": id["encKeyType"],
		"pkS": id["pkS"], "pkE": id["pkE"],
	}
	if private {
		out["skS"], out["skE"] = id["skS"], id["skE"]
	}
	return out
}

// qb64 -> binary domain -> base64url, as the runner does.
func wire(t *testing.T, qb64 string) string {
	t.Helper()
	b, err := base64.RawURLEncoding.DecodeString(qb64)
	if err != nil {
		t.Fatal(err)
	}
	return base64.RawURLEncoding.EncodeToString(b)
}

type reply struct {
	ID     int             `json:"id"`
	OK     bool            `json:"ok"`
	Result json.RawMessage `json:"result"`
	Error  *struct {
		Code    string `json:"code"`
		Message string `json:"message"`
	} `json:"error"`
}

func run(t *testing.T, reqs ...map[string]any) []reply {
	t.Helper()
	var in bytes.Buffer
	for i, r := range reqs {
		r["id"] = i + 1
		line, err := json.Marshal(r)
		if err != nil {
			t.Fatal(err)
		}
		in.Write(append(line, '\n'))
	}
	var out bytes.Buffer
	if err := serve(&in, &out); err != nil {
		t.Fatal(err)
	}
	var replies []reply
	sc := bufio.NewScanner(&out)
	sc.Buffer(nil, 64<<20)
	for sc.Scan() {
		var r reply
		if err := json.Unmarshal(sc.Bytes(), &r); err != nil {
			t.Fatalf("non-JSON output line %q: %v", sc.Text(), err)
		}
		replies = append(replies, r)
	}
	if len(replies) != len(reqs) {
		t.Fatalf("%d requests, %d replies", len(reqs), len(replies))
	}
	for i, r := range replies {
		if r.ID != i+1 {
			t.Fatalf("reply %d has id %d", i, r.ID)
		}
	}
	return replies
}

func result(t *testing.T, r reply) map[string]any {
	t.Helper()
	if !r.OK {
		t.Fatalf("request %d failed: %+v", r.ID, r.Error)
	}
	var m map[string]any
	if err := json.Unmarshal(r.Result, &m); err != nil {
		t.Fatal(err)
	}
	return m
}

func TestHelloAndOpenDirectHPKEBase(t *testing.T) {
	f := loadFixture(t)
	v := f.Vectors["direct-hpke-base"]
	replies := run(t,
		map[string]any{"op": "hello"},
		map[string]any{
			"op":       "open",
			"receiver": identity(f, v["receiver"], true),
			"sender":   identity(f, v["sender"], false),
			"message":  wire(t, v["message"]),
		},
	)
	hello := result(t, replies[0])
	if hello["language"] != "go" || hello["protocol"] != float64(1) {
		t.Fatalf("hello: %v", hello)
	}
	opened := result(t, replies[1])
	payload := opened["payload"].(map[string]any)
	if opened["scheme"] != "hpke-base" || payload["type"] != "scs" || payload["payloadSender"] != nil || payload["padding"] != "" {
		t.Fatalf("open: %v", opened)
	}
	if data, _ := base64.RawURLEncoding.DecodeString(payload["data"].(string)); string(data) != "hello world" {
		t.Fatalf("data %q", data)
	}
	if ver := opened["version"].(map[string]any); ver["major"] != float64(0) || ver["minor"] != float64(2) {
		t.Fatalf("version %v", ver)
	}
}

func TestAllVectorsOpenAndReproduce(t *testing.T) {
	f := loadFixture(t)
	for name, v := range f.Vectors {
		opened := result(t, run(t, map[string]any{
			"op": "open", "receiver": identity(f, v["receiver"], true), "sender": identity(f, v["sender"], false),
			"message": wire(t, v["message"]),
		})[0])
		payload := opened["payload"].(map[string]any)

		var eph any
		switch {
		case v["ikmE"] != "":
			eph = map[string]any{"ikmE": v["ikmE"]}
		case v["skEm"] != "":
			eph = map[string]any{"skEm": v["skEm"]}
		case opened["scheme"] != "signed-only":
			continue // direct-hpke-base-pq: open-only
		}
		in := map[string]any{}
		for _, k := range []string{"type", "data", "nonce", "replyPath", "referral", "digest", "hops", "inner", "padding", "payloadSender"} {
			if val, ok := payload[k]; ok {
				in[k] = val
			}
		}
		packed := result(t, run(t, map[string]any{
			"op": "pack", "scheme": opened["scheme"], "sender": identity(f, v["sender"], true),
			"receiver": identity(f, v["receiver"], false), "payload": in, "ephemeral": eph,
		})[0])
		if packed["message"] != wire(t, v["message"]) {
			t.Errorf("%s: pack did not reproduce the vector", name)
		}
		if strings.HasPrefix(name, "control-rf") && name != "control-rfd" && packed["digest"] == nil {
			t.Errorf("%s: pack reported no digest", name)
		}
	}
}

func TestEndpointsAndErrors(t *testing.T) {
	f := loadFixture(t)
	alice, bob := identity(f, "alice", true), identity(f, "bob", true)
	alicePub, bobPub := identity(f, "alice", false), identity(f, "bob", false)
	a, b := alice["id"], bob["id"]

	replies := run(t,
		map[string]any{"op": "endpoint.create", "identities": []any{alice}, "peers": []any{bobPub}},
		map[string]any{"op": "endpoint.create", "identities": []any{bob}, "peers": []any{alicePub}},
	)
	epA, epB := result(t, replies[0])["endpoint"], result(t, replies[1])["endpoint"]
	if epA == epB {
		t.Fatal("endpoint handles collide")
	}

	// Endpoint state lives in one driver instance; call handleLine directly so
	// each step can use the previous reply, as the runner does.
	d := newDriver()
	call := func(req map[string]any) reply {
		line, _ := json.Marshal(req)
		resp := d.handleLine(line)
		raw, _ := json.Marshal(resp)
		var r reply
		json.Unmarshal(raw, &r)
		return r
	}
	ea := result(t, call(map[string]any{"id": 1, "op": "endpoint.create", "identities": []any{alice}, "peers": []any{bobPub}}))["endpoint"]
	eb := result(t, call(map[string]any{"id": 2, "op": "endpoint.create", "identities": []any{bob}, "peers": []any{alicePub}}))["endpoint"]

	send := call(map[string]any{"id": 3, "op": "endpoint.send", "endpoint": ea, "from": a, "to": b, "data": "aGk"})
	if send.OK || send.Error.Code != "relationship" {
		t.Fatalf("send without relationship: %+v", send)
	}
	inv := result(t, call(map[string]any{"id": 4, "op": "endpoint.invite", "endpoint": ea, "from": a, "to": b}))
	ev := result(t, call(map[string]any{"id": 5, "op": "endpoint.receive", "endpoint": eb, "message": inv["message"]}))
	if ev["event"] != "invite" || ev["digest"] != inv["digest"] || ev["from"] != a || ev["to"] != b {
		t.Fatalf("receive invite: %v", ev)
	}
	st := result(t, call(map[string]any{"id": 6, "op": "endpoint.state", "endpoint": eb, "local": b, "remote": a}))
	if st["state"] != "invite-received" {
		t.Fatalf("state: %v", st)
	}
	acc := result(t, call(map[string]any{"id": 7, "op": "endpoint.accept", "endpoint": eb, "from": b, "to": a}))
	if acc["digest"] != inv["digest"] || acc["replyDigest"] == nil {
		t.Fatalf("accept: %v", acc)
	}
	ev = result(t, call(map[string]any{"id": 8, "op": "endpoint.receive", "endpoint": ea, "message": acc["message"]}))
	if ev["event"] != "accept" || ev["replyDigest"] != acc["replyDigest"] {
		t.Fatalf("receive accept: %v", ev)
	}
	msg := result(t, call(map[string]any{"id": 9, "op": "endpoint.send", "endpoint": ea, "from": a, "to": b, "data": "aGk"}))
	ev = result(t, call(map[string]any{"id": 10, "op": "endpoint.receive", "endpoint": eb, "message": msg["message"]}))
	if ev["event"] != "message" || ev["data"] != "aGk" {
		t.Fatalf("receive message: %v", ev)
	}
	st = result(t, call(map[string]any{"id": 11, "op": "endpoint.state", "endpoint": ea, "local": a, "remote": b}))
	if st["state"] != "bidirectional" || st["digest"] != inv["digest"] || st["replyDigest"] != acc["replyDigest"] {
		t.Fatalf("state: %v", st)
	}
	cancel := result(t, call(map[string]any{"id": 12, "op": "endpoint.cancel", "endpoint": eb, "from": b, "to": a}))
	ev = result(t, call(map[string]any{"id": 13, "op": "endpoint.receive", "endpoint": ea, "message": cancel["message"]}))
	if ev["event"] != "cancel" {
		t.Fatalf("receive cancel: %v", ev)
	}

	// Error codes.
	v := f.Vectors["direct-hpke-base"]
	tampered, _ := base64.RawURLEncoding.DecodeString(v["message"])
	tampered[len(tampered)-1] ^= 1
	for _, c := range []struct {
		req  map[string]any
		code string
	}{
		{map[string]any{"op": "nope"}, "unsupported"},
		{map[string]any{"op": "open", "receiver": bob, "sender": alicePub, "message": base64.RawURLEncoding.EncodeToString(tampered)}, "signature"},
		{map[string]any{"op": "open", "receiver": alice, "sender": bobPub, "message": wire(t, v["message"])}, "receiver"},
		{map[string]any{"op": "peek", "message": "AAAA"}, "malformed"},
		{map[string]any{"op": "open", "receiver": bob, "sender": alicePub, "message": "!!"}, "invalid-input"},
	} {
		r := call(c.req)
		if r.OK || r.Error.Code != c.code {
			t.Errorf("%v: got %+v, want %s", c.req["op"], r.Error, c.code)
		}
	}
	if resp := d.handleLine([]byte("not json")); resp.OK || resp.Error.Code != "invalid-input" {
		t.Fatalf("junk line: %+v", resp)
	}
}

func TestPackOptionsThroughTheDriver(t *testing.T) {
	f := loadFixture(t)
	alice, bob := identity(f, "alice", true), identity(f, "bob", true)
	alicePub, bobPub := identity(f, "alice", false), identity(f, "bob", false)
	q := f.Identifiers["q"]
	nonce := base64.RawURLEncoding.EncodeToString(bytes.Repeat([]byte{9}, 16))

	for _, scheme := range []string{"hpke-base", "sealed-box", "signed-only"} {
		for _, referral := range []any{
			nil,
			map[string]any{"vid": q["id"], "skS": q["skS"]},
			map[string]any{"vid": q["id"], "signature": base64.RawURLEncoding.EncodeToString(make([]byte, 64))},
		} {
			payload := map[string]any{
				"type": "rfi", "nonce": nonce, "replyPath": []string{"did:example:hop"},
				"referral": referral, "padding": "cGFk", "payloadSender": alice["id"],
			}
			packed := result(t, run(t, map[string]any{"op": "pack", "scheme": scheme, "sender": alice, "receiver": bobPub, "payload": payload})[0])
			opened := result(t, run(t, map[string]any{"op": "open", "receiver": bob, "sender": alicePub, "message": packed["message"]})[0])
			p := opened["payload"].(map[string]any)
			if opened["scheme"] != scheme || p["digest"] != packed["digest"] || p["digestAlg"] != packed["digestAlg"] ||
				p["padding"] != "cGFk" || p["payloadSender"] != alice["id"] || p["nonce"] != nonce {
				t.Fatalf("%s: %v / %v", scheme, packed, opened)
			}
			if (referral == nil) != (p["referral"] == nil) {
				t.Fatalf("%s: referral %v", scheme, p["referral"])
			}
			wantAlg := "sha2-256"
			if scheme == "sealed-box" {
				wantAlg = "blake2b-256"
			}
			if p["digestAlg"] != wantAlg {
				t.Fatalf("%s: digestAlg %v", scheme, p["digestAlg"])
			}
		}
	}
	r := run(t, map[string]any{"op": "pack", "scheme": "hpke-base", "sender": alice, "receiver": bobPub,
		"payload": map[string]any{"type": "scs", "data": "aGk", "payloadSender": "did:example:mallory"}})[0]
	if r.OK || r.Error.Code != "unsupported" {
		t.Fatalf("foreign payloadSender: %+v", r)
	}
}
