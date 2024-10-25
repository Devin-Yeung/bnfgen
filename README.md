# 📚 Bnfgen

Bnfgen is a general purposed BNF grammar based random string generator ⚙️,
with a powerful grammar extension to make it more 🪑 ergonomic to use.

## Pitfall of BNF based generator

- Generation of complex `token` is hard to maintain, e.g. numbers, variable

```text
<letter> ::= "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" 
           | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" 
           | "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" 
           | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" ;
```

This is a sample of how to represent a letter in BNF, it is obvious that the maintainability of this syntax is quite
low.
In our grammar extension, we incorporate the regular language to make it more maintainable.
Under our extension, the above syntax can simply be written as:

```text
<letter> ::= re("[a-zA-Z]")
```