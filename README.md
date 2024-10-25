# 📚 Bnfgen

[![Build Status][actions-badge]][actions-url]
[![MIT Licensed][mit-badge]][mit-url]

[actions-badge]: https://github.com/Devin-Yeung/bnfgen/actions/workflows/ci.yml/badge.svg?branch=master

[actions-url]: https://github.com/Devin-Yeung/bnfgen/actions/workflows/ci.yml

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg

[mit-url]: https://github.com/Devin-Yeung/bnfgen/blob/master/LICENSE-MIT

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
To address this, we integrate the regular express into our grammar extension seamlessly.
Under our extension, the above syntax can simply be written as:

```text
<letter> ::= re("[a-zA-Z]")
```

## Beyond the generation

The design of Bnfgen is heavily influenced by the Rust Programming Language, particularly its outstanding error messages.
Bnfgen conducts various semantic analyses on the BNF grammar, aiming to make its error messages as informative as possible.

```text
  × May be trapped in a dead loop
   ╭─[5:13]
 4 │        <term> ::= "Terminal" ;
 5 │        <A> ::= <B> ;
   ·        ──────┬──────
   ·        ╰── this rule may be trapped in a dead loop
 6 │        <B> ::= <C> ;
   ·        ──────┬──────
   ·        ╰── this rule may be trapped in a dead loop
 7 │        <C> ::= <A> ;
   ·        ──────┬──────
   ·        ╰── this rule may be trapped in a dead loop
 8 │
   ╰────
```

Currently, we support the following semantic analysis:

- [x] Invalid invoke limit range detection
- [x] Undefined rule detection
- [x] Duplicated rule detection
- [x] Unreachable rule detection
- [x] Dead loop detection (which avoid the possible infinite loop in the generation)

We believe that an informative error message is the key to make the tool more __ergonomic__ to use.

## Acknowledgement

- Born of this project is __highly inspired__ by _Daniil Baturin_'s [work](https://github.com/dmbaturin/bnfgen),
  which is also a BNF based random string generator, but in OCaml 🐫.
  The design of the grammar extension is also heavily influenced by his work.

- I want to thank _Andrew Gallant_ for his [work](https://github.com/rust-lang/regex) on the regex crate in Rust
  Eco-system.
  The incorporation of regular language can't be done so easily without the help of this crate.

- I'd also like to express my gratitude to _Maciej Hirsz_ and _Niko Matsakis_,
  their excellent work on the _[logos](https://github.com/maciejhirsz/logos)_
  and _[lalrpop](https://github.com/maciejhirsz/logos)_ crate makes the parsing of the grammar file a breeze 🍃.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.