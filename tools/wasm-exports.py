#!/usr/bin/env python3
"""Minimal WASM export-section reader (no deps). Proves required ABI exports."""
import sys, struct

def leb_u(buf, i):
    r = 0; s = 0
    while True:
        b = buf[i]; i += 1
        r |= (b & 0x7F) << s
        if not (b & 0x80):
            return r, i
        s += 7

def main(path):
    buf = open(path, 'rb').read()
    assert buf[:4] == b'\x00asm', "not a wasm module (bad magic)"
    ver = struct.unpack('<I', buf[4:8])[0]
    i = 8
    kinds = {0: 'func', 1: 'table', 2: 'memory', 3: 'global'}
    exports = []
    sections = []
    while i < len(buf):
        sid = buf[i]; i += 1
        size, i = leb_u(buf, i)
        sections.append((sid, size))
        body = buf[i:i + size]
        if sid == 7:
            j = 0
            n, j = leb_u(body, j)
            for _ in range(n):
                ln, j = leb_u(body, j)
                name = body[j:j + ln].decode('utf-8', 'replace'); j += ln
                k = body[j]; j += 1
                _idx, j = leb_u(body, j)
                exports.append((name, kinds.get(k, f'kind{k}')))
        i += size
    print(f"file: {path}")
    print(f"bytes: {len(buf)}")
    print(f"wasm version: {ver}")
    print(f"section ids: {','.join(str(s[0]) for s in sections)}")
    print(f"exports ({len(exports)}):")
    for n, k in sorted(exports):
        print(f"  {n}  [{k}]")

    # prompt.md shorthand "(meridian_abi=1, alloc, run)" == the three
    # meridian_-prefixed no_mangle fns in src/lib.rs.
    required = {'meridian_abi': 'func', 'meridian_alloc': 'func', 'meridian_run': 'func'}
    print("--- required check ---")
    ok = True
    ed = dict(exports)
    for name, kind in required.items():
        if name not in ed:
            print(f"MISSING: {name}")
            ok = False
        elif ed[name] != kind:
            print(f"WRONG KIND: {name} is {ed[name]}, want {kind}")
            ok = False
        else:
            print(f"present: {name} [{kind}]")
    print("RESULT:", "ALL_REQUIRED_EXPORTS_PRESENT" if ok else "MISSING_EXPORTS")
    return 0 if ok else 1

if __name__ == '__main__':
    sys.exit(main(sys.argv[1]))
