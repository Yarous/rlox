# Rlox

A tree-walking interpreter for Lox, written in Rust.

Lox is the language from Bob Nystrom's book *Crafting Interpreters*. This is my
implementation of it — mostly a way to actually understand how parsers and
interpreters work under the hood, rather than just reading about it.

## What's implemented so far

- Expression and statement parsing (recursive descent)
- Functions, including closures
- Loops (`while`, `for`)
- Ternary expressions (`cond ? a : b`)
- Control flow: `if`/`else`, `break`, `return`

Still missing: classes, a standard library, and any kind of bytecode VM. Right
now it just walks the AST directly, so don't expect it to be fast.

## Building it

You need Rust installed (stable is fine). Then:

```
git clone https://github.com/<your-username>/rlox.git
cd rlox
cargo build --release
```

## Running it

Without arguments it drops you into a REPL:

```
cargo run
```

Or give it a script:

```
cargo run -- script.lox
```

## Example

```lox
fun fib(n) {
  if (n <= 1) return n;
  return fib(n - 1) + fib(n - 2);
}

for (var i = 0; i < 10; i = i + 1) {
  print fib(i);
}

var result = 10 > 5 ? "ten is bigger" : "five is bigger";
print result;
```

## Why

Mainly to learn. Parsers and interpreters look mysterious from the outside,
and writing one — even a slow, tree-walking one — clears that up fast. Rust
also forces you to think more carefully about ownership of the AST and
environment than you would in, say, Java or Python, which was part of the
appeal.

## What's next

Classes and inheritance are the obvious next step, since the book covers them
and I haven't gotten there yet. After that, probably some basic standard
library functions and better error messages (right now they're not great).
A bytecode VM is tempting but that's really a separate project at that point.

## License

MIT.
