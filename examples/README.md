# Examples

This directory contains a collection (in progress) of examples to show what can be quickly
achieved with simple LLVM passes.

- [**Hello World**][hello-world], is a rewrite of one the examples from [llvm-tutor]

   <details>
    <summary><em>Execute the example</em></summary>

    ```shell
    $ cargo b --example hello-world --features llvm10-0
    $ opt --load-pass-plugin=target/debug/examples/libhello_world.so --passes=hello-world in.ll -S -o out.ll
    ```

   </details>

- [**Opcode Counter**][opcode-counter], is a rewrite of one the examples from [llvm-tutor]

   <details>
    <summary><em>Execute the example</em></summary>

    ```shell
    $ cargo b --example opcode-counter --features llvm10-0
    $ opt --load-pass-plugin=target/debug/examples/libopcode_counter.so --passes=opcode-counter-printer in.ll -S -o out.ll
    ```

   </details>

- [**Inject Function Call**][inject-printf], is a rewrite of one the examples from [llvm-tutor]

   <details>
    <summary><em>Execute the example</em></summary>

    ```shell
    $ cargo b --example inject-printf --features llvm10-0
    $ opt --load-pass-plugin=target/debug/examples/libinject_printf.so --passes=inject-func-call in.ll -S -o out.ll
    ```

   </details>

- [**Static Call Counter**][static-call-counter], is a rewrite of one the examples from [llvm-tutor]

   <details>
    <summary><em>Execute the example</em></summary>

    ```shell
    $ cargo b --example static-call-counter --features llvm10-0
    $ opt --load-pass-plugin=target/debug/examples/libstatic_call_counter.so --passes=static-cc-printer in.ll -S -o out.ll
    ```

   </details>

- [**Strings Obfuscator**][strings], is a rewrite of [llvm-string-obfuscator]

   <details>
    <summary><em>Execute the example</em></summary>

    ```shell
    $ cargo b --example string-obf --features llvm10-0
    $ opt --load-pass-plugin=target/debug/examples/libstring_obf.so --passes=string-obfuscator-pass in.ll -S -o out.ll
    ```

   </details>

[llvm-tutor]: https://github.com/banach-space/llvm-tutor
[llvm-string-obfuscator]: https://github.com/tsarpaul/llvm-string-obfuscator

[hello-world]: ./hello_world.rs
[opcode-counter]: ./opcode_counter.rs
[inject-printf]: ./inject_printf.rs
[static-call-counter]: ./static_call_counter.rs
[strings]: ./strings.rs
