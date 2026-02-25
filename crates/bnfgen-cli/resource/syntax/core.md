---
uri: file:///bnfgen/syntax/core
name: Core Syntax
---

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

## Valid examples

```bnf
<S> ::= "hello" ;
```

```bnf
<Start> ::= <Greeting> <Name> ;
<Greeting> ::= "hello" | "hi" ;
<Name> ::= "world" ;
```

## Invalid examples

```bnf
<S> := "hello" ;
```

Reason: `<Rule>` uses `::=` for definition.

```bnf
<S> ::= "hello"
```

Reason: each rule must end with `;`.

```bnf
<S> ::= ;
```

Reason: `<Alt>` requires `<Symbols>` and cannot be empty.

## Common mistakes

- Missing `;` at the end of a rule.
- Using `:=` instead of `::=`.
- Referencing non-terminals without angle brackets.
- Passing start symbol as `<S>` instead of `S` in tool input.

## Next resources

- Getting started guide: `file:///bnfgen/onboard`
- Regex symbol form: `file:///bnfgen/syntax/regex`
- Invoke limits on alternatives: `file:///bnfgen/syntax/limit`
- Alternative weights: `file:///bnfgen/syntax/weight`
