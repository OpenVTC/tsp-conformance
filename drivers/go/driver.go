package main

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"os"
	"runtime/debug"
	"strconv"

	tsp "github.com/affinidi/affinidi-tsp-go"
)

const (
	driverName    = "affinidi-tsp-go"
	driverVersion = "0.1.0"
)

// capabilities advertised by hello. Every entry is exercised by the library's
// own tests; see README.md for what each covers and the one qualification
// (deterministic packing is unavailable for hpke-pq, whose pack with an
// ephemeral answers unsupported).
var capabilities = []string{
	"hpke-base",
	"signed-only",
	"sealed-box",
	"hpke-pq",
	"payload.scs", "payload.ctl", "payload.pad",
	"payload.rfi", "payload.rfa", "payload.rfd",
	"rfi.reply-path", "rfi.referral",
	"payload.hop",
	"padding",
	"payload-sender",
	"deterministic",
	"peek",
	"endpoint",
}

type response struct {
	ID     json.RawMessage `json:"id"`
	OK     bool            `json:"ok"`
	Result any             `json:"result,omitempty"`
	Error  *errorBody      `json:"error,omitempty"`
}

type errorBody struct {
	Code    string `json:"code"`
	Message string `json:"message"`
}

// opError is a failure with a protocol error code.
type opError struct {
	code, msg string
}

func (e *opError) Error() string { return e.code + ": " + e.msg }

func fail(code, format string, args ...any) error {
	return &opError{code: code, msg: fmt.Sprintf(format, args...)}
}

func invalid(format string, args ...any) error { return fail("invalid-input", format, args...) }

type driver struct {
	endpoints map[string]*tsp.Endpoint
	next      int
}

func newDriver() *driver { return &driver{endpoints: make(map[string]*tsp.Endpoint)} }

func (d *driver) handleLine(line []byte) (resp *response) {
	line = bytes.TrimSpace(line)
	if len(line) == 0 {
		return nil
	}
	var head struct {
		ID json.RawMessage `json:"id"`
		Op string          `json:"op"`
	}
	resp = &response{ID: json.RawMessage("null")}
	if err := json.Unmarshal(line, &head); err != nil {
		resp.Error = &errorBody{Code: "invalid-input", Message: "request is not a JSON object: " + err.Error()}
		return resp
	}
	if len(head.ID) > 0 {
		resp.ID = head.ID
	}
	defer func() {
		if p := recover(); p != nil {
			fmt.Fprintf(os.Stderr, "panic in %s: %v\n%s", head.Op, p, debug.Stack())
			resp.OK, resp.Result = false, nil
			resp.Error = &errorBody{Code: "internal", Message: fmt.Sprintf("driver panic: %v", p)}
		}
	}()

	result, err := d.dispatch(head.Op, line)
	if err != nil {
		resp.Error = toErrorBody(err)
		return resp
	}
	resp.OK, resp.Result = true, result
	return resp
}

func toErrorBody(err error) *errorBody {
	if oe, ok := err.(*opError); ok {
		return &errorBody{Code: oe.code, Message: oe.msg}
	}
	return &errorBody{Code: tsp.KindOf(err).String(), Message: err.Error()}
}

func (d *driver) dispatch(op string, raw []byte) (any, error) {
	switch op {
	case "hello":
		return map[string]any{
			"name":         driverName,
			"version":      driverVersion,
			"language":     "go",
			"protocol":     1,
			"capabilities": capabilities,
		}, nil
	case "pack":
		return d.pack(raw)
	case "open":
		return d.open(raw)
	case "peek":
		return d.peek(raw)
	case "endpoint.create":
		return d.endpointCreate(raw)
	case "endpoint.invite", "endpoint.accept", "endpoint.cancel", "endpoint.send":
		return d.endpointSend(op, raw)
	case "endpoint.receive":
		return d.endpointReceive(raw)
	case "endpoint.state":
		return d.endpointState(raw)
	}
	return nil, fail("unsupported", "unknown op %q", op)
}

// ---------------------------------------------------------------------------
// Encoding helpers

func b64d(field, s string) ([]byte, error) {
	b, err := base64.RawURLEncoding.DecodeString(s)
	if err != nil {
		return nil, invalid("%s is not base64url without padding: %v", field, err)
	}
	return b, nil
}

func b64e(b []byte) string { return base64.RawURLEncoding.EncodeToString(b) }

func digestAlgName(a tsp.DigestAlg) string { return a.String() }

func parseDigestAlg(s string) (tsp.DigestAlg, error) {
	switch s {
	case "sha2-256":
		return tsp.DigestSHA2256, nil
	case "blake2b-256":
		return tsp.DigestBlake2b256, nil
	}
	return 0, invalid("unknown digestAlg %q", s)
}

func parseDigest(field, value string, alg tsp.DigestAlg) (tsp.Digest, error) {
	b, err := b64d(field, value)
	if err != nil {
		return tsp.Digest{}, err
	}
	if len(b) != 32 {
		return tsp.Digest{}, invalid("%s must be 32 bytes, got %d", field, len(b))
	}
	d := tsp.Digest{Alg: alg}
	copy(d.Value[:], b)
	return d, nil
}

func parseScheme(s string) (tsp.Scheme, error) {
	switch s {
	case "hpke-base":
		return tsp.SchemeHPKEBase, nil
	case "hpke-pq":
		return tsp.SchemeHPKEPQ, nil
	case "sealed-box":
		return tsp.SchemeSealedBox, nil
	case "signed-only":
		return tsp.SchemeSignedOnly, nil
	}
	return 0, fail("unsupported", "scheme %q", s)
}

// ---------------------------------------------------------------------------
// Identities

type identityJSON struct {
	ID         string  `json:"id"`
	SigKeyType string  `json:"sigKeyType"`
	EncKeyType string  `json:"encKeyType"`
	PkS        string  `json:"pkS"`
	PkE        string  `json:"pkE"`
	SkS        *string `json:"skS"`
	SkE        *string `json:"skE"`
}

func (j *identityJSON) private(field string) (*tsp.PrivateIdentity, error) {
	if j == nil {
		return nil, invalid("%s is required", field)
	}
	p := &tsp.PrivateIdentity{Identity: tsp.Identity{VID: j.ID}}
	switch j.SigKeyType {
	case "Ed25519":
		p.SigKeyType = tsp.SigKeyEd25519
	case "MlDsa65":
		p.SigKeyType = tsp.SigKeyMLDSA65
	default:
		return nil, fail("unsupported", "%s.sigKeyType %q", field, j.SigKeyType)
	}
	switch j.EncKeyType {
	case "X25519":
		p.EncKeyType = tsp.EncKeyX25519
	case "MLKEM768-X25519":
		p.EncKeyType = tsp.EncKeyMLKEM768X25519
	case "":
	default:
		return nil, fail("unsupported", "%s.encKeyType %q", field, j.EncKeyType)
	}
	var err error
	if p.SigningKey, err = b64d(field+".pkS", j.PkS); err != nil {
		return nil, err
	}
	if j.PkE != "" {
		if p.EncryptionKey, err = b64d(field+".pkE", j.PkE); err != nil {
			return nil, err
		}
	}
	if j.SkS != nil {
		if p.SigningPrivateKey, err = b64d(field+".skS", *j.SkS); err != nil {
			return nil, err
		}
	}
	if j.SkE != nil {
		if p.DecryptionPrivateKey, err = b64d(field+".skE", *j.SkE); err != nil {
			return nil, err
		}
	}
	if err := p.Validate(); err != nil {
		return nil, err
	}
	return p, nil
}

func (j *identityJSON) public(field string) (*tsp.Identity, error) {
	if j == nil {
		return nil, invalid("%s is required", field)
	}
	cp := *j
	cp.SkS, cp.SkE = nil, nil
	p, err := cp.private(field)
	if err != nil {
		return nil, err
	}
	return p.Public(), nil
}

// ---------------------------------------------------------------------------
// Payloads

type referralJSON struct {
	VID        string  `json:"vid"`
	Signature  *string `json:"signature"`
	SkS        *string `json:"skS"`        // extension: the introduced VID's signing key
	SigKeyType string  `json:"sigKeyType"` // extension: its type (default by key length)
}

type payloadJSON struct {
	Type          string          `json:"type"`
	Data          *string         `json:"data"`
	Nonce         *string         `json:"nonce"`
	ReplyPath     []string        `json:"replyPath"`
	Referral      *referralJSON   `json:"referral"`
	Digest        *string         `json:"digest"`
	DigestAlg     *string         `json:"digestAlg"`
	Hops          []string        `json:"hops"`
	Inner         *string         `json:"inner"`
	Padding       *string         `json:"padding"`
	PayloadSender json.RawMessage `json:"payloadSender"`
}

// buildPayload translates a request payload into a library payload and the
// pack options it implies.
func buildPayload(j *payloadJSON, scheme tsp.Scheme, sender *tsp.PrivateIdentity) (tsp.Payload, *tsp.PackOptions, error) {
	if j == nil {
		return nil, nil, invalid("payload is required")
	}
	opts := &tsp.PackOptions{Scheme: scheme}
	if j.Padding != nil {
		b, err := b64d("payload.padding", *j.Padding)
		if err != nil {
			return nil, nil, err
		}
		opts.Padding = b
	}
	switch ps := bytes.TrimSpace(j.PayloadSender); {
	case len(ps) == 0:
		opts.PayloadSender = tsp.SenderFieldDefault
	case string(ps) == "null":
		opts.PayloadSender = tsp.SenderFieldNull
	default:
		var s string
		if err := json.Unmarshal(ps, &s); err != nil {
			return nil, nil, invalid("payloadSender must be null or a string")
		}
		if s != sender.VID {
			return nil, nil, fail("unsupported", "payloadSender other than the envelope sender")
		}
		opts.PayloadSender = tsp.SenderFieldPresent
	}
	nonce := func() ([]byte, error) {
		if j.Nonce == nil {
			return nil, nil
		}
		return b64d("payload.nonce", *j.Nonce)
	}
	data := func() ([]byte, error) {
		if j.Data == nil {
			return nil, invalid("payload.data is required")
		}
		return b64d("payload.data", *j.Data)
	}
	digestAlg := func(def tsp.DigestAlg) (tsp.DigestAlg, error) {
		if j.DigestAlg == nil {
			return def, nil
		}
		return parseDigestAlg(*j.DigestAlg)
	}
	schemeAlg := tsp.DigestSHA2256
	if scheme == tsp.SchemeSealedBox {
		schemeAlg = tsp.DigestBlake2b256
	}

	switch j.Type {
	case "scs", "ctl":
		b, err := data()
		if err != nil {
			return nil, nil, err
		}
		if j.Type == "scs" {
			return &tsp.SCS{Data: b}, opts, nil
		}
		return &tsp.CTL{Data: b}, opts, nil
	case "pad":
		n, err := nonce()
		if err != nil {
			return nil, nil, err
		}
		return &tsp.PAD{Nonce: n}, opts, nil
	case "rfi":
		n, err := nonce()
		if err != nil {
			return nil, nil, err
		}
		rfi := &tsp.RFI{Nonce: n, ReplyPath: j.ReplyPath}
		if r := j.Referral; r != nil {
			rfi.Referral = &tsp.Referral{VID: r.VID}
			switch {
			case r.Signature != nil:
				if rfi.Referral.Signature, err = b64d("payload.referral.signature", *r.Signature); err != nil {
					return nil, nil, err
				}
			case r.SkS != nil:
				signer, err := referralSigner(r)
				if err != nil {
					return nil, nil, err
				}
				opts.ReferralSigner = signer
			default:
				return nil, nil, invalid("payload.referral needs a signature (or the introduced VID's skS)")
			}
		}
		return rfi, opts, nil
	case "rfa":
		if j.Digest == nil {
			return nil, nil, invalid("payload.digest is required")
		}
		alg, err := digestAlg(schemeAlg)
		if err != nil {
			return nil, nil, err
		}
		d, err := parseDigest("payload.digest", *j.Digest, alg)
		if err != nil {
			return nil, nil, err
		}
		return &tsp.RFA{InviteDigest: d}, opts, nil
	case "rfd":
		if j.Digest == nil {
			return nil, nil, invalid("payload.digest is required")
		}
		alg, err := digestAlg(tsp.DigestSHA2256)
		if err != nil {
			return nil, nil, err
		}
		d, err := parseDigest("payload.digest", *j.Digest, alg)
		if err != nil {
			return nil, nil, err
		}
		return &tsp.RFD{Digest: d}, opts, nil
	case "hop":
		if j.Inner == nil {
			return nil, nil, invalid("payload.inner is required")
		}
		inner, err := b64d("payload.inner", *j.Inner)
		if err != nil {
			return nil, nil, err
		}
		hops := j.Hops
		if hops == nil {
			hops = []string{}
		}
		return &tsp.HOP{Hops: hops, Inner: inner}, opts, nil
	}
	return nil, nil, fail("unsupported", "payload type %q", j.Type)
}

func referralSigner(r *referralJSON) (*tsp.PrivateIdentity, error) {
	sk, err := b64d("payload.referral.skS", *r.SkS)
	if err != nil {
		return nil, err
	}
	typ := tsp.SigKeyEd25519
	switch {
	case r.SigKeyType == "MlDsa65", r.SigKeyType == "" && len(sk) == 4032:
		typ = tsp.SigKeyMLDSA65
	case r.SigKeyType != "" && r.SigKeyType != "Ed25519":
		return nil, fail("unsupported", "payload.referral.sigKeyType %q", r.SigKeyType)
	}
	return tsp.NewPrivateIdentity(r.VID, typ, sk, tsp.EncKeyUnknown, nil)
}

func payloadResult(m *tsp.Message) (map[string]any, error) {
	out := map[string]any{"padding": b64e(m.Padding)}
	if m.PayloadSender == "" {
		out["payloadSender"] = nil
	} else {
		out["payloadSender"] = m.PayloadSender
	}
	switch p := m.Payload.(type) {
	case *tsp.SCS, *tsp.CTL:
		var data []byte
		if s, ok := p.(*tsp.SCS); ok {
			out["type"], data = "scs", s.Data
		} else {
			out["type"], data = "ctl", p.(*tsp.CTL).Data
		}
		out["data"] = b64e(data)
	case *tsp.PAD:
		out["type"], out["nonce"] = "pad", b64e(p.Nonce)
	case *tsp.RFI:
		out["type"] = "rfi"
		out["nonce"] = b64e(p.Nonce)
		out["replyPath"] = nonNil(p.ReplyPath)
		out["referral"] = nil
		if p.Referral != nil {
			out["referral"] = map[string]any{"vid": p.Referral.VID, "signature": b64e(p.Referral.Signature)}
		}
		out["digest"], out["digestAlg"] = b64e(p.Digest.Value[:]), digestAlgName(p.Digest.Alg)
	case *tsp.RFA:
		out["type"] = "rfa"
		out["digest"] = b64e(p.InviteDigest.Value[:])
		out["replyDigest"] = b64e(p.ReplyDigest.Value[:])
		out["digestAlg"] = digestAlgName(p.ReplyDigest.Alg)
	case *tsp.RFD:
		out["type"] = "rfd"
		out["digest"], out["digestAlg"] = b64e(p.Digest.Value[:]), digestAlgName(p.Digest.Alg)
	case *tsp.HOP:
		out["type"], out["hops"], out["inner"] = "hop", nonNil(p.Hops), b64e(p.Inner)
	default:
		return nil, fail("internal", "unexpected payload %T", m.Payload)
	}
	return out, nil
}

func nonNil(s []string) []string {
	if s == nil {
		return []string{}
	}
	return s
}

func versionResult(v tsp.Version) map[string]int {
	return map[string]int{"major": v.Major, "minor": v.Minor}
}

// ---------------------------------------------------------------------------
// pack / open / peek

func (d *driver) pack(raw []byte) (any, error) {
	var req struct {
		Scheme    string        `json:"scheme"`
		Sender    *identityJSON `json:"sender"`
		Receiver  *identityJSON `json:"receiver"`
		Payload   *payloadJSON  `json:"payload"`
		Ephemeral *struct {
			IkmE *string `json:"ikmE"`
			SkEm *string `json:"skEm"`
		} `json:"ephemeral"`
	}
	if err := json.Unmarshal(raw, &req); err != nil {
		return nil, invalid("pack request: %v", err)
	}
	scheme, err := parseScheme(req.Scheme)
	if err != nil {
		return nil, err
	}
	sender, err := req.Sender.private("sender")
	if err != nil {
		return nil, err
	}
	receiver, err := req.Receiver.public("receiver")
	if err != nil {
		return nil, err
	}
	payload, opts, err := buildPayload(req.Payload, scheme, sender)
	if err != nil {
		return nil, err
	}
	if e := req.Ephemeral; e != nil {
		switch {
		case scheme == tsp.SchemeSignedOnly:
			// Nothing is encrypted; there is no ephemeral key to pin.
		case scheme == tsp.SchemeHPKEPQ:
			return nil, fail("unsupported", "deterministic hpke-pq encapsulation")
		case scheme == tsp.SchemeHPKEBase && e.IkmE != nil:
			if opts.Ephemeral, err = b64d("ephemeral.ikmE", *e.IkmE); err != nil {
				return nil, err
			}
		case scheme == tsp.SchemeSealedBox && e.SkEm != nil:
			if opts.Ephemeral, err = b64d("ephemeral.skEm", *e.SkEm); err != nil {
				return nil, err
			}
		default:
			return nil, invalid("ephemeral does not carry the material %s uses (ikmE for hpke-base, skEm for sealed-box)", req.Scheme)
		}
	}
	packed, err := tsp.Pack(sender, receiver, payload, opts)
	if err != nil {
		return nil, err
	}
	res := map[string]any{"message": b64e(packed.Message), "digest": nil, "digestAlg": nil}
	if packed.Digest != nil {
		res["digest"], res["digestAlg"] = b64e(packed.Digest.Value[:]), digestAlgName(packed.Digest.Alg)
	}
	return res, nil
}

func (d *driver) open(raw []byte) (any, error) {
	var req struct {
		Receiver *identityJSON `json:"receiver"`
		Sender   *identityJSON `json:"sender"`
		Message  string        `json:"message"`
	}
	if err := json.Unmarshal(raw, &req); err != nil {
		return nil, invalid("open request: %v", err)
	}
	receiver, err := req.Receiver.private("receiver")
	if err != nil {
		return nil, err
	}
	sender, err := req.Sender.public("sender")
	if err != nil {
		return nil, err
	}
	msg, err := b64d("message", req.Message)
	if err != nil {
		return nil, err
	}
	m, err := tsp.Open(receiver, sender, msg)
	if err != nil {
		return nil, err
	}
	payload, err := payloadResult(m)
	if err != nil {
		return nil, err
	}
	return map[string]any{
		"version":          versionResult(m.Version),
		"envelopeSender":   m.Sender,
		"envelopeReceiver": m.Receiver,
		"scheme":           m.Scheme.String(),
		"payload":          payload,
	}, nil
}

func (d *driver) peek(raw []byte) (any, error) {
	var req struct {
		Message string `json:"message"`
	}
	if err := json.Unmarshal(raw, &req); err != nil {
		return nil, invalid("peek request: %v", err)
	}
	msg, err := b64d("message", req.Message)
	if err != nil {
		return nil, err
	}
	env, err := tsp.Peek(msg)
	if err != nil {
		return nil, err
	}
	return map[string]any{
		"version":          versionResult(env.Version),
		"envelopeSender":   env.Sender,
		"envelopeReceiver": env.Receiver,
		"confidential":     env.Confidential,
	}, nil
}

// ---------------------------------------------------------------------------
// endpoint.*

var ctx = context.Background()

func (d *driver) endpointCreate(raw []byte) (any, error) {
	var req struct {
		Identities []*identityJSON `json:"identities"`
		Peers      []*identityJSON `json:"peers"`
	}
	if err := json.Unmarshal(raw, &req); err != nil {
		return nil, invalid("endpoint.create request: %v", err)
	}
	resolver, _ := tsp.NewStaticResolver()
	for i, p := range req.Peers {
		id, err := p.public(fmt.Sprintf("peers[%d]", i))
		if err != nil {
			return nil, err
		}
		if err := resolver.Add(id); err != nil {
			return nil, err
		}
	}
	ep := tsp.NewEndpoint(resolver)
	for i, p := range req.Identities {
		id, err := p.private(fmt.Sprintf("identities[%d]", i))
		if err != nil {
			return nil, err
		}
		if err := ep.AddIdentity(id); err != nil {
			return nil, err
		}
	}
	d.next++
	handle := "ep" + strconv.Itoa(d.next)
	d.endpoints[handle] = ep
	return map[string]any{"endpoint": handle}, nil
}

func (d *driver) endpoint(handle string) (*tsp.Endpoint, error) {
	ep, ok := d.endpoints[handle]
	if !ok {
		return nil, invalid("unknown endpoint %q", handle)
	}
	return ep, nil
}

func digestStr(p *tsp.Digest) any {
	if p == nil {
		return nil
	}
	return b64e(p.Value[:])
}

func (d *driver) endpointSend(op string, raw []byte) (any, error) {
	var req struct {
		Endpoint string  `json:"endpoint"`
		From     string  `json:"from"`
		To       string  `json:"to"`
		Data     *string `json:"data"`
	}
	if err := json.Unmarshal(raw, &req); err != nil {
		return nil, invalid("%s request: %v", op, err)
	}
	ep, err := d.endpoint(req.Endpoint)
	if err != nil {
		return nil, err
	}
	switch op {
	case "endpoint.invite":
		p, err := ep.Invite(ctx, req.From, req.To)
		if err != nil {
			return nil, err
		}
		return map[string]any{"message": b64e(p.Message), "digest": digestStr(p.Digest)}, nil
	case "endpoint.accept":
		p, err := ep.Accept(ctx, req.From, req.To)
		if err != nil {
			return nil, err
		}
		r, err := ep.State(ctx, req.From, req.To)
		if err != nil {
			return nil, err
		}
		return map[string]any{"message": b64e(p.Message), "digest": digestStr(r.Digest), "replyDigest": digestStr(p.Digest)}, nil
	case "endpoint.cancel":
		p, err := ep.Cancel(ctx, req.From, req.To)
		if err != nil {
			return nil, err
		}
		return map[string]any{"message": b64e(p.Message)}, nil
	default: // endpoint.send
		if req.Data == nil {
			return nil, invalid("data is required")
		}
		data, err := b64d("data", *req.Data)
		if err != nil {
			return nil, err
		}
		p, err := ep.Send(ctx, req.From, req.To, data)
		if err != nil {
			return nil, err
		}
		return map[string]any{"message": b64e(p.Message)}, nil
	}
}

func (d *driver) endpointReceive(raw []byte) (any, error) {
	var req struct {
		Endpoint string `json:"endpoint"`
		Message  string `json:"message"`
	}
	if err := json.Unmarshal(raw, &req); err != nil {
		return nil, invalid("endpoint.receive request: %v", err)
	}
	ep, err := d.endpoint(req.Endpoint)
	if err != nil {
		return nil, err
	}
	msg, err := b64d("message", req.Message)
	if err != nil {
		return nil, err
	}
	ev, err := ep.Receive(ctx, msg)
	if err != nil {
		return nil, err
	}
	out := map[string]any{"from": ev.From, "to": ev.To}
	switch ev.Kind {
	case tsp.EventInvite:
		out["event"], out["digest"] = "invite", digestStr(ev.Digest)
	case tsp.EventAccept:
		out["event"], out["digest"], out["replyDigest"] = "accept", digestStr(ev.Digest), digestStr(ev.ReplyDigest)
	case tsp.EventCancel:
		out["event"], out["digest"] = "cancel", digestStr(ev.Digest)
	case tsp.EventMessage:
		out["event"], out["data"] = "message", b64e(ev.Data)
	default:
		return nil, fail("unsupported", "endpoint.receive of a %v message", ev.Kind)
	}
	return out, nil
}

func (d *driver) endpointState(raw []byte) (any, error) {
	var req struct {
		Endpoint string `json:"endpoint"`
		Local    string `json:"local"`
		Remote   string `json:"remote"`
	}
	if err := json.Unmarshal(raw, &req); err != nil {
		return nil, invalid("endpoint.state request: %v", err)
	}
	ep, err := d.endpoint(req.Endpoint)
	if err != nil {
		return nil, err
	}
	r, err := ep.State(ctx, req.Local, req.Remote)
	if err != nil {
		return nil, err
	}
	out := map[string]any{"state": r.State.String()}
	if r.Digest != nil {
		out["digest"] = digestStr(r.Digest)
	}
	if r.ReplyDigest != nil {
		out["replyDigest"] = digestStr(r.ReplyDigest)
	}
	return out, nil
}
