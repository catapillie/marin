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
  * [Just show me what it looks like](#just-show-me-what-it-looks-like)
* [Standard library](#standard-library)

## About

This project is originally meant to be an implementation for a personal school-related project (theorical computer science, compilers, their implementation). However, I'm also working on it to play around with my own ideas, especially to explore the design of a programming language.

Resources which I rely on for this project:
* [Types and Programming Languages](https://www.cis.upenn.edu/~bcpierce/tapl/), Benjamin C. Pierce
* [Crafting Interpreters](https://craftinginterpreters.com/), Robert Nystrom

## Installation

### Requirements
* Rust (2021 edition)
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
#### `--no-std`: prevents Marin's standard library from being automatically imported in your project.

Marin source files are meant to end with the `.mar` extension. Multiple files can be used, and can depend on each other with the `import` statement. Dependency cycles are forbidden.

## Quick overview
### Just show me what it looks like

No main function is required
```ocaml
"hello, world"
```

Let-bindings and built-in types
```ocaml
let a = 0
let u = ()
let (x, y) = (1.7, 5.2)
let text = "wherever you go"
let _ = [true, true, false, true, false]

let id(x) = x
let dup = fun(x) => (x, x)
let compose(g, f)(x) = f(g(x))
```

Record types
```ocaml
record person
    name: string
    age: int
    height: float
    likes: list
end

let myself = {
    name = "catapillie"
    age = 20
    height = 1.85
    likes = list.cons("music", list.cons("programming", list.empty))
}

let name = person.name(myself)


pub record vec2D(k)
    x: k
    y: k
end

let pos = { x = 1.0, y = 8.0 }
let vel = { x = 0.0, y = 4.5 }

let { x, y } = vel
```

Union types
```ocaml
pub union binary_tree(k)
    empty
    node(binary_tree(k), k, binary_tree(k))
end

pub alias binary_tree.empty as empty
pub alias binary_tree.node as node

let single(x) = node(empty, x, empty)

let tree = node(empty, "root", node(single("left"), "child", single("right")))
```

Pattern-matching
```ocaml
pub union tree(k)
    empty
    node(tree(k), k, tree(k))
end

let is_empty(t) match t with
    tree.empty => true
    _          => false
end

let flip(t) match t with
    tree.empty                => tree.empty
    tree.node(left, x, right) => tree.node(
        flip(left)
        x
        flip(right)
    )
end


pub record vec2D(k)
    x: k
    y: k
end

let is_on_axis(v) match v with
    { x = 0, y } => true
    { x, y = 0 } => true
    _            => false
end
```

Control flow
```ocaml
let tup = do
    let x = 0
    let y = 1
    (x, y)
end

let () = if is_even(7) then
    display("non-exhaustive expressions")
    "always return unit"
end

let final_value = while is_running() do
    update()
    render()
else
    "app finished"
end

"labels"
let integer = do<x>
    let b = true
    do<y>
        if b then
            break<x> 0
        end
    end
    1
end

let exit_code = loop<app>
    display("infinite loop")
    while<e> poll_event() do
        match get_event() with
            exit => break<app> 0
            skip_frame => skip<app>
            dismiss_events => break<e>
            _ => ()
        end
    end

    match update() with
        error => break<app> 1
        _ => ()
    end
end

let text = if some_condition(a, b, c) then
    "hi"
else if some_other_condition(a, b, c) then
    "hello"
else
    "...bye"
end
```

Type classes (or just "class") and class constraints
```ocaml
class Default(t)
    default: t
end

alias Default.default as default

let default_tup = (default, default)

let points = [(0, 0), (1, 2), default_tup, (8, 4)]
let points_f = [(0.1, 0.2), (1.3, 2.4), default_tup, (8.7, 4.8)]
let opposites = [default_tup, ("black", "white"), ("up", "down")]

"this function only works for option(X), where Default(X) has an implementation"
let unwrap_or_default(opt) match opt with
    some(x) => x
    none    => default
end

"let's implement Default for four basic types"
"we need to initialize default"
"the types are fully inferred"

have Default
    let default = 0
end

have Default
    let default = 0.0
end

have Default
    let default = false
end

have Default
    let default = ""
end

"class implementations that require other class implementation"
have Default
    let default = (Default.default, Default.default)
end
```
Associated type
```ocaml
"the type Result is unique for a given implementation of Addable"
class Addable(Left, Right) of Result
    add(Left, Right) => Result
end

alias Addable.add as add
"forall T b c, fun(a, b) => c, where [Addable(a, b) of c]"
"^^^^^^^^^^^^  ^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^"
"   domain           type              constraints       "


"forall T b c d e,
    fun(d, e, c) => a,
where [Addable(b, c) of a], [Addable(d, e) of b]"
let sum3_left(a, b, c) = add(add(a, b), c)

"forall T b c d e,
    fun(b, d, e) => a,
where [Addable(d, e) of c], [Addable(b, c) of a]"
let sum3_right(a, b, c) = add(a, add(b, c))

sum3_left(0, 1, 2)
sum3_left(0, 1.8, 2) "why not"
sum3_left("", "12", "hello")
```

An example, the Monoid class (taken from [`std/monoid.mar`](./std/monoid.mar))
```ocaml
pub class Monoid(K)
    empty: K
    append(K, K) => K
end

pub alias Monoid.empty as empty
pub alias Monoid.append as append

pub let concat = List.fold_right(append, empty)
```

## Standard library
The current standard library for Marin, which is automatically imported in every file (except if `--no-std` is on).

Summary
* [Utility functions](#utility-functions)
* [`class Default(T)`](#class-defaultt)
* [`union option(T)`](#union-optiont)
* [`union either(X, Y)`](#union-eitherx-y)
* [`union list(T)`](#union-listt)
* [`class monoid(K)`](#class-monoidk)

### [Utility functions](./std/prelude.mar)
#### `id` (`forall A, fun(A) => A`)
The utility function over any type.
#### `compose` (`forall A B C, fun((fun(A) => B), (fun(B) => C))(A) => C`)
Function composition (inverted notation). `compose(g, f)` is itself a function which takes `x` and returns `f(g(x))`.
#### `dup` (`forall A, fun(A) => (A, A)`)
Duplicates an element `x` and returns a pair `(x, x)`.
#### `swap` (`forall A B, fun((A, B)) => (B, A)`)
Swap the two values in a tuple.
#### `rot_left` (`forall A B C, fun((C, A, B)) => (A, B, C)`)
Rotates the elements of a 3-tuple to the left. 
#### `rot_right` (`forall A B C, fun((B, C, A)) => (A, B, C)`)
Rotates the elements of a 3-tuple to the right.

### [`class Default(T)`](./std/default.mar)
Represents the class of types which have a "default" value
#### `Default.default` (`forall T, T, where [Default(T)]`)
The default value.
#### Provided implementations
* [`have Default(int)`](./std/default_primitives.mar)
* [`have Default(float)`](./std/default_primitives.mar)
* [`have Default(string)`](./std/default_primitives.mar)
* [`have Default(bool)`](./std/default_primitives.mar)
* [`have Default(())`](./std/default_tuples.mar)
* [`have Default((T, T))`](./std/default_tuples.mar)
* [`have Default((T, T, T))`](./std/default_tuples.mar)
* [`have Default((T, T, T, T))`](./std/default_tuples.mar)
* [`have Default((T, T, T, T, T))`](./std/default_tuples.mar)
* [`have Default((T, T, T, T, T, T))`](./std/default_tuples.mar)
* [`have Default((T, T, T, T, T, T, T))`](./std/default_tuples.mar)
* [`have Default((T, T, T, T, T, T, T, T))`](./std/default_tuples.mar)

### [`union option(T)`](./std/option.mar)
The classic union type which represents either a value of type `T`, or nothing.
#### `Option.none` (`forall T, option(T)`)
The "none" variant
#### `Option.some` (`forall T, fun(T) => option(T)`)
The "some" variant, takes in a value from type `T`
#### `Option.is_none` (`forall T, fun(option(T)) => bool`)
Returns whether an option is none.
#### `Option.is_some` (`forall T, fun(option(T)) => bool`)
Returns whether an option is some value.
#### `Option.map` (`forall X Y, fun(fun(X) => Y)(option(X)) => option(Y)`)
Applies the provided function `f` onto the inner element if the option is some, otherwise remains none.
#### `Option.unwrap_or` (`forall T, fun(T)(option(T)) => T`)
Extracts the inner value if the option is some, or return the provided fallback value.
#### `Option.unwrap_or` (`forall T, fun(option(T)) => T, where Default(T)`)
Extracts the inner value if the option is some, or return the default value for type `T`. The class constraint `Default(T)` must be satisfied.
#### [implements `Default`](./std/default_option.mar) (`forall T, have Default(option(T))`)
The default value of an option is `none`.

### [`union either(X, Y)`](./std/either.mar)
A union which describe a choice between two values (left or right).
#### `Either.left` (`forall A B, fun(A) => either(A, B)`)
The "left" variant, containing the provided value `x` of type `A` in the left slot.
#### `Either.right` (`forall A B, fun(B) => either(A, B)`)
The "right" variant, containing the provided value `x` of type `B` in the right slot.
#### `Either.is_left` (`forall A B, fun(either(A, B)) => bool`)
Returns whether the left slot contains the value.
#### `Either.is_right` (`forall A B, fun(either(A, B)) => bool`)
Returns whether the right slot contains the value.
#### `Either.opt_left` (`forall A B, fun(either(A, B)) => option(A)`)
Returns `some(x)` if `x` is in the left slot, and `none` otherwise.
#### `Either.opt_right` (`forall A B, fun(either(A, B)) => option(B)`)
Returns `some(x)` if `x` is in the right slot, and `none` otherwise.
#### `Either.unwrap_left_or` (`forall A B, fun(A)(either(A, B)) => A`)
Returns `x` if `x` is in the left slot. If it isn't, the provided fallback value is returned instead.
#### `Either.unwrap_left_or_default` (`forall A B, fun(either(A, B)) => a, where [Default(A)]`)
Returns `x` if `x` is in the left slot. If it isn't, the default value for type `A` is returned instead.
#### `Either.unwrap_right_or` (`forall A B, fun(B)(either(A, B)) => B`)
Returns `x` if `x` is in the right slot. If it isn't, the provided fallback value is returned instead.
#### `Either.unwrap_right_or_default` (`forall A B, fun(either(b, a)) => a, where [Default(B)]`)
Returns `x` if `x` is in the right slot. If it isn't, the provided fallback value is returned instead.
#### `Either.map_left` (`forall AX AY B, fun((fun(AX) => AY))(either(AX, B)) => either(AY, B)`)
Applies a function `f` on the value in the left slot, and leaves the right slot unchanged.
#### `Either.map_right` (`forall A BX BY, fun((fun(BX) => BY))(either(A, BX)) => either(A, BY)`)
Applies a function `f` on the value in the right slot, and leaves the left slot unchanged.

### [`union list(T)`](./std/list.mar)
A simple linked list over any type, implemented as a union.
#### `List.empty` (`forall T, list(T)`)
The unique empty list.
#### `List.cons` (`forall T, fun(T, list(T)) => list(T)`)
The list constructor, which takes an element `x` and a tail list `tl`, and creates a new list whose head is `x` and has tail `tl`.
#### `List.head` (`forall T, fun(list(T)) => option(T)`)
Returns `some(x)` if the list is not empty (and has head `x`), otherwise returns `none`.
#### `List.tail` (`forall T, fun(list(T)) => option(list(T))`)
Returns `some(tl)` if the list is not empty (and has tail `tl`), otherwise returns `none`.
#### `List.map` (`forall A B, fun((fun(A) => B))(list(A)) => list(B)`)
Applies the function `f` (`A => B`) to each element of the list, changing it from a `list(A)` to a `list(B)`.
#### `List.iter` (`forall A B, fun((fun(A) => B))(list(A)) => ()`)
Applies the function `f` (`A => B`) to each element of the list, and returns unit. The results of `f` are ignored, so `f` can return any type.
#### `List.fold_right` (`forall A B, fun((fun(A, B) => B), B)(list(A)) => B`)
Given a list `[x_0, x_1, ..., x_n]`, a function `f` and an initial value `init`, calculate `f(x_0, f(x_1, f(..., f(x_n, init))))`.
#### `List.fold_left` (`forall A B, fun(A, (fun(A, B) => A))(list(B)) => A`)
Given a list `[x_0, x_1, ..., x_n]`, a function `f` and an initial value `init`, calculate `f(f(f(f(init, x_n), ...), x_1), x_0)`.
#### `List.filter` (`forall T, fun((fun(T) => bool))(list(T)) => list(T)`)
Extracts a list which only contains values `x` which verify `f(x) = true`.
#### `List.filter_map` (`forall A B, fun((fun(A) => option(B)))(list(A)) => list(B)`)
Extracts a list which only contains values `f(x)` which verify `f(x) = Some(...)`. This makes it possible to filter a list and map its elements at the same time.
#### `List.forall` (`forall T, fun((fun(T) => bool))(list(T)) => bool`)
Returns whether all elements of the list satisfy the given predicate.
#### `List.exists` (`forall T, fun((fun(T) => bool))(list(T)) => bool`)
Returns whether there exists an element of the list which satisfies the given predicate.
#### `List.rev` (`forall T, fun(list(T)) => list(T)`)
Reverse a list.
#### `List.find` (`forall T, fun((fun(T) => bool))(list(T)) => option(T)`)
Attempts to find an element `x` which satisfies the given predicate. If such an element exists, the function returns `some(x)` where `x` is the first such element in the list. `none` is otherwise returned.
#### `List.find_map` (`forall A B, fun((fun(A) => option(B)))(list(A)) => option(B)`)
Attempts to find an element `x` which verifies `f(x) = Some(y)`. If such an element exists, the function returns `some(y)` where `Some(y) = f(x)`, and where `x` is the first such element in the list. `none` is otherwise returned.
#### `List.take_while` (`forall T, fun((fun(T) => bool))(list(T)) => list(T)`)
Returns the first elements of the list which satisfy the given predicate, and leaves the rest as soon as the predicate is unverified.
#### `List.take_while_map` (`forall A B, fun((fun(A) => option(B)))(list(A)) => list(B)`)
Returns the first elements `y` of the list for which we find `f(x) = Some(y)`, and leaves the rest as soon as an element `x` such that `f(x) = none` is encountered.
#### `List.take_until` (`forall T, fun((fun(a) => bool))(list(T)) => list(T)`)
Returns the first elements from the list who don't satisfy the predicate, and ignore the rest as soon as the predicate is verified.
#### `List.drop_while` (`forall T, fun((fun(a) => bool))(list(T)) => list(T)`)
Removes the first elements from the list who satisfy the predicate, and keeps the rest as soon as the predicate is unverified.
#### `List.drop_until` (`forall T, fun((fun(a) => bool))(list(T)) => list(T)`)
Removes the first elements from the list who don't satisfy the predicate, and keeps the rest as soon as the predicate is verified.
#### `List.zip` (`forall A B, fun(list(A), list(B)) => list((A, B))`)
Combines a lists `[x_0, x_1, ...]` of type `list(A)` and a list `[y_0, y_1, ...]` of type `list(B)` into a single list of type `list((A, B))`, which consists of pairs `[(x_0, y_0), (x_1, y_1)]`. If one list is longer than the other, the excess is ignored.
#### `List.unzip` (`forall A B, fun(list((A, B))) => (list(A), list(B))`)
Takes a list `[(x_0, y_0), (x_1, y_1)]` of type `list((A, B))` and unzips it as two lists `[x_0, x_1, ...]` and `[y_0, y_1, ...]`, of type `list(A)` and `list(B)` respectively.
#### `List.concat` (`forall T, fun(list(T), list(T)) => list(T)`)
Concatenates two lists.
#### [implements `Default`](./std/default_list.mar) (`forall T, have Default(list(T))`)
The default value of a list is `empty`.
#### [implements `Monoid`](./std/monoid_list.mar) (`forall T, have Monoid(list(T))`)
The set `List(T)` with the concatenation operation forms a monoid. The neutral element for concatenation is `empty`, and the operation is [`List.concat`](#listconcat-forall-t-funlistt-listt--listt).

### [`class Monoid(K)`](./std/monoid.mar)
A monoid is an associative magma with an (unique) identity (or "neutral") element. That is, elements of a monoid type can be manipulated with an associative binary operator `· :: (T, T) => T`, and there exists a unique identity element `e` such that for any `x`, `x · e = e · x = x`.

Implementations of this class must satisfy these mathematical definitions and properties. The compiler cannot verify them.

Within the context of Marin, the monoid operation is called `append`, and the identity element is called `empty`.

#### `Monoid.empty` (`forall K, K, where [Monoid(K)]`)
The identity ("neutral") element of the monoid.
#### `Monoid.append` (`forall K, fun(K, K) => K, where [Monoid(K)]`)
The stable, associative operation of the monoid.
#### `Monoid.concat` (`forall K, fun(list(K)) => K, where [Monoid(K)]`)
Takes in a list of elements `[x_0, x_1, ..., x_n]`, and returns `x_0 · x_1 · ... · x_n`. If the list is empty, by convention, the identity element (`Monoid.empty`) is returned. The order in which the operations is done is unspecified, and does not affect the final returned value.
