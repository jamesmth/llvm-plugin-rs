llvm-plugin-rs 
==============

[<img alt="version" src="https://img.shields.io/crates/v/llvm-plugin.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/llvm-plugin)
[<img alt="doc" src="https://img.shields.io/badge/docs.rs-llvm--plugin-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/llvm-plugin)
[<img alt="linux" src="https://img.shields.io/github/actions/workflow/status/jamesmth/llvm-plugin-rs/linux.yml?branch=master&label=linux&style=for-the-badge&logo=linux" height="20">](https://github.com/jamesmth/llvm-plugin-rs/actions/workflows/linux.yml?query=branch%3Amaster)
[<img alt="windows" src="https://img.shields.io/github/actions/workflow/status/jamesmth/llvm-plugin-rs/windows.yml?branch=master&label=windows&style=for-the-badge&logo=windows" height="20">](https://github.com/jamesmth/llvm-plugin-rs/actions/workflows/windows.yml?query=branch%3Amaster)
[<img alt="macos" src="https://img.shields.io/github/actions/workflow/status/jamesmth/llvm-plugin-rs/macos.yml?branch=master&label=macos&style=for-the-badge&logo=apple" height="20">](https://github.com/jamesmth/llvm-plugin-rs/actions/workflows/macos.yml?query=branch%3Amaster)

This crate gives the ability to safely implement passes for the [new LLVM pass manager], by leveraging the strongly typed interface
provided by [Inkwell].

If you have never developed LLVM passes before, you can take a look at the available [examples]. They will (hopefully) give you a
better idea of how to use this crate.

If you want a deeper understanding of the many concepts surrounding the new LLVM pass manager, you should read the [official LLVM
documentation].

[Inkwell]: https://github.com/TheDan64/inkwell
[new LLVM pass manager]: https://blog.llvm.org/posts/2021-03-26-the-new-pass-manager/
[examples]: https://github.com/jamesmth/llvm-plugin-rs/tree/master/examples
[official LLVM documentation]: https://llvm.org/docs/NewPassManager.html

## Usage

When importing this crate in your `Cargo.toml`, you will need to specify the LLVM version to use with a corresponding feature flag:

```toml
[dependencies]
llvm-plugin = { git = "https://github.com/jamesmth/llvm-plugin-rs", features = ["llvm21-1"] }
```

Supported versions: LLVM 11-21 mapping to a cargo feature flag `llvmX-Y` where `X` and `Y` are the LLVM major and minor versions.

## Getting Started

An LLVM plugin is merely a dylib that is given a [PassBuilder] by the LLVM tool (e.g. [opt], [lld])
loading it.
Therefore, you must add the following line in your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]
```

A [PassBuilder] allows registering callbacks on specific actions being performed by the LLVM tool.

For instance, the `--passes` parameter of [opt] allows specifying a custom pass pipeline to be run on a given IR module. A plugin
could therefore register a callback for parsing an element of the given pipeline (e.g. a pass name), in order to insert a custom
pass to run by [opt].

The following code illustrates the idea:

```rust
use llvm_plugin::inkwell::module::Module;
use llvm_plugin::{
    LlvmModulePass, ModuleAnalysisManager, PassBuilder, PipelineParsing, PreservedAnalyses,
};

// A name and version is required.
#[llvm_plugin::plugin(name = "plugin_name", version = "0.1")]
fn plugin_registrar(builder: &mut PassBuilder) {
    // Add a callback to parse a name from the textual representation of
    // the pipeline to be run.
    builder.add_module_pipeline_parsing_callback(|name, manager| {
        if name == "custom-pass" {
            // the input pipeline contains the name "custom-pass",
            // so we add our custom pass to the pass manager
            manager.add_pass(CustomPass);

            // we notify the caller that we were able to parse
            // the given name
            PipelineParsing::Parsed
        } else {
            // in any other cases, we notify the caller that our
            // callback wasn't able to parse the given name
            PipelineParsing::NotParsed
        }
    });
}

struct CustomPass;
impl LlvmModulePass for CustomPass {
    fn run_pass(
        &self,
        module: &mut Module,
        manager: &ModuleAnalysisManager
    ) -> PreservedAnalyses {
        // transform the IR
        todo!()
    }
}
```

Now, executing this command would run our custom pass on some input `module.bc`:

```bash
opt --load-pass-plugin=libplugin.so --passes=custom-pass module.bc -disable-output
```

However, executing this command would not (`custom-pass2` cannot be parsed by our plugin):

```bash
opt --load-pass-plugin=libplugin.so --passes=custom-pass2 module.bc -disable-output
```

More callbacks are available, read the [documentation] for more details.

To learn more about how to sequentially apply more than one pass, read this [opt guide].

[opt]: https://www.llvm.org/docs/CommandGuide/opt.html
[lld]: https://lld.llvm.org/
[PassBuilder]: https://docs.rs/llvm-plugin/latest/llvm_plugin/struct.PassBuilder.html
[documentation]: https://docs.rs/llvm-plugin
[opt guide]: https://llvm.org/docs/NewPassManager.html#invoking-opt

## Linux & MacOS Requirements

Your LLVM toolchain should dynamically link the LLVM library. Fortunately, this is the case for toolchains
distributed on `apt` and `homebrew` registeries.

<details>
 <summary><em>Install LLVM-14 with apt</em></summary>

 ```shell
 $ apt install llvm-14
 ```

 </details>

<details>
 <summary><em>Install LLVM-14 with homebrew</em></summary>

 ```shell
 $ brew install llvm@14
 ```

 </details>

If you don't use any of these package managers, you can download a compatible LLVM toolchain from
this [LLVM fork] instead. In this case, don't forget to update your `PATH` environment variable with
your LLVM toolchain path, or use the `LLVM_SYS_XXX_PREFIX` environment variable to locate your toolchain.

For instance, if your LLVM-14 toolchain is located at `~/llvm`, you should set either of the following:
- `PATH=$PATH;$HOME/llvm/bin`
- `LLVM_SYS_140_PREFIX=$HOME/llvm`

## Windows Requirements

The official LLVM toolchain for Windows was not built with plugin support. However, compatible toolchains can be found
[here](https://github.com/jamesmth/llvm-project/releases).

Don't forget to update your `PATH` environment variable with your LLVM toolchain path, or use the `LLVM_SYS_XXX_PREFIX`
environment variable to locate your toolchain.

For instance, if your LLVM-14 toolchain is located at `C:\llvm`, you should set either of the following:
- `PATH=$PATH;C:\llvm\bin`
- `LLVM_SYS_140_PREFIX=C:\llvm`

## Compiling Rust/C++ code with custom LLVM plugins

This [LLVM fork] explains how to do so, and provides LLVM toolchains that will make the process easier.

[LLVM fork]: https://github.com/jamesmth/llvm-project

## Missing Features

- Support for loop passes (`Inkwell` doesn't currently provide safe wrappers)
- Support for CGSCC passes (`Inkwell` doesn't currently provide safe wrappers)
- FFI over the full manager proxy API (only a subset is currently implemented)
- FFI over the full analysis invalidation API (only a subset is currently implemented)
- FFI over builtin LLVM analyses (e.g. dominator tree)

Contributions are very welcome, make sure to check out the [Contributing Guide] first!

[Contributing Guide]: ./.github/CONTRIBUTING.md
