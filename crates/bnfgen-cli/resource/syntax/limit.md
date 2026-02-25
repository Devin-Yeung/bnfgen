---
uri: bnfgen://syntax/limit
name: Invoke Limits Syntax
---

## Invoke limit grammar

```bnf
<Alt>   ::= <Symbols>
          | <num> <Symbols>
          | <Symbols> <Limit>
          | <num> <Symbols> <Limit> ;

<Limit> ::= "{" <num> "}"
          | "{" <num> "," "}"
          | "{" <num> "," <num> "}" ;
```

## How to read each form

- `{n}`: exactly `n` selections.
- `{min,}`: at least `min` selections.
- `{min,max}`: between `min` and `max` selections (inclusive).

## Valid examples

```bnf
<S> ::= <A> {3} ;
```

```bnf
<S> ::= <A> {1,} | <B> {0,2} ;
```

```bnf
<S> ::= 5 <A> {2,4} | 1 <B> ;
```

## Invalid examples (with reasons)

```bnf
<S> ::= <A> {,5} ;
```

Reason: `<Limit>` requires a leading `<num>`.

```bnf
<S> ::= <A> {5,1} ;
```

Reason: this shape matches syntax, but `min > max` is invalid.

## Common mistakes

- Treating a limit as a rule-level attribute instead of an alternative-level attribute.
- Using `{min,max}` with `min > max`.
- Assuming weights remove limit constraints (limits are still enforced).

## Next resources

- Routing guide: `bnfgen://syntax/index`
- Core structure: `bnfgen://syntax/core`
- Regex symbol form: `bnfgen://syntax/regex`
- Alternative weights: `bnfgen://syntax/weight`
