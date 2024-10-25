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

## Acknowledgement

- Born of this project is __highly inspired__ by _Daniil Baturin_'s [work](https://github.com/dmbaturin/bnfgen),
which is also a BNF based random string generator, but in OCaml 🐫.
The design of the grammar extension is also heavily influenced by his work.

- I want to thank _Andrew Gallant_ for his [work](https://github.com/rust-lang/regex) on the regex crate in Rust Eco-system.
The incorporation of regular language can't be done so easily without the help of this crate.

- I'd also like to express my gratitude to _Maciej Hirsz_ and _Niko Matsakis_,
their excellent work on the _[logos](https://github.com/maciejhirsz/logos)_ 
and _[lalrpop](https://github.com/maciejhirsz/logos)_ crate makes the parsing of the grammar file a breeze 🍃.