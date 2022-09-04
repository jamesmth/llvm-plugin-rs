llvm-plugin-rs 
==============

[<img alt="version" src="https://img.shields.io/crates/v/llvm-plugin.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/llvm-plugin)
[<img alt="doc" src="https://img.shields.io/badge/docs.rs-llvm--plugin-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/llvm-plugin)
[<img alt="linux" src="https://img.shields.io/github/workflow/status/jamesmth/llvm-plugin-rs/Linux%20Test%20Suite/develop?label=linux&style=for-the-badge&logo=linux" height="20">](https://github.com/jamesmth/llvm-plugin-rs/actions/workflows/linux.yml?query=branch%3Adevelop)
[<img alt="windows" src="https://img.shields.io/github/workflow/status/jamesmth/llvm-plugin-rs/Windows%20Test%20Suite/develop?label=windows&style=for-the-badge&logo=windows" height="20">](https://github.com/jamesmth/llvm-plugin-rs/actions/workflows/windows.yml?query=branch%3Adevelop)
[<img alt="macos" src="https://img.shields.io/github/workflow/status/jamesmth/llvm-plugin-rs/MacOS%20Test%20Suite/develop?label=macos&style=for-the-badge&logo=apple" height="20">](https://github.com/jamesmth/llvm-plugin-rs/actions/workflows/macos.yml?query=branch%3Adevelop)

This crate gives the ability to safely implement passes for the [new LLVM pass manager], by leveraging the strongly typed interface
provided by [Inkwell].

If you have never developed LLVM passes before, perhaps you should take a look at this [LLVM guide] before carrying on. It will
give you a simple overview of the C++ API wrapped by this crate.

If you want a deeper understanding of the many concepts surrounding the new LLVM pass manager, you should read the [official LLVM
documentation].

[Inkwell]: https://github.com/TheDan64/inkwell
[new LLVM pass manager]: https://blog.llvm.org/posts/2021-03-26-the-new-pass-manager/
[LLVM guide]: https://llvm.org/docs/WritingAnLLVMNewPMPass.html
[official LLVM documentation]: https://llvm.org/docs/NewPassManager.html

## Usage

Out-of-tree LLVM passes are plugins implemented as dynamically-linked libraries loaded by the [opt] tool. Therefore,
you must add the following line in your `Cargo.toml`:

[opt]: https://releases.llvm.org/14.0.0/docs/CommandGuide/opt.html

```toml
[lib]
crate-type = ["cdylib"]
```

When importing this crate in your `Cargo.toml`, you will need to specify the LLVM version to use with a corresponding feature flag:

```toml
[dependencies]
llvm-plugin = { version = "0.1", features = ["llvm10-0"] }
```

Supported versions:

| LLVM Version | Cargo Feature Flag |    Linux    |   Windows   |    MacOS    |
| :----------: | :----------------: | :---------: | :---------: | :---------: |
|    10.0.x    |      llvm10-0      | **&check;** | **&cross;** | **&check;** |
|    11.0.x    |      llvm11-0      | **&check;** | **&check;** | **&check;** |
|    12.0.x    |      llvm12-0      | **&check;** | **&check;** | **&check;** |
|    13.0.x    |      llvm13-0      | **&check;** | **&check;** | **&check;** |
|    14.0.x    |      llvm14-0      | **&check;** | **&check;** | **&check;** |

## Example

A simple LLVM plugin which defines two passes, one being a transformation pass that queries the result of a second pass,
an analysis one:

```rust
// Define an LLVM plugin (a name and a version is required). Only cdylib crates
// should define plugins, and only one definition should be done per crate.
#[llvm_plugin::plugin(name = "plugin_name", version = "0.1")]
mod plugin {
    use llvm_plugin::{
        LlvmModuleAnalysis, LlvmModulePass, ModuleAnalysisManager, PreservedAnalyses,
    };
    use llvm_plugin::inkwell::module::Module;

    // Must implement the `Default` trait.
    #[derive(Default)]
    struct Pass1;

    // Define a transformation pass (a name is required). Such pass is allowed to
    // mutate the LLVM IR. If it does, it should return `PreservedAnalysis::None`
    // to notify the pass manager that all analyses are now invalidated.
    #[pass(name = "pass_name")]
    impl LlvmModulePass for Pass1 {
        fn run_pass(
            &self,
            module: &mut Module,
            manager: &ModuleAnalysisManager,
        ) -> PreservedAnalyses {
            // Ask the pass manager for the result of the analysis pass `Analysis1`
            // defined further below. If the result is not in cache, the pass
            // manager will call `Analysis1::run_analysis`.
            let result = manager.get_result::<Analysis1>(module);

            assert_eq!(result, "Hello World!");

            // no modification was made on the module, so the pass manager doesn't have
            // to recompute any analysis
            PreservedAnalyses::All
        }
    }

    // Must implement the `Default` trait.
    #[derive(Default)]
    struct Analysis1;

    // Define an analysis pass. Such pass is not allowed to mutate the LLVM IR. It should
    // be used only for inspection of the LLVM IR, and can return some result that will be
    // efficiently cached by the pass manager (to prevent recomputing the same analysis
    // every time its result is needed).
    #[analysis]
    impl LlvmModuleAnalysis for Analysis1 {
        fn run_analysis(
            &self,
            module: &Module,
            manager: &ModuleAnalysisManager,
        ) -> String {
            // .. inspect the LLVM IR of the module ..

            "Hello World!".to_owned()
        }
    }
}
```

Once you compiled your custom plugin, you can use it during the compilation of C/C++ with the `opt` tool:

```bash
$ clang -c -emit-llvm main.cc -o main.bc
$ opt -load-pass-plugin=libplugin.so -passes="pass_name" main.bc -o main.bc
$ llc main.bc -o main.s
$ clang -static main.s -o main
```

To learn more about how to sequentially apply more than one pass, read this [opt guide].

[opt guide]: https://llvm.org/docs/NewPassManager.html#invoking-opt

## Linux & MacOS Requirements

Your LLVM toolchain should dynamically link the LLVM library. Fortunately, this is the case for toolchains
distributed on `apt` and `homebrew` registeries.

If you are not in this case, you have to compile LLVM from sources by specifying the `LLVM_LINK_LLVM_DYLIB=ON`
cmake flag.

## Windows Requirements

You have to compile LLVM from sources in any case, because you need to apply some patches to the LLVM
code base before compiling. Then, you need to specify the `LLVM_EXPORT_SYMBOLS_FOR_PLUGINS=ON` cmake flag
while leaving the `LLVM_TARGETS_TO_BUILD` flag to its default value.

Here are the detailed steps (replace `llvm-XX` with the one matching your LLVM version):
```bash
$ cat ci/windows/llvm-XX.patch | patch -p1 -d <YOUR_LLVM_SRC_DIR>
$ mkdir <YOUR_LLVM_SRC_DIR>/build && cd <YOUR_LLVM_SRC_DIR>/build
$ cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX=<YOUR_INSTALL_PATH> \
    -DLLVM_EXPORT_SYMBOLS_FOR_PLUGINS=ON \
    -G Ninja
$ ninja install
$ cp lib/opt.lib <YOUR_INSTALL_PATH>/lib
```

Make sure you updated your `$PATH` environment variable with `<YOUR_INSTALL_PATH>/bin`
