#!/usr/bin/env python3
"""Extract TSP Rev 3 Appendix A test vectors from the specification markdown.

Usage: extract_spec_vectors.py <path/to/spec.md> <spec-commit> > fixtures/spec-vectors.json

Each identifier or vector in Appendix A is a `name` (backticked, or a #### heading)
followed by a ``` text block of `key value` lines, where a key with no value is
continued on the following indented lines.
"""
import json, re, sys

src, commit = sys.argv[1], sys.argv[2]
lines = open(src).read().splitlines()
start = next(i for i, l in enumerate(lines) if l.startswith("## Appendix A"))
lines = lines[start:]

def parse_block(block):
    out, key = {}, None
    for l in block:
        if l.startswith("  ") and key:
            out[key] += l.strip()
        elif l.strip() == "":
            key = None
        else:
            parts = l.split(None, 1)
            key = parts[0]
            out[key] = parts[1].strip() if len(parts) > 1 else ""
    return out

identifiers, vectors, name, section = {}, {}, None, None
i = 0
while i < len(lines):
    l = lines[i]
    if l.startswith("### ") or l.startswith("#### "):
        title = l.lstrip("#").strip()
        section = title
        name = None if title == "Identifiers" else title
    m = re.fullmatch(r"`([a-z_0-9]+)`", l.strip())
    if m:
        name = m.group(1)
    if l.strip().startswith("``` text") or l.strip() == "```text":
        j = i + 1
        while not lines[j].startswith("```"):
            j += 1
        block = parse_block(lines[i + 1 : j])
        if "id" in block and "pkS" in block:
            identifiers[name] = block
        elif "message" in block:
            vectors[name] = block
        i = j
    i += 1

json.dump({
    "_source": {
        "spec": "trustoverip/tswg-tsp-specification",
        "commit": commit,
        "section": "Appendix A: Test Vectors",
        "note": "Extracted mechanically by tools/extract_spec_vectors.py. Values are qb64 (CESR text domain). Private keys are published by the spec and must never be used for anything but checking these vectors.",
    },
    "identifiers": identifiers,
    "vectors": vectors,
}, sys.stdout, indent=2)
print()
