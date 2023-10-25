# Super Auto Lex
Lexer to parse SAP Pet effect text into a stream of tokens.

> WIP

### Usage
```bash
cargo add --git https://github.com/koisland/SuperAutoLex
```

### Rules
Item names are always uppercase.
* Pets can be one or two words long.
    * `Dog`
    * `Lizard Tail`
* Foods end with `Perk`.
    * `Melon Perk`
    * Can also omitted if prior word is `with`.
        * `Dog with Melon.`

### TODO
* [ ] Declarative macro to construct effects.
