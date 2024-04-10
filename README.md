<img src="https://github.com/bnkc/unpack/blob/main/logo.svg" alt="Logos logo" width="250" align="right">

# Unpack

[![CI](https://github.com/bnkc/unpack/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bnkc/unpack/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
<!-- [![Crates.io version shield](https://img.shields.io/crates/v/logos.svg)](https://crates.io/crates/logos) -->
<!-- [![Docs](https://docs.rs/logos/badge.svg)](https://docs.rs/logos) -->

_Unpack python packages from your project and more._

**Unpack** has a few goals:

+ To easily navigate and remove used, unused, and untracked python packages.
+ To quickly identify disk usage of packages in the above categories. 
+ To view the relationship between various packages and their dependencies. 

To achieve those, **Unpack**:

+ Collects all project imports by walking the [abstract syntax tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree).
+ Collects all declared dependencies from the [dependency specification file](https://peps.python.org/pep-0508/).
+ Maps local environment [site-packages](https://ffy00.github.io/blog/02-python-debian-and-the-install-locations/) to resolve dependencies and the        imports they expose.
+ Identifies local site-package dependencies as to not accidently remove actively used dependencies of other packages.
+ Calculates package(s) size, and total disk usage.

### Package States

+ `-used` is when the package is locally installed, one of it's aliases is actively used in the project, and a corresponding dependency is declared in `pyproject.toml` or `requirements.txt`. This state indicates a fully integrated and properly managed package.

+ `-unused` is when the package is locally installed, and a corresponding dependency is declared in `pyproject.toml` or `requirements.txt`, but is not actively used in the project. **Caveat:** This package must not be a dependency of any actively `-used` package to be considered unused.

+ `-untracked` is when the package is installed, and one of it's aliases is actively used in the project, but is not declared in `pyproject.toml` or `requirements.txt`. This highlights packages that are implicitly used but not formally declared, which may lead to inconsistencies or issues in dependency management and deployment.



## Usage

```
❯ un-pack

 📦 Unused Packages

 package      | version      | size     
--------------+--------------+----------
 scikit-learn | ^1.4.1.post1 | 33.2 MiB 
 pydantic     | ^1.9.0       | 7.2 MiB  
 keras        | ^3.0.5       | 3.9 MiB  

 💽 Total disk space: 44.3 MiB

 Note: There might be false-positives.
       For example, Unpack cannot detect usage of packages that are not imported under `[tool.poetry.*]`.
       Similarly, it can only detect declared packages in requirements.txt or pyproject.toml.
```

## Installation

### On macOS

You can install `fd` with [Homebrew](https://formulae.brew.sh/formula/fd):
```
brew install ...
```

… or with MacPorts:
```
sudo port install ..
```



### On crates.io

```
cargo install un-pack --locked
```




<!-- 
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
