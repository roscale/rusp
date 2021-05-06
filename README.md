# Rusp
A strange mix of Rust and Lisp. Expression-oriented. Compiles to Java bytecode.

### The JVM compiler is very much a work in progress.

## TODO
- Modify the language syntax to be able to specify types
- Basic constructs: variables, scopes, conditions, loops, etc.
- Infer types when possible
- Have feature parity with the current interpreter

## Usage
Compile with Rust Nightly. `cargo run -- examples.rsp && java -noverify Main`