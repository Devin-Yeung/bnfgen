---
uri: file:///bnfgen/syntax/weight
name: Weight Syntax
---

## Weight grammar

```bnf
<Alt> ::= <Symbols>
        | <num> <Symbols>
        | <Symbols> <Limit>
        | <num> <Symbols> <Limit> ;
```

Weighted form in `<Alt>` is the `<num> <Symbols>` prefix.

## Valid examples

```bnf
<Bit> ::= 9 "0" | 1 "1" ;
```

```bnf
<S> ::= 5 <A> {2,4} | 1 <B> ;
```

## Invalid examples

```bnf
<S> ::= <A> 5 ;
```

Reason: `<num>` must appear before `<Symbols>`.

```bnf
<S> ::= 2 ;
```

Reason: `<num>` must be followed by `<Symbols>`.

## Common mistakes

- Treating weight as a rule-level value instead of an alternative-level prefix.
- Writing weight after symbols instead of before symbols.
- Confusing weight (`<num> <Symbols>`) with invoke limits (`<Limit>`).

## Next resources

- Routing guide: `file:///bnfgen/syntax/index`
- Core structure: `file:///bnfgen/syntax/core`
- Invoke limits on alternatives: `file:///bnfgen/syntax/limit`
