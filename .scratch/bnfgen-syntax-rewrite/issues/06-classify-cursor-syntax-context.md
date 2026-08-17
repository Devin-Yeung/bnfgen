# 06 — Classify cursor syntax context

**What to build:** Give editor-facing callers one language-specific query that explains what kind of syntax surrounds a byte offset. Completion and navigation code can distinguish declaration, reference, literal, repeat, whitespace, and recovered contexts without walking private parser records or interpreting punctuation.

**Blocked by:** 05 — Retain typed facts inside incomplete rules.

**Status:** ready-for-agent

- [ ] The document classifies offsets in rule declarations, left-hand-side names and types, definition syntax, right-hand-side symbols, non-terminal reference names and types, literals, regular expressions, and repeat ranges.
- [ ] The query classifies inter-rule whitespace, comments, end of file, invalid tokens, and recovered regions.
- [ ] Cursor context remains useful for incomplete references and other representative mid-typing states.
- [ ] Token lookup and syntax-context classification agree on source ranges at representative token interiors and boundaries.
- [ ] Callers do not need generic parent traversal, raw token pattern matching, or access to private storage identifiers.
- [ ] Cursor-context fixtures cover every supported context through the public parse-and-query seam.
- [ ] Context results remain language-specific and contain no LSP position or protocol types.
