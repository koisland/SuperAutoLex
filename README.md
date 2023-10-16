# Super Auto Lex
Lexer to parse SAP Pet effect text. WIP


### Generate `data/` CSVs
Using [`saptest`](https://github.com/koisland/SuperAutoTest)'s `sap.db`.

```bash
mkdir -p data
for i in {1..6}; do sqlite3 -csv -header sap.db "select name,effect from pets where tier = ${i} and is_token = 'false'" | sort | uniq > data/t${i}.csv; done
```
