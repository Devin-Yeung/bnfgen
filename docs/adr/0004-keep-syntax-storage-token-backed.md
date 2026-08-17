# Keep syntax storage token-backed

`bnfgen-syntax` will retain the original source, a complete token buffer, and private records for recognized syntax and recovery. It will not require a generic concrete syntax tree or Rowan storage for V1: Rowan does not provide parser recovery, and materializing its tree from LALRPOP would add tree plans, trivia reinsertion, and another traversal without improving the language-specific interface.

## Consequences

Callers use immutable language-specific views, token lookup, source ranges, syntax errors, and cursor-context queries rather than generic parent/child traversal. Internal records may change without crossing the crate interface, and `ParsedDocument` remains `Send + Sync`. Rowan can be reconsidered if formatting, structural editing, incremental subtree reuse, or multiple consumers of generic syntax traversal create evidence for that machinery. This storage decision does not weaken the separate `bnfgen-syntax` crate seam.
