# 04 — Recover broken rules without losing later rules

**What to build:** Make parser failure local. When one rule is malformed, the document retains structured syntax errors and recovered source while continuing far enough to recognize later valid rules.

**Blocked by:** 03 — Expose complete grammar syntax through typed views.

**Status:** ready-for-agent

- [ ] Missing delimiters, missing right-hand sides, unexpected significant tokens, and lexical error tokens produce a `ParsedDocument` with structured syntax errors rather than a failed parse.
- [ ] A malformed rule before a valid rule does not prevent the later rule from appearing through the public typed views.
- [ ] Recovered or skipped input remains represented by the complete token buffer and queryable recovery ranges.
- [ ] Syntax error kinds and ranges are stable public data and do not require callers to parse rendered messages.
- [ ] Recovery never invents a complete generation object for malformed syntax.
- [ ] Fixtures prove recovery near rule terminators and at the start of a following rule without asserting on private synchronization mechanics.
- [ ] Public snapshots show recognized rules, recovery regions, and syntax errors in a normalized form.
- [ ] The public parse operation remains total and non-panicking for every recovery fixture.
