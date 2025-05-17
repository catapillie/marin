### Quick overview

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
"forall a b c, fun(a, b) => c, where [Addable(a, b) of c]"
"^^^^^^^^^^^^  ^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^"
"   domain           type              constraints       "


"forall a b c d e,
    fun(d, e, c) => a,
where [Addable(b, c) of a], [Addable(d, e) of b]"
let sum3_left(a, b, c) = add(add(a, b), c)

"forall a b c d e,
    fun(b, d, e) => a,
where [Addable(d, e) of c], [Addable(b, c) of a]"
let sum3_right(a, b, c) = add(a, add(b, c))

sum3_left(0, 1, 2)
sum3_left(0, 1.8, 2) "why not"
sum3_left("", "12", "hello")
```

An example, the `Monoid` class (taken from [`std/monoid.mar`](../std/monoid.mar))
```ocaml
pub class Monoid(K)
    empty: K
    append(K, K) => K
end

pub alias Monoid.empty as empty
pub alias Monoid.append as append

pub let concat = List.fold_right(append, empty)
```