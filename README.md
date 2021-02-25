# Rusp
A strange mix of Rust and Lisp. Expression-oriented. Interpreted.

## Features
- Everything is an expression (kinda)
  - Statements are expressions that evaluate to `()`, the Unit type.
- Integers, floats, strings, booleans (`true` and `false`)
- Variable declaration `let x = 42`
- Variable assignment `x = 69.69`
- Scopes `{ let a = 5 let b = 10 (+ a b) }`
  - The value of a scope is the value of its last expression, like in Rust.
- Named functions `fn add (x y) (+ x y)`
- Anonymous functions `fn (x y) (+ x y)`
- All functions are closures
- Arithmetic operators: `+`, `-`, `*`, `/`, `**`
- Comparaison operators: `<`, `<=`, `=`, `>=`, `>`
- Implicit integer to float to string casting
  - `(= (+ 1 5.8 "da") "6.8da")`
  - `(= (+ "da" 5.8 1) "da5.81")`
- Conditions
  - `if (< x y) (print x)`, always evaluates to `()`
  - `if (< x y) x else y`, evaluates to the branching expression
- While loops `while (< i 5) i = (+ i 1)`
  - Always evaluate to `()`
- Single line comments with `//`
- Some built-in functions: `print`, `println` and `dbg`
- No need for a main function
- Indentation doesn't matter, you can write everything on a single line if you wish (please don't)

## Examples
```rust
fn max (x y) if (> x y) x else y

fn sum (from to) {
    let sum = 0
    let i = from
    while (<= i to) {
        sum = (+ sum i)
        i = (+ i 1)
    }
    sum
}

let is_greater_than = fn (x) fn (y) (> y x)
let is_greater_than_4 = (is_greater_than 4)

(println ((is_greater_than 10) 11)) // true
(println (is_greater_than_4 3)) // false

fn recursive_loop (n f) {
    if (> n 0) {
        (f)
        (recursive_loop (- n 1) f)
    }
}

let arg = "yes"
(recursive_loop 3 fn () {
    (print (+ arg " "))
    arg = (+ arg "s")
})
// yes yess yesss
```

## TODO
- Logic operators
- Meaningful errors
- Lists and built-in functions for lists
- A mini standard library
- Custom types
- Do less variable cloning

## Usage
Compile with Rust Nightly. `cargo run -- examples.rsp`