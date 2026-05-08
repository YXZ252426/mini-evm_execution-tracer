# Rust Formatting and `?` Notes

This note summarizes the Rust formatting syntax used in this project, plus the difference between formatting `?` and the error propagation `?` operator.

## `format!`

`format!` creates a `String`.

```rust
let opcode = 0x60;
let opcode_hex = format!("0x{opcode:02x}");
```

Result:

```text
0x60
```

Format string breakdown:

```text
0x      literal text
{...}   placeholder
opcode  variable name
:02x    formatting rule
```

`02x` means:

```text
0   pad with zeroes
2   minimum width is 2 characters
x   lowercase hexadecimal
```

Examples:

```rust
format!("0x{5:02x}")    // "0x05"
format!("0x{96:02x}")   // "0x60"
format!("0x{171:02X}")  // "0xAB"
```

## `println!`

`println!` uses the same formatting syntax as `format!`, but writes to stdout and adds a newline.

```rust
println!("gas used: {}", gas_used);
println!("opcode: 0x{opcode:02x}");
println!("step: {step:?}");
println!("step: {step:#?}");
```

Related macros:

```rust
format!    // creates String
println!   // prints to stdout with newline
print!     // prints to stdout without newline
eprintln!  // prints to stderr with newline
```

## Common Formatting Forms

```rust
format!("{x}")       // Display
format!("{x:?}")     // Debug
format!("{x:#?}")    // pretty Debug
format!("{x:x}")     // lowercase hex, no 0x prefix
format!("{x:X}")     // uppercase hex, no 0x prefix
format!("{x:#x}")    // lowercase hex with 0x prefix
format!("{x:#X}")    // uppercase hex with 0x prefix
format!("{x:02x}")   // lowercase hex, width 2, zero padded
format!("0x{x:02x}") // manual 0x prefix, useful for opcode bytes
```

## `{}` vs `{:?}`

`{}` uses the `Display` trait.

```rust
println!("{}", gas_used);
```

`{:?}` uses the `Debug` trait.

```rust
println!("{:?}", steps);
```

Use `{}` for user-facing values.

Use `{:?}` or `{:#?}` for development/debugging output.

Example:

```rust
let values = vec![1, 2, 3];

println!("{:?}", values);  // [1, 2, 3]
println!("{:#?}", values);
```

Pretty Debug output:

```text
[
    1,
    2,
    3,
]
```

Many collection types, such as `Vec<T>`, implement `Debug` but not `Display`, so this fails:

```rust
println!("{}", values);
```

## Hex Formatting in the Tracer

Opcode byte:

```rust
opcode_hex: format!("0x{opcode:02x}")
```

This is good for opcodes because an opcode is one byte. It should always display as two hex digits:

```text
0x00
0x55
0x60
```

Stack value:

```rust
format!("{value:#x}")
```

This is good for EVM stack values because they are `U256` values and do not need fixed two-digit width:

```text
0x0
0x1
0xffff
```

Current project example:

```rust
let opcode = interp.bytecode.opcode();
let stack_top = interp
    .stack
    .data()
    .iter()
    .rev()
    .take(self.record_stack_top)
    .map(|value| format!("{value:#x}"))
    .collect();

self.steps.push(StepTrace {
    opcode_hex: format!("0x{opcode:02x}"),
    stack_top,
    gas_remaining: interp.gas.remaining(),
    memory_size: interp.memory.len(),
    // ...
});
```

Keep numeric fields numeric when JSON consumers may want to sort, filter, or calculate with them:

```rust
gas_remaining: interp.gas.remaining(), // u64, not String
memory_size: interp.memory.len(),      // usize, not String
```

Use formatted strings when the representation itself matters:

```rust
opcode_hex: format!("0x{opcode:02x}"),
stack_top: vec![format!("{value:#x}")],
```

## Two Different Meanings of `?`

Rust uses `?` in two unrelated ways.

### Debug Formatting

Inside a format string:

```rust
format!("{value:?}")
println!("{value:?}");
```

This means: format with the `Debug` trait.

Pretty Debug:

```rust
println!("{value:#?}");
```

### Error Propagation

Outside a format string:

```rust
let bytes = read_hex_file(path)?;
```

This means: if the result is `Ok(value)`, unwrap it; if it is `Err(error)`, return early from the current function.

This:

```rust
let bytes = read_hex_file(path)?;
```

is roughly equivalent to:

```rust
let bytes = match read_hex_file(path) {
    Ok(value) => value,
    Err(error) => return Err(error.into()),
};
```

The `?` operator can be used when the current function returns `Result`, `Option`, or another compatible type.

Example:

```rust
use eyre::Result;

fn load_contract(path: &Path) -> Result<Vec<u8>> {
    let bytes = read_hex_file(path)?;
    Ok(bytes)
}
```

## `wrap_err(...)?`

In this project, we often use `eyre::Context` with `?`:

```rust
let call_data = parse_hex(call_data).wrap_err("failed to parse calldata hex")?;
```

Meaning:

```text
parse_hex(call_data)
  Ok(value)  -> use value
  Err(error) -> attach context message
                return Err(...)
```

This is better than a bare `?` for CLI tools because the user sees where the error happened.

Bare `?`:

```rust
let call_data = parse_hex(call_data)?;
```

Contextual `?`:

```rust
let call_data = parse_hex(call_data).wrap_err("failed to parse calldata hex")?;
```

Prefer contextual errors at boundaries:

```rust
let contract_bytes = read_hex_file(contract_path).wrap_err_with(|| {
    format!(
        "failed to read contract bytecode from {}",
        contract_path.display()
    )
})?;
```

## Quick Reference

```rust
format!("{x}")          // Display
format!("{x:?}")        // Debug
format!("{x:#?}")       // pretty Debug
format!("{x:x}")        // hex
format!("{x:#x}")       // hex with 0x
format!("{x:02x}")      // hex, width 2, zero padded
format!("0x{x:02x}")    // opcode-style hex

println!("{x}")         // print Display
println!("{x:?}")       // print Debug
println!("{x:#?}")      // print pretty Debug

some_result?            // unwrap Ok or return Err
some_result.wrap_err("context")? // add context, then propagate error
```

