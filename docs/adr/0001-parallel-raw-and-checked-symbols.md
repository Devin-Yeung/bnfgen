# Parallel Raw and Checked symbol types

Raw Grammar and Checked Grammar use distinct symbol/production types (`Symbol Kind` + `Regex Lit` vs `Checked Symbol Kind` + `Regex Engine` + `Checked Production`). `to_checked` is an explicit compile from one to the other (including recompiling regex HIR), so analysis never sees generation engines and generation never carries source spans or parse-only payloads.

**Considered options:** (1) one generic production/symbol parameterized by stage — awkward with spanned AST vs spanless checked forms; (2) mutate AST alternatives in place during `to_checked` — leaves a hybrid “maybe compiled” invariant; (3) side table of compiled regexes keyed by pattern — extra cross-module lookup invariant. Rejected in favor of parallel types for a clear seam ahead of a future analysis/LSP split.
