# NWPython

A curly-brace, semicolon-terminated Python-like language with C-style operators, transpiled to Python using Rust.

## Features

- **Curly brace syntax**: Use `{}` for code blocks instead of Python's indentation
- **Semicolon termination**: End statements with `;` like C/JavaScript
- **C-style operators**: Support for `i++`, `++i`, `i--`, `--i` increment/decrement
- **Multiple comment styles**: `//`, `#`, and `/* */` multi-line comments
- **Python compatibility**: Generates clean, readable Python code
- **CLI tool**: Transpile and run code with a single command

## Example

### Input (main.nwpy)

```py
// Find factorial using recursion
def factorial(n: int) {
    if (n == 0) {
        return 1;
    }
    return n * factorial(n - 1);
}

/* Multi-line comment
   for demonstration */
def main() {
    x = 5;
    print("Factorial calculation");
    result = factorial(x);
    print(f"factorial of {x} is {result}");

    // C-style increment/decrement
    i = 10;
    print(i++);  // prints 10, then i becomes 11
    print(++i);  // i becomes 12, then prints 12
}

if (__name__ == "__main__") {
    main();
}
```

### Output (main.py)

```python
# Find factorial using recursion
def factorial(n: int):
    if (n == 0):
        return 1
    return n * factorial(n - 1)

"""Multi-line comment
   for demonstration"""
def main():
    x = 5
    print("Factorial calculation")
    result = factorial(x)
    print(f"factorial of {x} is {result}")

    # C-style increment/decrement
    i = 10
    print(i)
    i += 1
    i += 1
    print(i)

if (__name__ == "__main__"):
    main()
```

## Architecture

The project consists of three Rust crates:

### `nwparser`

- **Purpose**: Tokenizes `.nwpy` source code
- **Location**: `nwparser/src/tokenizer.rs`
- **Functionality**:
  - Converts source code into tokens (`Token::LBrace`, `Token::Text`, etc.)
  - Handles comments (`//`, `#`, `/* */`)
  - Manages string literals and quote handling
  - Splits braces, semicolons, and statements correctly

### `nwtranspiler`

- **Purpose**: Converts tokens to Python code
- **Location**: `nwtranspiler/src/transpiler.rs`
- **Functionality**:
  - Manages indentation for block structure
  - Converts `{}` blocks to Python indentation
  - Handles C-style operators (`i++`, `++i`, etc.)
  - Processes comments and converts them to Python format
  - Generates valid Python syntax

### `nwcli`

- **Purpose**: Command-line interface
- **Location**: `nwcli/src/main.rs`
- **Functionality**:
  - Reads `.nwpy` source files
  - Orchestrates parsing and transpilation
  - Writes output `.py` files
  - Optionally runs the generated Python code

## 🛠️ Usage

### Basic transpilation

```bash
cargo run --bin nwcli example.nwpy
# Generates example.py
```

### Transpile and run

```bash
cargo run --bin nwcli example.nwpy --run
# Generates example.py and executes it
```

## Supported Syntax

### Comments

- Single-line: `// comment` or `# comment` → `# comment`
- Multi-line: `/* comment */` → `"""comment"""`

### Increment/Decrement Operators

- `i++` (post-increment) → `i += 1` (standalone) or `return i` (in return statements)
- `++i` (pre-increment) → `i += 1` (standalone) or `i += 1; return i` (in return statements)
- `i--` and `--i` work similarly with `-= 1`
- Special handling for `print(i++)`, `print(++i)`, etc.

### Control Flow

- Block headers: `if`, `elif`, `else`, `def`, `while`, `for`
- Automatic colon insertion: `if (condition) {` → `if (condition):`
- Proper indentation management

## Limitations

- **Interactive input**: `input()` only works when running the generated `.py` file in a real terminal
- **Not a full parser**: Uses tokenization + regex, not a complete AST
- **Python target only**: Currently only transpiles to Python
- **Edge cases**: Some complex expressions may not be handled correctly

## 🔧 Development

### Build

```bash
cargo build
```

### Run tests

```bash
cargo test
```

### Lint

```bash
cargo clippy
```

### Project Structure

```
nwpython/
├── Cargo.toml          # Workspace configuration
├── nwparser/           # Tokenizer crate
│   ├── src/
│   │   ├── lib.rs      # Re-exports
│   │   └── tokenizer.rs # Tokenization logic
│   └── Cargo.toml
├── nwtranspiler/       # Transpiler crate
│   ├── src/
│   │   ├── lib.rs      # Re-exports
│   │   └── transpiler.rs # Transpilation logic
│   └── Cargo.toml
├── nwcli/              # CLI binary crate
│   ├── src/
│   │   └── main.rs     # CLI implementation
│   └── Cargo.toml
└── example.nwpy        # Example source file
```

## How It Works

1. **Tokenization**: The `nwparser` crate breaks your `.nwpy` source into tokens
2. **Transpilation**: The `nwtranspiler` crate converts tokens to Python code
3. **Code Generation**: Handles indentation, operators, and comment conversion
4. **Output**: Produces clean, readable Python that maintains the original logic

## Contributing

Contributions are welcome! This is a learning project exploring language design and implementation.

## License

GPLV3 License - see LICENSE file for details.
