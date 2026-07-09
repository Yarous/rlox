# План Разделения Структуры Проекта

Цель: разрезать крупные файлы (`interpreter.rs`, `parser.rs`, `scanner.rs`) по зонам ответственности, чтобы проект было удобно развивать дальше без файлов на 1000 строк.

Предпочтительный стиль модулей: классический Rust-style через `folder/mod.rs`.

```text
src/frontend/mod.rs
src/frontend/scanner/mod.rs
src/frontend/parser/mod.rs
```

Такой стиль хорошо читается в больших деревьях: папка является модулем, а `mod.rs` описывает ее публичную поверхность и подключает дочерние файлы.

## Целевая Структура

```text
src/
  main.rs
  lib.rs

  app/
    mod.rs
    cli.rs
    diagnostics.rs
    lox.rs
    repl.rs
    runner.rs

  frontend/
    mod.rs
    token.rs
    scanner/
      mod.rs
      error.rs
      tests.rs
    parser/
      mod.rs
      declarations.rs
      error.rs
      expressions.rs
      statements.rs
      tests.rs

  ast/
    mod.rs
    display.rs
    expr.rs
    stmt.rs

  runtime/
    mod.rs
    callable/
      mod.rs
      function.rs
      lambda.rs
      native.rs
    control_flow.rs
    environment.rs
    error.rs
    interpreter.rs
    value.rs

tests/
  cli.rs
```

## Главный Принцип

Разделять не по количеству строк, а по ownership:

- `frontend`: превращает source text в AST.
- `ast`: хранит структуры языка.
- `runtime`: исполняет AST.
- `app`: CLI, REPL, диагностика, exit codes, IO.
- `main.rs`: тонкая точка входа.

Идеальный pipeline:

```text
source
  -> scanner
  -> tokens
  -> parser
  -> AST statements
  -> interpreter
  -> runtime result
  -> app diagnostics / exit code
```

## `main.rs` И `lib.rs`

`main.rs` должен стать минимальным:

```rust
fn main() {
    rlox::app::cli::run();
}
```

`lib.rs` должен открыть внутреннюю структуру crate:

```rust
pub mod app;
pub mod ast;
pub mod frontend;
pub mod runtime;
```

Публичность лучше держать консервативной: внутри модулей использовать `pub(crate)`, а наружу открывать только то, что реально нужно CLI, integration tests или будущему library API.

## `frontend`

`frontend` отвечает за scan/parse pipeline.

```text
src/frontend/mod.rs
src/frontend/token.rs
src/frontend/scanner/mod.rs
src/frontend/scanner/error.rs
src/frontend/scanner/tests.rs
src/frontend/parser/mod.rs
src/frontend/parser/error.rs
src/frontend/parser/declarations.rs
src/frontend/parser/statements.rs
src/frontend/parser/expressions.rs
src/frontend/parser/tests.rs
```

`src/frontend/mod.rs`:

```rust
pub(crate) mod parser;
pub(crate) mod scanner;
pub(crate) mod token;
```

`scanner/mod.rs`:

- `Scanner`
- `scan_tokens`
- byte cursor helpers
- token emitting helpers

`scanner/error.rs`:

- `ScanError`
- `ScanResult`

`parser/mod.rs`:

- `Parser`
- parser state fields
- cursor helpers: `peek`, `advance`, `previous`, `check`, `switch` или будущий `match_any`
- `parse`
- `synchronize`
- подключение parser submodules

`parser/error.rs`:

- `ParseError`
- `ParseResult`

`parser/declarations.rs`:

- `declaration`
- `var_declaration`
- `function_statement`
- `parse_function_parameters`

`parser/statements.rs`:

- `statement`
- `print_statement`
- `expression_statement`
- `block`
- `if_statement`
- `while_statement`
- `for_statement`
- `break_statement`
- `return_statement`

`parser/expressions.rs`:

- `expression`
- `comma`
- `assignment`
- `ternary`
- `or`
- `and`
- `equality`
- `comparison`
- `term`
- `factor`
- `unary`
- `call`
- `finish_call`
- `primary`

Важно: один `impl Parser<'_>` можно спокойно разложить по нескольким файлам.

Пример:

```rust
// src/frontend/parser/mod.rs
mod declarations;
mod error;
mod expressions;
mod statements;

pub(crate) struct Parser<'tokens> {
    tokens: &'tokens [Token],
    current: usize,
    errors: Vec<ParseError<'tokens>>,
    loop_depth: usize,
    function_depth: usize,
}
```

```rust
// src/frontend/parser/expressions.rs
impl<'tokens> Parser<'tokens> {
    pub(super) fn expression(&mut self) -> Result<Expr, ParseError<'tokens>> {
        self.comma()
    }
}
```

## `ast`

AST лучше держать отдельно от frontend/runtime.

```text
src/ast/mod.rs
src/ast/expr.rs
src/ast/stmt.rs
src/ast/display.rs
```

`src/ast/mod.rs`:

```rust
pub(crate) mod display;
pub(crate) mod expr;
pub(crate) mod stmt;

pub(crate) use expr::{Expr, Op};
pub(crate) use stmt::Stmt;
```

`expr.rs`:

- `Op`
- `Expr`
- `impl From<&Token> for Op`

`stmt.rs`:

- `Stmt`

`display.rs`:

- `impl Display for Expr`
- `impl Display for Stmt`
- shared formatting helpers

Причина вынести display отдельно: data definitions остаются короткими и читаемыми, а formatting не раздувает AST-файлы.

## `runtime`

Сейчас `interpreter.rs` смешивает values, errors, callables, native functions, closures и execution. Его нужно резать первым.

```text
src/runtime/mod.rs
src/runtime/interpreter.rs
src/runtime/environment.rs
src/runtime/value.rs
src/runtime/error.rs
src/runtime/control_flow.rs
src/runtime/callable/mod.rs
src/runtime/callable/native.rs
src/runtime/callable/function.rs
src/runtime/callable/lambda.rs
```

`src/runtime/mod.rs`:

```rust
pub(crate) mod callable;
pub(crate) mod control_flow;
pub(crate) mod environment;
pub(crate) mod error;
pub(crate) mod interpreter;
pub(crate) mod value;

pub(crate) use interpreter::Interpreter;
```

`value.rs`:

- `Value`
- `ValueType`
- `impl Display for Value`
- `impl PartialEq for Value`
- `impl From<Literal> for Value`

`error.rs`:

- `RuntimeError`
- `RuntimeErrorKind`

Текущее имя `ValidationError` лучше заменить на `RuntimeErrorKind`, потому что ошибки возникают во время evaluation, а не static validation.

`control_flow.rs`:

- `ControlFlow`
- `RuntimeError(RuntimeError)`
- `Break`
- `Return(Value)`

`environment.rs`:

- `Environment`
- `EnvRef`
- `define`
- `get`
- `assign`

`callable/mod.rs`:

- trait `LoxCallable`
- re-export callable implementations

`callable/native.rs`:

- `Clock`

`callable/function.rs`:

- `LoxFunction`

`callable/lambda.rs`:

- `LoxLambda`

`interpreter.rs`:

- `Interpreter`
- `interpret`
- `execute`
- `evaluate`
- statement evaluation methods
- expression evaluation methods
- operator/type helpers

Long-term улучшение: убрать зависимость `Interpreter -> Lox`. Интерпретатор должен возвращать `Result`, а `app` слой должен решать, как печатать ошибку и какой exit code выставлять.

## `app`

`app` слой должен быть единственным местом, где есть CLI, IO, stderr/stdout policy, REPL и exit codes.

```text
src/app/mod.rs
src/app/cli.rs
src/app/diagnostics.rs
src/app/lox.rs
src/app/repl.rs
src/app/runner.rs
```

`app/mod.rs`:

```rust
pub mod cli;
pub(crate) mod diagnostics;
pub(crate) mod lox;
pub(crate) mod repl;
pub(crate) mod runner;
```

`cli.rs`:

- `Args`
- `clap::Parser`
- `run`
- mapping `had_error -> exit 65`, `had_runtime_error -> exit 70`

`lox.rs`:

- `Lox`
- `had_error`
- `had_runtime_error`
- small state transitions

`runner.rs`:

- `run_source`
- scanner/parser/interpreter orchestration

`repl.rs`:

- prompt loop
- persistent interpreter/global environment
- resetting per-line error flags

`diagnostics.rs`:

- scanner error formatting
- parser error formatting
- runtime error formatting

## Миграция Без Боли

Не нужно одновременно перемещать файлы и менять поведение. Лучший порядок:

1. Добавить `src/lib.rs`.
2. Перенести module declarations из `main.rs` в `lib.rs`.
3. Сделать `main.rs` thin wrapper.
4. Перенести `token.rs` в `frontend/token.rs`.
5. Перенести `expr.rs` и `stmt.rs` в `ast/expr.rs`, `ast/stmt.rs`.
6. Вынести `Display` impl-ы в `ast/display.rs`.
7. Перенести `environment.rs` в `runtime/environment.rs`.
8. Разрезать `interpreter.rs` на `runtime/value.rs`, `runtime/error.rs`, `runtime/control_flow.rs`, `runtime/callable/*`, `runtime/interpreter.rs`.
9. Разрезать `parser.rs` на `frontend/parser/mod.rs` и `frontend/parser/*.rs`.
10. Разрезать `scanner.rs` на `frontend/scanner/mod.rs`, `frontend/scanner/error.rs`, `frontend/scanner/tests.rs`.
11. Вынести CLI/REPL/runner из `main.rs` в `app/*`.
12. После каждого шага запускать:

```bash
cargo fmt --check
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features
```

## Что Не Делать

- Не резать файлы одновременно с изменением semantics.
- Не делать premature public API.
- Не оставлять `main.rs` владельцем всех модулей.
- Не переносить тесты в integration tests, если им нужен private API. Для scanner/parser/interpreter unit-тесты рядом с модулем нормальны.
- Не смешивать `src/foo.rs` и `src/foo/mod.rs` для одного и того же модуля.

## Ориентиры По Размерам

Нормальные размеры после разрезания:

```text
frontend/parser/mod.rs              80-140 строк
frontend/parser/expressions.rs     250-350 строк
frontend/parser/statements.rs      150-250 строк
frontend/parser/declarations.rs     80-150 строк
frontend/scanner/mod.rs            250-350 строк
runtime/interpreter.rs             250-400 строк
runtime/value.rs                    80-140 строк
runtime/callable/*.rs               80-180 строк
app/*.rs                            50-150 строк
```

Файл на 300 строк может быть нормальным, если он отвечает за одну цель. Файл на 1000 строк плох не самим числом, а тем, что в нем смешаны разные уровни: data model, errors, execution, IO policy, native functions и tests.
