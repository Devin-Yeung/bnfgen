# 03 — Expose complete grammar syntax through typed views

**What to build:** Let callers inspect every currently valid Bnfgen grammar construct through immutable language-specific views. Complete documents expose rules, alternatives, symbols, typed and untyped non-terminals, weights, repeat ranges, terminals, and regular-expression syntax without importing generation models or semantic validation.

**Blocked by:** 02 — Preserve complete lexical input.

**Status:** ready-for-agent

- [x] All valid constructs accepted by the current grammar are represented through document-borrowing typed syntax views.
- [x] Rule views expose left-hand sides and alternatives; alternative views expose weights, symbols, and repeat syntax; symbol views distinguish terminal, non-terminal, and regular-expression forms.
- [x] Typed and untyped non-terminal declarations and references preserve their raw spelling and precise source ranges.
- [x] String, integer, type, and regular-expression values remain raw syntax and are not decoded, parsed, compiled, resolved, or validated.
- [x] A significant-token adapter lets LALRPOP recognize structure without removing trivia from the complete token buffer.
- [x] Representative examples produce normalized public syntax snapshots covering every valid grammar form.
- [x] Private storage records, token identifiers, and LALRPOP values do not cross the crate interface.
- [x] Existing valid grammar examples parse without syntax errors through the new public seam.
