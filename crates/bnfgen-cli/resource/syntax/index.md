---
uri: bnfgen://syntax/index
name: Syntax Index
---

## Resource routing

- Read `bnfgen://syntax/core` for base structure (`<Grammar>`, `<Rule>`, `<Alts>`, `<Symbol>`, `<NonTerm>`).
- Read `bnfgen://syntax/regex` when terminal-like tokens are easier as patterns.
- Read `bnfgen://syntax/limit` when alternatives need bounded invocation (`{n}`, `{min,}`, `{min,max}`).
- Read `bnfgen://syntax/weight` when alternatives need weighted selection (`<num> <Symbols>`).