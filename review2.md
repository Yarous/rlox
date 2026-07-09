# Review: lambda expressions

Короткий вывод: направление в целом правильное, но текущая реализация свернула в неверную семантику на этапе интерпретации и сейчас даже не компилируется.

Ты правильно начал с того, что:

- добавляешь `Expr::Lambda` как выражение, а не как statement;
- переиспользуешь `fun (...) { ... }` синтаксис;
- кладешь тело лямбды в `Vec<Stmt>`, как у обычной функции;
- пытаешься переиспользовать уже существующий механизм `LoxCallable`;
- захватываешь `closure` при создании callable-объекта.

Главная ошибка: лямбда-выражение не должно сразу вызываться.

Сейчас `evaluate_lambda_expression()` создает `LoxLambda`, потом пытается вычислить `parameters` как аргументы и сразу вызывает `lambda.call(...)`. Это неправильная модель.

Правильная модель такая:

```lox
var f = fun (a) {
  return a + 1;
};

print f(2);
```

Когда интерпретатор вычисляет выражение `fun (a) { ... }`, результатом должен быть `Value::Callable(...)`. Сам вызов должен происходить только позже, когда парсер/интерпретатор видит `(...)` после callee. У тебя это уже есть в `evaluate_call_expression()`.

То есть `Expr::Lambda` должен работать примерно как function literal:

- `fun (a) { return a + 1; }` -> создает callable value;
- `fun (a) { return a + 1; }(2)` -> сначала создает callable value, потом обычный `Expr::Call` вызывает его с аргументом `2`;
- `var f = fun (a) { ... }; f(2);` -> переменная хранит callable.

## Конкретные проблемы

1. `Expr::Lambda` хранит параметры как `Vec<Expr>`, а параметры функции должны быть `Vec<Token>`.

В `src/expr.rs` сейчас:

```rust
Lambda {
    parameters: Vec<Expr>,
    paren: Token,
    _body: Vec<Stmt>,
}
```

Но `parse_function_parameters()` возвращает `Vec<Token>`, и это правильно. Параметр - это имя, то есть токен identifier, а не выражение. Его нельзя вычислять через `self.evaluate(arg)`.

2. `parser.rs` создает `Expr::Lambda` без поля `paren`.

В AST поле есть, но здесь оно не передается:

```rust
return Ok(Expr::Lambda { parameters, _body });
```

Из-за этого сборка падает.

Для самой лямбды `paren` может быть вообще не нужен. Ошибка arity должна брать строку из `Expr::Call.paren`, потому что именно вызов знает закрывающую скобку аргументов. Если хочется красивое имя/позицию для lambda literal, лучше хранить `keyword: Token` или ничего.

3. `function_statement()` теперь дважды ожидает `(`.

Сейчас там осталось:

```rust
self.consume(TokenKind::LeftParen, "Expect '(' after function name!")?;
let parameters = self.parse_function_parameters()?;
```

А `parse_function_parameters()` тоже начинает с:

```rust
self.consume(TokenKind::LeftParen, "Expect '(' before parameters")?;
```

Из-за этого обычные функции ломаются: после `fun foo(` helper ожидает еще одну `(`, но видит первый параметр или `)`.

Нужно выбрать один контракт:

- либо helper сам потребляет `(` и `)`, тогда `function_statement()` не должен потреблять `(` заранее;
- либо helper парсит только содержимое между скобками, тогда lambda parser должен потреблять `(` отдельно.

4. `LoxLambda` почти полностью дублирует `LoxFunction`.

Это не фатально на первом шаге, но видно, что логика одинаковая:

- arity = число параметров;
- создать environment с `closure`;
- забиндить параметры к аргументам;
- выполнить _body;
- поймать `ControlFlow::Return(value)`.

Можно оставить отдельный `LoxLambda`, пока доводишь фичу до рабочего состояния. Но хорошее финальное решение - обобщить функцию над декларацией вида `{ name: Option<Token>, parameters, _body, closure }` или сделать отдельную структуру function object без привязки к `Stmt::Function`.

5. `return` внутри lambda, скорее всего, у тебя будет работать правильно, но `return` вне функции сейчас не ограничен.

Это уже существующая проблема вокруг функций: парсер принимает `return` как statement где угодно. Для lambda это становится заметнее. В книге это обычно решается resolver'ом: нельзя `return` вне function/lambda _body.

6. `Display for Stmt` выглядит как побочный шум.

Для реализации lambda он не обязателен. Он появился, вероятно, потому что `Expr::Lambda` пытается печатать _body. Это не основная часть runtime-семантики и может отвлечь. Если нужен debug print AST, лучше держать это отдельно от интерпретатора.

## Почему сейчас не собирается

`cargo test` падает с такими смысловыми ошибками:

- pattern matching для `Expr::Lambda` не учитывает поле `paren`;
- у `&Expr` нет поля `lexeme`, потому что параметры ошибочно объявлены как `Vec<Expr>`;
- parser передает `Vec<Token>` туда, где AST ожидает `Vec<Expr>`;
- при создании `Expr::Lambda` не задан `paren`.

То есть компилятор подсвечивает ровно тот же архитектурный конфликт: параметры лямбды должны быть tokens, а вычисление lambda literal не должно вычислять параметры.

## Как бы я продолжал

Минимальный правильный путь:

1. Сделать `Expr::Lambda { parameters: Vec<Token>, _body: Vec<Stmt> }`.
2. Починить контракт `parse_function_parameters()`, чтобы он не ломал function declaration.
3. В `primary()` парсить `fun (...) { ... }` в `Expr::Lambda`.
4. В `evaluate_lambda_expression()` просто возвращать `Value::Callable(Rc::new(LoxLambda::new(expr.clone(), self.environment.clone())))`.
5. Оставить вызов полностью на `evaluate_call_expression()`.
6. Проверить сценарии:

```lox
var inc = fun (x) {
  return x + 1;
};
print inc(41); // 42
```

```lox
print fun (x) {
  return x * 2;
}(21); // 42
```

```lox
var makeAdder = fun (a) {
  return fun (b) {
    return a + b;
  };
};
var add10 = makeAdder(10);
print add10(32); // 42
```

Если третий пример работает, значит closure для lambda сделан правильно.

Итог: ты идешь в правильном направлении по структуре AST/parser/callable, но сейчас перепутал "создать функцию как значение" и "вызвать функцию". Это ключевой момент для lambda expressions.
