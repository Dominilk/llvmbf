# llvmbf
> An LLVM frontend for the brainf*ck esoteric programming language.

Spec of brainf*ck can be found [here](https://github.com/sunjay/brainfuck/blob/master/brainfuck.md).
> **Disclaimer:** Tape is fixed-size of 2^15, dynamic alloc would be needed for "infinite" size - didn't find the time to implement this - therefore tape wrap-around is used.

# dependencies
LLVM (up to 17) installed on system.
General installation instructions can be found [here](https://apt.llvm.org). On macOS you can also install via `brew`:
```
brew install llvm@17
```
Be sure to add to path so that it will be found by `llvm-sys`.

# build
```
cargo build --release
```

# examples
**Run generated IR directly using `lli`:**
```
> cargo run -- examples/helloworld.bf | lli
```
**Compile & link IR to executable:**
```
> cargo run -- examples/helloworld.bf > helloworld.ll
> clang helloworld.ll
> ./a.out
```