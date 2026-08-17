# 05 — Retain typed facts inside incomplete rules

**What to build:** Preserve useful syntax inside the rule currently being edited. Callers can query the left-hand side, alternatives, or partial non-terminal reference already present in a malformed rule, while missing required syntax is reported through optional typed accessors.

**Blocked by:** 04 — Recover broken rules without losing later rules.

**Status:** ready-for-agent

- [ ] Incomplete rule declarations retain any recognized left-hand-side name and type syntax.
- [ ] Incomplete right-hand sides retain recognized alternatives, symbols, and non-terminal reference prefixes where the source establishes them.
- [ ] Missing closing delimiters, definition syntax, symbols, repeat bounds, or rule terminators appear as absent typed children rather than fabricated values.
- [ ] Representative typing states include incomplete typed non-terminals, non-terminal references, literals, regular expressions, alternatives, and repeat ranges.
- [ ] Analysis-style callers can obtain recognized facts entirely through typed views and never need to reconstruct grammar structure from raw token patterns.
- [ ] Syntax errors and recovery ranges remain separately queryable from the recognized partial facts.
- [ ] Normalized fixtures demonstrate that adding the missing syntax turns the partial views into complete views without changing unrelated ranges.
- [ ] No semantic decoding, resolution, graph analysis, or generation validation enters the syntax crate.
