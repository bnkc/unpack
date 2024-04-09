<img src="https://github.com/bnkc/pyprune/blob/main/prune.svg" alt="Logos logo" width="250" align="right">

# PyPrune

[![CI](https://github.com/bnkc/pyprune/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bnkc/pyprune/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
<!-- [![Crates.io version shield](https://img.shields.io/crates/v/logos.svg)](https://crates.io/crates/logos) -->
<!-- [![Docs](https://docs.rs/logos/badge.svg)](https://docs.rs/logos) -->

_Prune unused packages from your python project and more._

**PyPrune** has a few goals:

+ To easily navigate and remove used, unused, and untracked python packages.
+ To quickly identify disk usage of packages in the above categories. 
+ To view the relationship between various packages and their dependencies. 

To achieve those, **PyPrune**:

+ Collects all project imports by walking the [abstract syntax tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree).
+ Collects all declared dependencies from the [dependency specification file](https://peps.python.org/pep-0508/).
+ Maps local environment [site-packages](https://ffy00.github.io/blog/02-python-debian-and-the-install-locations/) to resolve dependencies and the        imports they expose.
+ Identifies local site package dependencies as to not accidently remove actively used dependencies of other packages.
+ Calculates package size, and total disk space.


<!-- ## Example

```rust
 use logos::Logos;

 #[derive(Logos, Debug, PartialEq)]
 #[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
 enum Token {
     // Tokens can be literal strings, of any length.
     #[token("fast")]
     Fast,

     #[token(".")]
     Period,

     // Or regular expressions.
     #[regex("[a-zA-Z]+")]
     Text,
 }

 fn main() {
     let mut lex = Token::lexer("Create ridiculously fast Lexers.");

     assert_eq!(lex.next(), Some(Ok(Token::Text)));
     assert_eq!(lex.span(), 0..6);
     assert_eq!(lex.slice(), "Create");

     assert_eq!(lex.next(), Some(Ok(Token::Text)));
     assert_eq!(lex.span(), 7..19);
     assert_eq!(lex.slice(), "ridiculously");

     assert_eq!(lex.next(), Some(Ok(Token::Fast)));
     assert_eq!(lex.span(), 20..24);
     assert_eq!(lex.slice(), "fast");

     assert_eq!(lex.next(), Some(Ok(Token::Text)));
     assert_eq!(lex.slice(), "Lexers");
     assert_eq!(lex.span(), 25..31);

     assert_eq!(lex.next(), Some(Ok(Token::Period)));
     assert_eq!(lex.span(), 31..32);
     assert_eq!(lex.slice(), ".");

     assert_eq!(lex.next(), None);
 }
```

For more examples and documentation, please refer to the
[Logos handbook](https://maciejhirsz.github.io/logos/) or the
[crate documentation](https://docs.rs/logos/latest/logos/).

## How fast?

Ridiculously fast!

```norust
test identifiers                       ... bench:         647 ns/iter (+/- 27) = 1204 MB/s
test keywords_operators_and_punctators ... bench:       2,054 ns/iter (+/- 78) = 1037 MB/s
test strings                           ... bench:         553 ns/iter (+/- 34) = 1575 MB/s
```

## Acknowledgements

+ [Pedrors](https://pedrors.pt/) for the **Logos** logo.

## Thank you

**Logos** is very much a labor of love. If you find it useful, consider
[getting me some coffee](https://github.com/sponsors/maciejhirsz). ☕

If you'd like to contribute to Logos, then consider reading the
[Contributing guide](https://maciejhirsz.github.io/logos/contributing).

## License

This code is distributed under the terms of both the MIT license
and the Apache License (Version 2.0), choose whatever works for you.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details. -->