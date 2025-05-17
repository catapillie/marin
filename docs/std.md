# `marin` standard library

The language's standard library is implemented in the [`std`](../std/) directory.

The [`std/prelude.mar`](../std/prelude.mar) file imports all of the other modules from the library into one unified module. By default, the prelude file is imported into every user file. This behavior can be disabled with the `--no-std` compiler option.

> [!IMPORTANT]
> Note that omitting the standard library will render all operator symbols useless, since they rely on definitions in the standard library.

Table of contents :
* <u>Utils</u>
    * [`Prelude`](#prelude-stdpreludemar)
    * [`Math`](#math-stdmathmar)
    * [`Assert`](#assert-stdassertmar)
* <u>Data types</u>
    * [`Option`](#option-stdoptionmar)
    * [`Either`](#either-stdeithermar)
    * [`List`](#list-stdlistmar)
* <u>Typeclasses</u>
    * [Operators in `ops`](#operators-in-ops-stdopsmar)
    * [`Default`](#default-stddefaultmar)
    * [`Display`](#display-stddisplaymar)
    * [`Monoid`](#monoid-stdmonoidmar)

<!----------------------------------------------->

## `Prelude` ([`std/prelude.mar`](../std/prelude.mar))
The prelude re-imports all files from [`/std`](../std/), which gives access to all pub declarations in the standard library. It also defines some utility functions:

* **`id(x)`**
The utility function over any type.
* **`compose(f, g)(x)`**
Function composition (inverted notation). `compose(g, f)` is itself a function which takes `x` and returns `f(g(x))`.
* **`dup(x)`**
Duplicates an element `x` and returns a pair `(x, x)`.
* **`swap((a, b))`**
Swap the two values in a tuple.
* **`rot_left((x, y, z))`**
Rotates the elements of a 3-tuple to the left. 
* **`rot_right((x, y, z))`**
Rotates the elements of a 3-tuple to the right.

<!----------------------------------------------->

## `Math` ([`std/math.mar`](../std/math.mar))

A small mathematical library which provides classic functions over floating-point numbers. By convention, all trigonometric functions work with radians.

Constants:
* **`Math.pi_over_two`**
$\pi/2$
* **`Math.pi`**
$\pi$
* **`Math.tau`**
$\tau=2\pi$
* **`Math.e`**
Euler's constant $e$
* **`Math.phi`**
The golden ratio $\varphi$

Exponential functions
* **`Math.pow(x, a)`**
$x^a$
* **`Math.exp(x)`**
$\exp x$
* **`Math.ln(x)`**
$\ln x$
* **`Math.log(b)(x)`**
The base-$b$ logarithm, defined by $\log_b x=\frac{\ln x}{\ln(b)}$
* **`Math.sqrt(x)`**
$\sqrt x$

Hyperbolic functions
* **`Math.cosh(x)`**
$\operatorname{cosh} x$
* **`Math.sinh(x)`**
$\operatorname{sinh} x$
* **`Math.tanh(x)`**
$\operatorname{tanh} x$

Trigonometric functions
* **`Math.sin(x)`**
$\operatorname{sin} x$
* **`Math.asin(x)`**
$\operatorname{asin} x$
* **`Math.csc(x)`**
$\operatorname{csc} x$
* **`Math.acsc(x)`**
$\operatorname{acsc} x$
* **`Math.cos(x)`**
$\operatorname{cos} x$
* **`Math.acos(x)`**
$\operatorname{acos} x$
* **`Math.sec(x)`**
$\operatorname{sec} x$
* **`Math.asec(x)`**
$\operatorname{asec} x$
* **`Math.tan(x)`**
$\operatorname{tan} x$
* **`Math.atan(x)`**
$\operatorname{atan} x$
* **`Math.cot(x)`**
$\operatorname{cot} x$
* **`Math.acot(x)`**
$\operatorname{acot} x$
* **`Math.atan2(y, x)`**
The angle (measured in radians) between the $x$-axis and the line passing through the origin and $(x, y)$. The returned value is always contained in the range $\left]-\pi, +\pi\right]$.

Degrees & radians
* **`Math.to_deg(rad)`**
Radian to degree convertion.
* **`Math.to_rad(deg)`**
Degree to radian convertion.

<!----------------------------------------------->

## `Assert` ([`std/assert.mar`](../std/assert.mar))

Provides a few functions for assertions. When an assertion fails, the `@panic` compiler built-in is called, and when possible, shows why the assertion failed.

Guard assertion
* **`Assert.condition(guard)`**
Asserts that the given booleans value is `true`.

Comparison assertions
* **`Assert.eq(a: A, b: B)`**
Asserts that the two passed values are equal; panics otherwise, displaying the two values. As such, **the function is constrained with** **`Eq(A, B)`, as well as `Display(A)` and `Display(B)`.**
* **`Assert.ne(a: A, b: B)`**
Asserts that the two passed values are *not* equal; panics otherwise, displaying the two values. As such, **the function is constrained with** **`Eq(A, B)`, as well as `Display(A)` and `Display**(B)`.

<!----------------------------------------------->

## `Option` ([`std/option.mar`](../std/option.mar))
The classic union type which represents either a value of type `T`, or nothing.

Variants
* **`Option.none`**
The "none" variant.
* **`Option.some(x: T)`**
The "some" variant, takes in a value from type `T`.

Checking variants
* **`Option.is_none(opt)`**
Returns whether an option is `none`.
* **`Option.is_some(opt)`**
Returns whether an option is `some`.

Unwrapping
* **`Option.unwrap(opt)`**
Extracts the inner value in the option. If there is none, the function causes a panic.
* **`Option.unwrap_or(fallback)(opt)`**
Extracts the inner value if the option is some, or return the provided `fallback` value.
* **`Option.unwrap_or_default(opt)`**
Extracts the inner value if the option is some, or return the default value for type `T`. The class constraint `Default(T)` must thus be satisfied.

Mapping
* **`Option.map(f)(opt)`**
Applies the provided function `f` onto the inner element if the option is some, otherwise remains none.

Related implementations
* **[implements `Default`](../std/default_option.mar)** (`forall T, have Default(option(T))`)
The default value of an option is `none`.
* **[implements `Display`](../std/display_option.mar)** (`forall a, have [Display(option(a))], where [Display(a)]`) The inner type must also implement `Display`.

<!----------------------------------------------->

## `Either` ([`std/either.mar`](../std/either.mar))
A union which describe a choice between two values (left or right), whose types can be different.

Variants
* **`Either.left(a: A)`**
The "left" variant, containing the provided value `x` of type `A` in the left slot.
* **`Either.right(b: B)`**
The "right" variant, containing the provided value `x` of type `B` in the right slot.

Checking variants
* **`Either.is_left(e)`**
Returns whether the left slot contains the value.
* **`Either.is_right(e)`**
Returns whether the right slot contains the value.

Turning into an [`Option`](#option-stdoptionmar)
* **`Either.opt_left(e)`**
Returns `Option.some(x)` if `x` is in the left slot, and `Option.none` otherwise.
* **`Either.opt_right(e)`**
Returns `Option.some(x)` if `x` is in the right slot, and `Option.none` otherwise.

Unwrapping
* **`Either.unwrap_left(e)`**
Returns `x` if `x` is in the left slot. If it isn't, the function panics.
* **`Either.unwrap_left_or(fallback)(e)`**
Returns `x` if `x` is in the left slot. If it isn't, the provided `fallback` value is returned instead.
* **`Either.unwrap_left_or_default(e)`**
Returns `x` if `x` is in the left slot. If it isn't, the default value for the left type is returned instead. Needs a `Default` constraint on the corresponding type.
* **`Either.unwrap_right(e)`**
Returns `x` if `x` is in the right slot. If it isn't, the function panics.
* **`Either.unwrap_right_or(fallback)(e)`**
Returns `x` if `x` is in the right slot. If it isn't, the provided `fallback` value is returned instead.
* **`Either.unwrap_right_or_default(e)`**
Returns `x` if `x` is in the left slot. If it isn't, the default value for the right type is returned instead. Needs a `Default` constraint on the corresponding type.

Mapping
* **`Either.map_left(f)(e)`**
Applies a function `f` on the value in the left slot, and leaves the right slot unchanged.
* **`Either.map_right(f)(e)`**
Applies a function `f` on the value in the right slot, and leaves the left slot unchanged.

Related implementations
* **[implements `Display`](../std/display_either.mar)** (`forall a b, have [Display(either(a, b))], where [Display(a)], [Display(b)]`) Both inner types must also implement `Display`.

<!----------------------------------------------->

## `List` ([`std/list.mar`](../std/list.mar))
A simple linked list over any type, implemented as a union.

Variants
* **`List.empty`**
The unique empty list.
* **`List.cons(x, xs)`**
The list constructor, which takes an element `x` and a tail list `xs`, and creates a new list whose head is `x` and has tail `xs`.

Turning into an [`Option`](#option-stdoptionmar)
* **`List.head(l)`**
Returns `some(x)` if the list is not empty (and has head `x`), otherwise returns `none`.
* **`List.tail(l)`**
Returns `some(tl)` if the list is not empty (and has tail `tl`), otherwise returns `none`.

Mapping and iterating
* **`List.map(f)(l)`**
Applies the function `f` (`A => B`) to each element of the list, changing it from a `list(A)` to a `list(B)`.

Traversing and iterating
* **`List.len(l)`**
Calculates the length of the list, as an integer `int`.
* **`List.iter(f)(l)`**
Applies the function `f` (`A => B`) to each element of the list, and returns unit. The results of `f` are ignored, so `f` can return any type.
* **`List.fold_right(f, init)(l)`**
Given a list `[x_0, x_1, ..., x_n]`, a function `f` and an initial value `init`, calculate `f(x_0, f(x_1, f(..., f(x_n, init))))`.
* **`List.fold_left(init, f)(l)`**
Given a list `[x_0, x_1, ..., x_n]`, a function `f` and an initial value `init`, calculate `f(f(f(f(init, x_n), ...), x_1), x_0)`.
* **`List.forall(p)(l)`**
Returns whether all elements of the list satisfy the predicate `p`.
* **`List.exists(p)(l)`**
Returns whether there exists an element of the list which satisfies the predicate `p`.

Filtering
* **`List.filter(f)(l)`**
Extracts a list which only contains values `x` which verify `f(x) = true`.
* **`List.filter_map(f)(l)`**
Extracts a list which only contains values `y` which verify `f(x) = some(y)`. This makes it possible to filter a list and map its elements at the same time.

Reverse
* **`List.rev(l)`**
Reverse a list.

Searching through a list
* **`List.find`**
Attempts to find an element `x` which satisfies the given predicate. If such an element exists, the function returns `Option.some(x)` where `x` is the first such element in the list. `Option.none` is otherwise returned.
* **`List.find_map`**
Attempts to find an element `x` which verifies `f(x) = some(y)`. If such an element exists, the function returns `Option.some(y)` where `some(y) = f(x)`, and where `x` is the first such element in the list. `Option.none` is otherwise returned.
* **`List.take_while(p)(l)`**
Returns the first elements of the list which satisfy the given predicate, and ignores the rest as soon as the predicate is unverified.
* **`List.take_while_map(f)(l)`**
Returns the first elements `y` of the list for which we find `f(x) = some(y)`, and ignores the rest as soon as an element `x` such that `f(x) = none` is encountered.
* **`List.take_until(p)(l)`**
Returns the first elements from the list who don't satisfy the predicate, and ignores the rest as soon as the predicate is verified.
* **`List.drop_while(p)(l)`**
Removes the first elements from the list who satisfy the predicate, and keeps the rest as soon as the predicate is unverified.
* **`List.drop_until(p)(l)`**
Removes the first elements from the list who don't satisfy the predicate, and keeps the rest as soon as the predicate is verified.

Combining lists
* **`List.zip(la, lb)`**
Combines a lists `[x_0, x_1, ...]` of type `list(A)` and a list `[y_0, y_1, ...]` of type `list(B)` into a single list of type `list((A, B))`, which consists of pairs `[(x_0, y_0), (x_1, y_1)]`. If one list is longer than the other, the excess is ignored.
* **`List.unzip(l)`**
Takes a list `[(x_0, y_0), (x_1, y_1)]` of type `list((A, B))` and unzips it as two lists `[x_0, x_1, ...]` and `[y_0, y_1, ...]`, of type `list(A)` and `list(B)` respectively.
* **`List.concat(la, lb)`**
Concatenates two lists.

Related implementations
* **[implements `Default`](../std/default_list.mar)** (`forall T, have Default(list(T))`)
The default value of a list is `List.empty`.
* **[implements `Monoid`](../std/monoid_list.mar)** (`forall T, have Monoid(list(T))`)
The set `List(T)` with the concatenation operation forms a monoid. The neutral element for concatenation is `List.empty`, and the operation is `List.concat`. See [`List`](#list-stdlistmar).

<!----------------------------------------------->

## Operators in `ops` ([`std/ops.mar`](../std/ops.mar))
This module defines typeclasses associated with each operator in Marin.

Binary operators
* **`ops.Add(T)`**
    The typeclass tied to the `+` binary operator: **Addition**
    * `Add.op(T, T) => T`

* **`ops.Sub(T)`**
    The typeclass tied to the `-` binary operator: **Subtraction**
    * `Sub.op(T, T) => T`

* **`ops.Mul(T)`**
    The typeclass tied to the `*` binary operator: **Multiplication**
    * `Mul.op(T, T) => T`

* **`ops.Div(T)`**
    The typeclass tied to the `/` binary operator: **Division**
    * `Div.op(T, T) => T`

* **`ops.Mod(T)`**
    The typeclass tied to the `%` binary operator: **Remainder**
    * `Mod.op(T, T) => T`

* **`ops.BitAnd(T)`**
    The typeclass tied to the `&` binary operator: **Bit-wise AND**
    * `BitAnd.op(T, T) => T`

* **`ops.BitOr(T)`**
    The typeclass tied to the `|` binary operator: **Bit-wise OR**
    * `BitOr.op(T, T) => T`

* **`ops.BitXor(T)`**
    The typeclass tied to the `^` binary operator: **Bit-wise XOR**
    * `BitXor.op(T, T) => T`

Unary operators
* **`ops.Pos(T)`**
    The typeclass tied to the `+` unary operator: **Positive**
    * `op(T) => T`

* **`ops.Neg(T)`**
    The typeclass tied to the `-` unary operator: **Negation**
    * `op(T) => T`

* **`ops.BitNeg(T)`**
    The typeclass tied to the `~` unary operator: **Bit-wise NOT**
    * `op(T) => T`

Comparison operators

* **`ops.Eq(T)`**
    The typeclass of types whose values can be checked to be equal or not equal.
    * `eq(T, T) => bool` Tied to the `==` operator: **Equality check**
    * `ne(T, T) => bool` Tied to the `!=` operator: **Inequality check**

* **`ops.Ord(T)`**
    The typeclass of types whose values can be compared according to some defined order.
    * `lt(T, T) => bool` Tied to the `<` operator: **Less-than check**
    * `le(T, T) => bool` Tied to the `<=` operator: **Less-than-or-equal check**
    * `gt(T, T) => bool` Tied to the `>` operator: **Greater-than check**
    * `ge(T, T) => bool` Tied to the `>=` operator: **Greater-than-or-equal check**

**Provided implementations for `int`**
* [`Add(int)`](../std/ops.mar)
* [`Sub(int)`](../std/ops.mar)
* [`Mul(int)`](../std/ops.mar)
* [`Div(int)`](../std/ops.mar)
* [`Mod(int)`](../std/ops.mar)
* [`BitAnd(int)`](../std/ops.mar)
* [`BitOr(int)`](../std/ops.mar)
* [`BitXor(int)`](../std/ops.mar)
* [`Pos(int)`](../std/ops.mar)
* [`Neg(int)`](../std/ops.mar)
* [`BitNeg(int)`](../std/ops.mar)
* [`Eq(int)`](../std/ops.mar)
* [`Ord(int)`](../std/ops.mar)

**Provided implementations for `float`**
* [`Add(float)`](../std/ops.mar)
* [`Sub(float)`](../std/ops.mar)
* [`Mul(float)`](../std/ops.mar)
* [`Div(float)`](../std/ops.mar)
* [`Mod(float)`](../std/ops.mar)
* [`Pos(float)`](../std/ops.mar)
* [`Neg(float)`](../std/ops.mar)
* [`Eq(float)`](../std/ops.mar)
* [`Ord(float)`](../std/ops.mar)

**Provided implementations for `string`**
* [`Add(string)`](../std/ops.mar)
* [`Eq(string)`](../std/ops.mar)
* [`Ord(string)`](../std/ops.mar)

**Provided implementations for `bool`**
* [`BitAnd(bool)`](../std/ops.mar)
* [`BitOr(bool)`](../std/ops.mar)
* [`BitXor(bool)`](../std/ops.mar)
* [`BitNeg(bool)`](../std/ops.mar)
* [`Eq(bool)`](../std/ops.mar)


<!----------------------------------------------->

## `Default` ([`std/default.mar`](../std/default.mar))
A typeclass which allows types to have a "default" value.

Items
* **`Default.default`**
The default value. Reimported as `default` in the prelude.

**Provided implementations
* [`Default(int)`](../std/default_primitives.mar)**
* [`Default(float)`](../std/default_primitives.mar)
* [`Default(string)`](../std/default_primitives.mar)
* [`Default(bool)`](../std/default_primitives.mar)
* [`Default(())`](../std/default_tuples.mar)
* [`Default((T, T))`](../std/default_tuples.mar)
* [`Default((T, T, T))`](../std/default_tuples.mar)
* [`Default((T, T, T, T))`](../std/default_tuples.mar)
* [`Default((T, T, T, T, T))`](../std/default_tuples.mar)
* [`Default((T, T, T, T, T, T))`](../std/default_tuples.mar)
* [`Default((T, T, T, T, T, T, T))`](../std/default_tuples.mar)
* [`Default((T, T, T, T, T, T, T, T))`](../std/default_tuples.mar)

<!----------------------------------------------->

## `Display` ([`std/display.mar`](../std/display.mar))
A typeclass for which types can be converted into a `string` representation.

Items
* **`Display.str(x)`**
The string representation of `x`. Reimported as `str` in the prelude.

**Provided implementations
* [`Display(string)`](../std/display_primitives.mar)**
* [`Display(bool)`](../std/display_primitives.mar)
* [`Display(())`](../std/display_tuples.mar)
* [`Display((T, T))`](../std/display_tuples.mar)
* [`Display((T, T, T))`](../std/display_tuples.mar)
* [`Display((T, T, T, T))`](../std/display_tuples.mar)
* [`Display((T, T, T, T, T))`](../std/display_tuples.mar)
* [`Display((T, T, T, T, T, T))`](../std/display_tuples.mar)
* [`Display((T, T, T, T, T, T, T))`](../std/display_tuples.mar)
* [`Display((T, T, T, T, T, T, T, T))`](../std/display_tuples.mar)

<!----------------------------------------------->

## `Monoid` ([`std/monoid.mar`](../std/monoid.mar))
A monoid is an associative magma with an (unique) identity (or "neutral") element. That is, elements of a monoid type can be manipulated with an associative binary operator `· :: (T, T) => T`, and there exists a unique identity element `e` such that for any `x`, `x · e = e · x = x`.

Implementations of this class must satisfy these mathematical definitions and properties. The compiler cannot verify all of them. Violating any of them is a logic error.

Within the context of the Marin programming language, the monoid operation is called `append`, and the identity element is called `empty`.

Items
* **`Monoid.empty`**
The identity ("neutral") element of the monoid.
* **`Monoid.append(x, y)`**
The stable, associative operation of the monoid.

Additional operations
* **`Monoid.concat(elements)`**
Takes in a [`list`](#list-stdlistmar) of elements `[x_0, x_1, ..., x_n]`, and returns `x_0 · x_1 · ... · x_n`. If the list is empty, by convention, the identity element (`Monoid.empty`) is returned. The order in which the operations is done is unspecified, and does not affect the final returned value.

<!----------------------------------------------->
