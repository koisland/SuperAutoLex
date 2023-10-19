# Super Auto Lex
Lexer to parse SAP Pet effect text. WIP


### Generate `data/` CSV
Using [`saptest`](https://github.com/koisland/SuperAutoTest)'s `sap.db`.

```bash
mkdir -p data
sqlite3 -csv -header sap.db "select name,effect_trigger,effect from pets where is_token = 'false'" | sort | uniq > data/effects.csv
```

### Unique Effect Triggers

```bash
awk -F, '{ gsub("\"", "", $2); print $2 }' data/effects.csv | sort | uniq
```

### Rules
The first word of any effect must be:
* An action
    * `Give any friend ...`
* A condition
    * `If all friends ...`

Item names are always uppercase.
* Pets can be one or two words long.
    * `Dog`
    * `Lizard Tail`
* Foods end with `Perk`.
    * `Melon Perk`

### TODO
* [ ] Declarative macro to construct effects.
