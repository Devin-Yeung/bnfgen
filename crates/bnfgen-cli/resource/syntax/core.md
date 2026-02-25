---
uri: bnfgen://syntax/core
name: Core Syntax
---

## When to read this

Use this resource first. It defines the structural backbone of the DSL: grammar, rules, alternatives, symbols, terminals, and non-terminals.

## Core grammar

```bnf
<Grammar> ::= <Rule> | <Grammar> <Rule>     ;
<Rule>    ::= "<" <id> ">" "::=" <Alts> ";" ;
<Alts>    ::= <Alt> | <Alts> <Alt>          ;
<Alt>     ::= <Symbols>                     ;
<Symbols> ::= <Symbol> | <Symbols> <Symbol> ;
<Symbol>  ::= <str> | <NonTerm>             ;
<NonTerm> ::= "<" <id> ">"                  ;
```