<img src="https://github.com/bnkc/unpack/blob/main/logo.svg" width="250" align="right">

# Unpack

[![CI](https://github.com/bnkc/unpack/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bnkc/unpack/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io version shield](https://img.shields.io/crates/v/logos.svg)]([https://crates.io/crates/logos](https://crates.io/crates/un-pack))
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
❯ unpack

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
For more documentation, please refer to the unpack [crate documentation](https://crates.io/crates/un-pack)

## Installation

### On macOS

You can install `unpack` with Homebrew
```
brew tap bnkc/tap
brew install unpack
```

… or with MacPorts:
```
sudo port install [COMING SOON]
```


### On crates.io

```
cargo install un-pack --locked
```
Alternatively, you can install `unpack` directly from the repository by using:
```
git clone https://github.com/bnkc/unpack
cargo install --path ./unpack
```

> [!WARNING]
> There are scenarios where using `Unpack` can yield false positives. Mapping `site-packages` to their corresponding
> dependencies/imports are not always a 1:1 relationship. For Example: `scikit-learn` is imported as `sklearn`.
> Alot of decisions were made based on [Metadata for Python Software Packages](https://packaging.python.org/en/latest/specifications/core-metadata/#core-metadata)

### Command-line options

This is the output of `un-pack -h`. To see the full set of command-line options, use `un-pack --help` which
also includes a much more detailed help text.

```
Usage: un-pack [OPTIONS]

Options:
  -b, --base-directory <BASE_DIRECTORY>
          The path to the directory to search for Python files. [default: .]
  -s, --package-status <STATUS>
          Select the packages status to search for [default: unused] [possible values: used,
          unused, untracked]
  -i, --ignore-hidden
          Ignore hidden files and directories.
  -d, --max-depth <DEPTH>
          Set maximum search depth (default: none)
  -o, --output <OUTPUT>
          The output format to use allows for the selection of the output format for the results
          of the unused packages search. The default output format is `human`. The `json` format
          is also available [default: human] [possible values: human, json]
  -t, --dep-type <DEP_TYPE>
          Select the depencency specification file of choice if more than one exists. By default,
          `pyproject.toml` is selected [default: poetry] [possible values: pip, poetry]
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

## Thank you

**Unpack** is very much a labor of love. There's alot of improvements
that could be done, but if you find this project useful, PM me!

If you'd like to contribute to Unpack, please open an issue/PR ❤️
