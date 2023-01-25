# The granny problem

````
Бабушке нужно сгенерировать пароль, она слышала, что если взять четыре
слова из английского словаря, то можно получить хороший вариант. Но
проблема в том, что бабушка печатает одним пальцем и перемещать палец по
клавиатуре ей затруднительно, поэтому необходимо использовать такие слова,
которые эти перемещения минимизируют (считаются перемещения по четырём
сторонам, например, от "F" до "H" необходимо выполнить два перемещения, а
от "A" до "E" три), при том, что общая длина пароля будет от 20 до 24 символов.
Требуется найти наилучший пароль для бабушки.
````

This is the solution for the `Granny` problem.

It's written on *Rust* and uses `nightly` env.

## Quick Start

Prepare `rustup`:

```sh
rustup-init
```

Ensure you choose `nightly` version:

```sh
rustc --version
> rustc 1.69.0-nightly (c8e6a9e8b 2023-01-23)
```

To build documentation and run tests:

```sh
cargo test
cargo doc
```

To run solver:

```
cargo run
```
