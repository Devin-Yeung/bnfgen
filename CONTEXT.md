# Bnfgen Grammar

Bnfgen describes grammars used to generate strings and parse trees. This glossary names concepts in the grammar language independently of their implementation.

## Language

**Start Symbol**:
The non-terminal selected as the root of one generation operation. Reachability and generation viability are evaluated relative to this symbol.
_Avoid_: Entry rule, root rule

**Unreachable Rule**:
A rule that cannot be reached transitively from a particular Start Symbol. The same rule may be reachable when a different Start Symbol is selected.
_Avoid_: Unused variable, unused rule
