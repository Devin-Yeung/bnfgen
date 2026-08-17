# 07 — Harden total parsing across real typing states

**What to build:** Demonstrate that the public document contract survives realistic editing, malformed input, and UTF-8 text. Prefix and mutation coverage drives any required recovery fixes until parsing is consistently total and every exposed range remains safe.

**Blocked by:** 06 — Classify cursor syntax context.

**Status:** ready-for-agent

- [ ] Every prefix of representative valid grammar examples parses without panicking and returns a `ParsedDocument`.
- [ ] Mutations cover removed, duplicated, and replaced delimiters; altered strings and comments; invalid characters; alternatives; and repeat ranges.
- [ ] Multibyte UTF-8 fixtures prove that every exposed token, syntax, error, and recovery range is ordered, in bounds, and safe for source slicing.
- [ ] Later valid rules remain recognizable across the mutation corpus whenever recovery can establish a new rule boundary.
- [ ] Token lookup and cursor-context queries remain total for offsets from the start of source through end of file.
- [ ] Normalized public snapshots describe only caller-visible tokens, syntax views, errors, recovery regions, and contexts.
- [ ] Tests do not assert on Logos callbacks, LALRPOP productions, private records, or internal token identities.
- [ ] Full-document parsing completes acceptably across the repository's current example corpus without adding incremental parsing machinery.
