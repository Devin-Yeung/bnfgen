# 02 — Preserve complete lexical input

**What to build:** Make every source character observable through the token-backed document. Callers can inspect significant tokens, whitespace, comments, invalid input, and recoverable unterminated forms in source order, with raw text derived from the retained source rather than copied token payloads.

**Blocked by:** 01 — Parse one complete rule through `bnfgen-syntax`.

**Status:** ready-for-agent

- [x] Token iteration retains significant tokens, whitespace, comments, invalid input, and representative unterminated forms in source order.
- [x] Token kinds and UTF-8 byte ranges are public, while raw token text is obtained by slicing the document source.
- [x] Integer, string, type, and regular-expression tokenization preserves raw lexemes instead of decoding or validating their values.
- [x] Lexical failures are retained as tokens or equivalent recoverable syntax input and never terminate document construction.
- [x] Comments at end of file and unterminated strings have explicit public-seam fixtures.
- [x] Token lookup can locate tokens at representative byte offsets without exposing internal token identifiers.
- [x] Public token ranges are ordered, in bounds, and safe for source slicing.
- [x] Normalized public snapshots replace reliance on the legacy lexer's private `Debug` layout.
