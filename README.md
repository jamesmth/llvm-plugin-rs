llvm-plugin-rs 
==============

[<img alt="version" src="https://img.shields.io/crates/v/llvm-plugin.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/llvm-plugin)
[<img alt="doc" src="https://img.shields.io/badge/docs.rs-llvm--plugin-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/llvm-plugin)
[<img alt="linux" src="https://img.shields.io/github/workflow/status/jamesmth/llvm-plugin-rs/Linux%20Test%20Suite/develop?label=linux&style=for-the-badge&logo=linux" height="20">](https://github.com/jamesmth/llvm-plugin-rs/actions/workflows/linux.yml?query=branch%3Adevelop)
[<img alt="windows" src="https://img.shields.io/github/workflow/status/jamesmth/llvm-plugin-rs/Windows%20Test%20Suite/develop?label=windows&style=for-the-badge&logo=windows" height="20">](https://github.com/jamesmth/llvm-plugin-rs/actions/workflows/windows.yml?query=branch%3Adevelop)
[<img alt="macos" src="https://img.shields.io/github/workflow/status/jamesmth/llvm-plugin-rs/MacOS%20Test%20Suite/develop?label=macos&style=for-the-badge&logo=apple" height="20">](https://github.com/jamesmth/llvm-plugin-rs/actions/workflows/macos.yml?query=branch%3Adevelop)

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
llvm-plugin = { version = "0.2", features = ["llvm10-0"] }
```

Supported versions:

| LLVM Version | Cargo Feature Flag |    Linux    |   Windows   |    MacOS    |
| :----------: | :----------------: | :---------: | :---------: | :---------: |
|    10.0.x    |      llvm10-0      | **&check;** | **&cross;** | **&check;** |
|    11.0.x    |      llvm11-0      | **&check;** | **&check;** | **&check;** |
|    12.0.x    |      llvm12-0      | **&check;** | **&check;** | **&check;** |
|    13.0.x    |      llvm13-0      | **&check;** | **&check;** | **&check;** |
|    14.0.x    |      llvm14-0      | **&check;** | **&check;** | **&check;** |

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
[PassBuilder]: https://docs.rs/llvm-plugin/struct.PassBuilder.html
[documentation]: https://docs.rs/llvm-plugin
[opt guide]: https://llvm.org/docs/NewPassManager.html#invoking-opt

## Linux & MacOS Requirements

Your LLVM toolchain should dynamically link the LLVM library. Fortunately, this is the case for toolchains
distributed on `apt` and `homebrew` registeries.

If you are not in this case, you have to compile LLVM from sources by specifying the `LLVM_LINK_LLVM_DYLIB=ON`
cmake flag.

<details>
 <summary><em>Compile LLVM-14</em></summary>

 ```shell
 $ wget https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0/llvm-14.0.0.src.tar.xz
 $ tar xf llvm-14.0.0.src.tar.xz && cd llvm-14.0.0.src
 $ mkdir build && cd build
 $ cmake .. \
     -DCMAKE_BUILD_TYPE=Release \
     -DCMAKE_INSTALL_PREFIX="$HOME/llvm" \
     -DLLVM_LINK_LLVM_DYLIB=ON \
     -G Ninja
 $ ninja install
 ```

 </details>

## Windows Requirements

You have to compile LLVM from sources in any case, because you need to apply some patches to the LLVM
code base before compiling. Then, you need to specify the `LLVM_EXPORT_SYMBOLS_FOR_PLUGINS=ON` cmake flag
while leaving the `LLVM_TARGETS_TO_BUILD` flag to its default value.

<details>
 <summary><em>Compile LLVM-14</em></summary>

 ```shell
 $ wget https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0/llvm-14.0.0.src.tar.xz
 $ tar xf llvm-14.0.0.src.tar.xz && cd llvm-14.0.0.src
 $ cat ../ci/windows/llvm-14.patch | patch -p1
 $ mkdir build && cd build
 $ cmake .. \
     -DCMAKE_BUILD_TYPE=Release \
     -DCMAKE_INSTALL_PREFIX="C:\\llvm" \
     -DLLVM_EXPORT_SYMBOLS_FOR_PLUGINS=ON \
     -G Ninja
 $ ninja install
 $ cp lib/opt.lib /c/llvm/lib
 ```

</details>

Don't forget to update your `PATH` environment variable with your LLVM installation path.

## Missing Features

- Support for loop passes (`Inkwell` doesn't currently provide safe wrappers)
- Support for CGSCC passes (`Inkwell` doesn't currently provide safe wrappers)
- FFI over the full manager proxy API (only a subset is currently implemented)
- FFI over the full analysis invalidation API (only a subset is currently implemented)
- FFI over builtin LLVM analyses (e.g. dominator tree)

Contributions are very welcome, make sure to check out the [Contributing Guide] first!

[Contributing Guide]: ./.github/CONTRIBUTING.md