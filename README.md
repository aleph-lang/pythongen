# pythongen

Generates Python source code from an [`AlephTree`](https://github.com/aleph-lang/aleph-syntax-tree).

## Installation

```toml
[dependencies]
pythongen = "0.1"
```

## Usage

```rust
let code = pythongen::generate(ast);
```

## Example

```rust
let ast = AlephTree::If {
    condition: Box::new(AlephTree::Bool { value: "true".into() }),
    then: Box::new(AlephTree::Int { value: "1".into() }),
    els: Box::new(AlephTree::Int { value: "0".into() }),
};
let code = pythongen::generate(ast);
// produces:
// if(true):
//     1
// else:
//     0
```

## Supported constructs

Literals, arithmetic, boolean logic, `if/else`, `while`, `let`, `match`, arrays, tuples, functions (`LetRec`, `App`), `return`, `import`, `assert`.

## Related

- [`aleph-syntax-tree`](https://github.com/aleph-lang/aleph-syntax-tree) — AST definition
- [`alephc`](https://github.com/aleph-lang/aleph) — uses this generator with `--features python_gen`
- [`aleph2py`](https://github.com/aleph-lang/aleph) — standalone Aleph → Python binary
