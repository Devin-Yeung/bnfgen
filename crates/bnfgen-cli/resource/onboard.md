---
uri: file:///bnfgen/onboard
name: Getting Started
---

## Overview

Bnfgen is a BNF grammar-based random string generator with extensions for fuzzing and grammar-based testing. It extends standard BNF with regex support, invoke limits, weighted alternatives, and typed non-terminals.

## Quick Start

Use the `generate` tool to create random strings from a grammar:

```
grammar: "<S> ::= \"hello\" ;"
start_symbol: "S"
count: 5
```

This returns 5 random strings matching the grammar.

## Writing Grammars

### Basic Structure

```bnf
<Start> ::= <Greeting> <Name> ;
<Greeting> ::= "hello" | "hi" ;
<Name> ::= "world" | "there" ;
```

Rules end with `;`. Alternatives are separated by `|`.

### Available Resources

- **Core Syntax** (`file:///bnfgen/syntax/core`): Base grammar structure — `<Grammar>`, `<Rule>`, `<Alts>`, `<Symbol>`, `<NonTerm>`
- **Regex** (`file:///bnfgen/syntax/regex`): Terminal-like tokens using patterns like `re("[a-zA-Z]+")`
- **Invoke Limits** (`file:///bnfgen/syntax/limit`): Bounded repetition with `{n}`, `{min,}`, `{min,max}`
- **Weighted Selection** (`file:///bnfgen/syntax/weight`): Probabilistic alternatives with `<num> <Symbols>`

## Example Grammars

### Email Address

```bnf
<Email> ::= <Local> "@" <Domain> ;
<Local> ::= <Word> | <Local> "." <Word> ;
<Domain> ::= <Word> "." <TLD> ;
<Word> ::= re("[a-zA-Z]+") ;
<TLD> ::= "com" | "org" | "net" ;
```

### JSON Fragment

```bnf
<Json> ::= "{" <Pairs> "}" | "[" <Values> "]" ;
<Pairs> ::= <Pair> | <Pairs> "," <Pair> ;
<Pair> ::= <Key> ":" <Value> ;
<Values> ::= <Value> | <Values> "," <Value> ;
<Key> ::= re("\"[a-z]+\"") ;
<Value> ::= "null" | "true" | "false" | re("[0-9]+") | <Json> ;
```