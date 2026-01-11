# Blood 🩸

Blood is a simple, interpreted programming language written in Rust. It's designed to be fast, readable, and strict about mutability.

The syntax borrows from Lua (`do ... end` blocks) but enforces Rust-like discipline when it comes to variables. If you don't explicitly say a variable can change, it won't.

## Features

- **Strict Mutability**: Variables are immutable by default (`let`). You must use `let mod` to allow reassignment.
- **Control Flow**: `if`, `elseif`, `while`, `loop` (infinite), `break`, and `continue`.
- **Functions**: First-class support for functions with isolated scope and recursion.
- **Clean Syntax**: No semicolons required. Block-based structure using `do` / `then` / `end`.
- **Comments**: Standard C-style `//` for single lines and `/* ... */` for blocks.
- **Error Handling**: Clear, human-readable runtime errors (no scary compiler panics).

## Getting Started

### Prerequisites

You'll need [Rust](https://www.rust-lang.org/) installed on your machine.

### Building and Running

Clone the repo and run the interpreter using Cargo:

```bash
# Run a specific file
cargo run -- example/primes.bd

# Run the test suite (all features)
cargo run -- example/all_features.bd
```

## Syntax Guide

### Variables

By default, once you set a value, it stays set.

```blood
let x = 10
# x = 20  // This will crash the program!

let mod y = 5
y = y + 1  // This is fine.
print(y)
```

### Control Flow

We use `then` and `do` keywords to keep things readable.

**Conditionals:**
```blood
if x > 10 then
    print(1)
elseif x == 10 then
    print(2)
else
    print(3)
end
```

**Loops:**
```blood
let mod i = 0
while i < 5 do
    print(i)
    i = i + 1
end

# Infinite loop
loop do
    print(1)
    break
end
```

### Functions

Functions define their own scope. Arguments are passed by value and are immutable inside the function.

```blood
fn add(a, b) do
    return a + b
end

print(add(10, 20))
```

Recursive functions work as expected:

```blood
fn fact(n) do
    if n == 0 then
        return 1
    end
    return n * fact(n - 1)
end
```

### Comments

```blood
// This is a comment
print(1) // Inline comment

/* 
   This is a
   multi-line comment
*/
```

## Project Structure

- `src/main.rs`: Entry point.
- `src/lexer.rs`: Tokenizer.
- `src/parser.rs`: Recursive descent parser.
- `src/ast.rs`: Abstract Syntax Tree definitions.
- `src/interpreter.rs`: The tree-walk interpreter and environment logic.

## License

This project is open-source and available under the [MIT License](LICENSE).
