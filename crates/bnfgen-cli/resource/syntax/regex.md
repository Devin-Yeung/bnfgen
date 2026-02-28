---
uri: file:///bnfgen/syntax/regex
name: Regex Syntax
---

## Regex grammar

The regex syntax allows you to define a symbol using a regular expression pattern. This is the PREFERRED way to define
tokens like identifiers or numbers without having to list out all possible combinations as fixed literals.

For example, instead of writing:

```bnf
<Id> ::= "a" | "b" | "c" | ... | "x" | "y" | "z" | "aa" | "ab" | ... ;
```

You SHOULD write:

```bnf
<Id> ::= re("[a-z]+") ;
```

Some useful regex patterns for common token types:

```bnf
<Id> ::= re("[a-z]+") ;
<Num> ::= re("[0-9]|[1-9][0-9]+") ;
```

## Common mistakes

The most common mistakes when using regex in BNF are:

- Forgetting to put the regex pattern in a string.
- Forgetting `(` or `)` around the pattern.
- Using regex for fixed literals that are simpler as `<str>`.

```bnf
<Id> ::= re([a-z]+) ;
```

Reason: the regex pattern must be a `<str>`.

```bnf
<Id> ::= re("[a-z]+" ;
```

Reason: `re` must follow `"re" "(" <str> ")"`.
