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

Thanks!