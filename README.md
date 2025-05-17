# `marin`

An experimental functional+imperative programming language, inspired by OCaml, which features the more advanced concept of typeclasses (or just classes).

The main features of Marin are :
* A type system based on Hindley-Milner, extended with type-`class`, and various useful constructs such as `union`, `record`.
* Full type inference which attempts to type `let`-bindings with the most general type-scheme, taking into account type-class constraints.
* A functional+imperative feel, as seen in [OCaml](https://ocaml.org/), or [Rust](https://www.rust-lang.org/).
* Clean and readable syntax, strongly inspired by [Lua](https://www.lua.org/)'s.
* Runs on a tiny virtual machine with its own bytecode.

---

**Table of contents**
* [About](#about)
* [Installation](#installation)
  * [Requirements](#requirements)
* [Usage](#usage)
* [Quick overview](#quick-overview)
* [Standard library](#standard-library)

## About

This project is originally meant to be an implementation for a personal school-related project (theorical computer science, compilers, their implementation). However, I'm also working on it to play around with my own ideas, especially to explore the design of a programming language.

Resources which I rely on for this project:
* [Types and Programming Languages](https://www.cis.upenn.edu/~bcpierce/tapl/), Benjamin C. Pierce
* [Crafting Interpreters](https://craftinginterpreters.com/), Robert Nystrom

## Installation

### Requirements
* Rust (2024 edition)
* Cargo

Clone the repository and build the application :
```sh
git clone https://github.com/catapillie/marin
cd marin
cargo build --release
```

The executable should be located in `./target/release/`.

> Please note that the app builds with an `std` folder meant for internal use by the compiler.

## Usage
> Currently the compiler is still it its initial WIP state and does not have many CLI options.

Check and compile a program to bytecode, then immediately execute it with:
```sh
marin <files...> [options...]
```

Available options:
* **`--no-std`**: prevents Marin's standard library from being automatically imported in your project.
* **`--show-disassembly`**: prints all of the bytecode upon execution.

Marin source files are meant to end with the `.mar` extension. Multiple files can be used, and can depend on each other with the `import` statement. Dependency cycles are forbidden.

## Quick overview
Find some examples in [`docs/overview.md`](./docs/overview.md).

## Standard library
The current standard library for Marin is automatically imported in every file (except if `--no-std` is on), and contains various definitions such as data types, mathematical functions and operations, and typeclasses. **You can find its documentation in [`docs/std.md`](./docs/std.md).**
