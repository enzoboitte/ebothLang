# Eboth Programming Language

A stack-based programming language that compiles to x86-64 assembly and features a built-in interpreter for rapid development and testing.

## Description

Eboth is a minimalist stack-based programming language designed for low-level system programming with direct access to Linux syscalls. The language follows a postfix notation where operations are performed on a data stack, similar to Forth but with a modern syntax.

The compiler generates native x86-64 assembly code that can be assembled and linked into standalone executables, providing excellent performance and direct control over system resources. Additionally, Eboth includes an integrated interpreter that allows immediate code execution without the compilation step, making it ideal for prototyping and learning.

### Key Features

- **Stack-based architecture**: All operations work with an implicit data stack
- **Native compilation**: Compiles directly to x86-64 assembly (NASM syntax)
- **Built-in interpreter**: Execute code instantly without compilation
- **Linux syscall interface**: Direct access to syscalls (syscall0 through syscall6)
- **Procedure definitions**: Modular code organization with `proc` blocks
- **Constant definitions**: Define reusable constant expressions with `const`
- **Zero-overhead abstractions**: Procedures and constants are inlined during compilation

## Prerequisites

### Required Software

- **Rust** (2024 edition or later) - [Install Rust](https://www.rust-lang.org/tools/install)
- **NASM** - For assembling the generated code
  ```bash
  # Ubuntu/Debian
  sudo apt install nasm
  
  # Arch Linux
  sudo pacman -S nasm
  
  # macOS
  brew install nasm
  ```
- **GNU Linker (ld)** - Usually pre-installed on Linux systems

### Building the Compiler

```bash
# Clone or navigate to the project directory
cd eboth

# Build the compiler
cargo build --release

# The executable will be available at target/release/eboth
```

## Execution

### Quick Start

Eboth processes source files (`.eb` extension) and outputs both interpreted results and compiled assembly.

```bash
# Run the compiler and interpreter on a source file
cargo run ./example/basic.eb

# Assemble and run the compiled output
./run.sh ./out.asm
```

### Step-by-Step Execution

1. **Compile Eboth source to assembly:**
   ```bash
   cargo run ./example/basic.eb
   ```
   This generates `out.asm` containing x86-64 assembly code.

2. **Assemble and link:**
   ```bash
   nasm -f elf64 out.asm -o out.o
   ld out.o -o program
   ```

3. **Execute the binary:**
   ```bash
   ./program
   ```

Alternatively, use the provided shell script:
```bash
./run.sh ./out.asm
```

## Language Features

### ✅ Implemented Features

#### Core Operations
- **Literals**
  - Integer literals: `42`, `-17`, `0`
  - String literals: `"Hello, World!\n"`

- **Arithmetic Operations**
  - `+` - Addition (pops two values, pushes sum)
  - `-` - Subtraction (pops two values, pushes difference)
  - `*` - Multiplication
  - `/` - Division
  - `%` - Modulo

- **Stack Manipulation**
  - `dup` - Duplicate top stack value
  - `swap` - Swap top two stack values
  - `dump` - Pop and print integer from stack
  - `puts` - Pop and print string from stack

- **System Calls**
  - `syscall` - Syscall with 0 arguments
  - `syscall1` - Syscall with 1 argument
  - `syscall2` - Syscall with 2 arguments
  - `syscall3` - Syscall with 3 arguments (e.g., write)
  - `syscall4` - Syscall with 4 arguments
  - `syscall5` - Syscall with 5 arguments
  - `syscall6` - Syscall with 6 arguments

#### Program Structure
- **Procedures**: `proc [name] in ... end`
  - Define reusable code blocks
  - Automatic return handling
  - Procedure calls by name
  
- **Constants**: `const [name] in ... end`
  - Define constant expressions
  - Evaluated at compile/interpret time
  - Invoked like procedures

#### Type System (Partial)
- Type annotations: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `f32`, `f64`, `ptr`, `str`, `bool`, `void`
- Type casting infrastructure (defined but not fully integrated in parser)

#### Comments
- Line comments: `# This is a comment`

### Execution Modes

#### Interpreter Mode
- Executes code immediately using a virtual stack
- Perfect for testing and debugging
- Supports all language features
- Automatic output to stdout

#### Compiler Mode
- Generates optimized x86-64 assembly
- Produces standalone executables
- Uses system stack for efficient execution
- Full syscall support

## Language Documentation

### Basic Syntax

Eboth uses postfix notation where operands come before operators:

```eboth
# Traditional: 2 + 3
# Eboth:
2 3 +
```

### Hello World

```eboth
proc main in
    1 1 "Hello World!\n" syscall3
end
```

**Explanation:**
- `1` - File descriptor (stdout)
- `1` - Syscall number for write (on x86-64 it's actually 1)
- `"Hello World!\n"` - String to print
- `syscall3` - Invoke syscall with 3 arguments

### Stack Operations

```eboth
proc main in
    10 20 swap dump dump  # Prints: 10 20
    42 dup + dump         # Prints: 84 (42 * 2)
end
```

**Stack evolution:**
```
10 20 swap → 20 10
20 10 dump → 20 (prints 10)
20 dump → (prints 20)

42 dup → 42 42
42 42 + → 84
84 dump → (prints 84)
```

### Procedures

Procedures allow code reuse and modularity:

```eboth
proc square in
    dup *
end

proc main in
    5 square dump  # Prints: 25
end
```

### Constants

Constants define reusable expressions evaluated at compile-time:

```eboth
proc PI in 314159 end
proc E in 271828 end

proc main in
    PI E + dump  # Prints: 585987
end
```

### Arithmetic Example

```eboth
proc main in
    # Calculate (10 + 20) * 3
    10 20 + 3 * dump  # Prints: 90
    
    # Calculate 100 / 4
    100 4 / dump      # Prints: 25
end
```

### String Output

```eboth
proc main in
    "Result: " puts
    42 dump
    "\n" puts
end
```

**Output:** `Result: 42`

### Using Syscalls

Direct system call interface for advanced operations:

```eboth
proc main in
    # syscall3 for write: write(fd, buf, count)
    1              # stdout
    1              # syscall number (write)
    "Hello!\n"     # buffer (string pointer + length handled internally)
    syscall3
    dump           # Print return value (bytes written)
end
```

### Complete Example

```eboth
proc STDOUT in 1 end
proc WRITE in 1 end

proc print_number in
    # Expects number on stack
    dump
    "\n" puts
end

proc add_and_print i64 i64 ret i64 in
    +
    dup print_number
end

proc main in
    "Calculating 15 + 27:\n" puts
    15 27 add_and_print
    "Done!\n" puts
end
```

### Advanced: Procedure Composition

```eboth
proc double in
    2 *
end

proc triple in
    3 *
end

proc add_six in
    6 +
end

proc main in
    10 double triple add_six dump  # ((10 * 2) * 3) + 6 = 66
end
```

## Project Structure

```
eboth/
├── src/
│   ├── main.rs      # Entry point, IR definitions, and interpreter
│   ├── syntax.rs    # Lexer and parser implementation
│   └── engine.rs    # x86-64 assembly code generator
├── example/         # Example programs
│   ├── basic.eb     # Hello World
│   ├── functions.eb # Procedure examples
│   ├── consts.eb    # Constants usage
│   ├── example.eb   # General examples
│   └── manipulation.eb  # Stack operations
├── Cargo.toml       # Rust project configuration
├── run.sh           # Build and execute script
└── README.md        # This file
```

## Architecture

### Modules

- **main.rs**: Contains the IR enum (`EIrInstr`), type definitions (`EType`), interpreter (`CStackToInterpreter`), and orchestrates the compilation pipeline
- **syntax.rs**: Implements the lexer (`CLexer`) and parser (`CParser`) that transform source code into IR
- **engine.rs**: Contains `StackAsmBuilder` for assembly generation and `StackCompiler` for compiling IR to x86-64 assembly

## Implementation Notes

### Memory Model

- **Data stack (r15)**: Primary data stack for all operations, grows downward
- **Procedure stack**: Callee-saved r15 via push/pop for isolated stack frames
- **String pool**: Strings stored in `.data` section with automatic labeling (`str_0`, `str_1`, etc.)

### Calling Convention

Procedures use the following convention:
1. Caller pushes arguments onto the data stack
2. Callee saves r15 (stack pointer) at entry
3. Callee operates on its local stack view
4. On return: if procedure returns a value, it replaces the top of the caller's stack
5. For void procedures: stack is simply restored to caller's state

### Assembly Output

Generated assembly includes:
- `.bss` section: Data stack buffer (4096 qwords) and dump buffer
- `.data` section: String literals with null termination
- `.text` section: Helper functions (`dump_i`, `dump_str`) and procedure definitions
- Entry point `_start`: Initializes stack pointer and calls `proc_main`

## Examples

See the `example/` directory for more code samples:

- **basic.eb**: Simple Hello World using syscalls
- **functions.eb**: Procedure definitions and calls
- **consts.eb**: Using constants for code organization
- **manipulation.eb**: Stack operation demonstrations
- **test.eb**: Various language features

## Development

### Running Tests

```bash
# Test with interpretation and compilation
cargo run ./example/test.eb

# Run the compiled version
./run.sh ./out.asm
```

### Debugging

The compiler outputs three stages:
1. **IR (Intermediate Representation)**: Parsed instruction stream
2. **Interpretation**: Direct execution with output
3. **Compilation**: Generated x86-64 assembly code

## Limitations & Future Work

- [ ] Type system is defined but not enforced during parsing
- [ ] No control flow structures (if/else/while/for) yet
- [ ] No local variables (only stack operations)
- [ ] Limited error messages with line numbers
- [ ] No optimization passes
- [ ] Linux x86-64 only (no Windows/macOS support)
- [ ] No standard library

## License

This project is available for educational and personal use.

## Author

Enzo Boitte

---

**Note**: Eboth is an experimental language designed for learning compiler construction and low-level programming concepts. Use in production environments is not recommended.
