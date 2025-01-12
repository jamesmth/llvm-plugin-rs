# Contributing to llvm-plugin-rs

Thank you for taking interest in `llvm-plugin-rs`!

## Found a Bug?

* Please ensure the bug was not already reported by searching the [issue tracker](https://github.com/jamesmth/llvm-plugin-rs/issues).
* If you're unable to find an open issue relating to the problem, please file an [issue](https://github.com/jamesmth/llvm-plugin-rs/issues/new).

## Want to Submit a Pull Request?

Please ensure your PR follows these guidelines:

### Lint & Formatting

* You used the `rustfmt` coding style for any newly added **Rust** code
* You used the [LLVM coding style] for any newly added **C++** code
* You have ran `clippy` and updated portions of **Rust** code pertaining to your changes
* You have ran [`clang-tidy`] and updated portions of **C++** code pertaining to your changes

[LLVM coding style]: ../llvm-plugin/cpp/.clang-format
[`clang-tidy`]: ../llvm-plugin/cpp/.clang-tidy

### Documentation & Tests

* You added documentation and possibly doc tests to any new functions or types
* You have updated documentation and doc tests to any modified functions or types as applicable
* You have added tests to cover your changes
* All new and existing tests passed

### Git Practices

* You are basing your changes off master
* You will keep your code reasonably up to date via rebasing over merging whenever possible

### Use conventional commits

We use [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) and check for them as
a lint build step. To help adhere to the format, we recommend to install
[Commitizen](https://commitizen-tools.github.io/commitizen/). By using this tool you automatically
follow the configuration defined in [.cz.toml](../.cz.toml). Your commit messages should have enough
information to help someone reading the [CHANGELOG](../CHANGELOG.md) understand what is new just from
the title. The summary helps expand on that to provide information that helps provide more context,
describes the nature of the problem that the commit is solving and any unintuitive effects of the
change. It's rare that code changes can easily communicate intent, so make sure this is clearly
documented.

Thanks!