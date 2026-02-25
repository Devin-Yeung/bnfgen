---
uri: bnfgen://syntax/regex
name: Regex Syntax
---

## Regex grammar

```bnf
<Symbol> ::= <str> | <NonTerm> | <Regex> ;
<Regex>  ::= "re" "(" <str> ")"   ;
```

## Valid examples

```bnf
<Id> ::= re("[a-z]+") ;
```

```bnf
<Num> ::= re("[0-9]|[1-9][0-9]+") ;
```

## Invalid examples (with reasons)

```bnf
<Id> ::= re([a-z]+) ;
```

Reason: the regex pattern must be a `<str>`.

```bnf
<Id> ::= re("[a-z]+" ;
```

Reason: `re` must follow `"re" "(" <str> ")"`.

## Common mistakes

- Forgetting to put the regex pattern in a string.
- Forgetting `(` or `)` around the pattern.
- Using regex for fixed literals that are simpler as `<str>`.

## Next resources

- Routing guide: `bnfgen://syntax/index`
- Core structure: `bnfgen://syntax/core`
- Invoke limits on alternatives: `bnfgen://syntax/limit`
- Alternative weights: `bnfgen://syntax/weight`
