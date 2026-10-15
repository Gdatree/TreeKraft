# TreeKraft u1

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Open Source](https://badges.frapsoft.com/os/v1/open-source.svg?v=103)](https://github.com/Gdatree/TreeKraft)
[![Windows](https://img.shields.io/badge/Windows-10%2F11-blue)](https://github.com/Gdatree/TreeKraft/releases/latest)

**TreeKraft** — консольный язык программирования с синтаксисом как Python: 2D-графика, SQLite, HTTP-сервер и TUI в одном `.exe` без зависимостей. Код открыт, никаких троянов.

Внутри два движка, быстрый байткод и медленный интерпретатор, и 54 дифференциальных теста проверяют, что оба отвечают одинаково.

---

## Скачать

Готовый установщик лежит в [релизах](https://github.com/Gdatree/TreeKraft/releases/latest) (`TreeKraft instaler.exe`, Windows 10/11, x64).

> Windows может показать предупреждение. Нажми **«Подробнее»** → **«Выполнить в любом случае»**. Файл безопасен, код открыт.

## Собрать из исходников

```sh
cargo build --release
target/release/treekraft.exe game.tk   # запустить скрипт
target/release/treekraft.exe            # REPL
```

```python
l = list(3, 1, 2)
print(list_sum(l))           # 6
for i in range(0, 100) do
    print("hi " + i)
end
```

## Проверки

```powershell
powershell -ExecutionPolicy Bypass -File "tools/run tests.ps1"   # сюита 10/10
powershell -ExecutionPolicy Bypass -File "tools/test diff.ps1"   # паритет движков
powershell -ExecutionPolicy Bypass -File "tools/test build.ps1"  # скрипт в exe
powershell -ExecutionPolicy Bypass -File "tools/test tkp.ps1"    # пакеты
```

---

## Возможности

| Категория | Что умеет |
|-----------|-----------|
| REPL | Интерактивный ввод команд, история, справка по темам (`help math`, `help algo`) |
| Многострочный ввод | `if/elif/end`, `while/end`, `for/end`, `loop/break/continue`, `try/catch` |
| Функции | `def name(params), body`, лямбды, `call`, `return`, модули (`import "lib.tk" as ns`) |
| Математика | 2963 команды в 57 модулях: `sin`, `sqrt`, `clamp`, `gcd`, `fibonacci`, матрицы, статистика |
| Строки | Интерполяция `"Hello {name}"`, `upper`, `split`, `replace`, `regex`, `slugify` |
| Списки и словари | `list`, `add`, `get`, `sort`, срезы `l[1:3]`, `dict`, `dget`, `dmerge` |
| ASCII-дисплей | `ldsc`, `ldbox`, `ldtext`, `ldshow`, спрайты, анимации |
| 2D-графика | Окно, пиксели, линии, круги, спрайты, клавиатура/мышь |
| SQLite | `db_open`, `db_exec`, `db_query`, `db_tables` |
| HTTP-сервер | `srv_start`, `srv_stop`, роуты, статика, JSON |
| TUI | Окна, кнопки, текст, прогресс в терминале |
| Файлы | `read`, `write`, `append`, `dir`, `mkdir`, `cp`, `mv`, `fsize` |
| Система | `shell`, `exec_output`, `env`, `sysinfo`, `timer`, `args` |
| Упаковка | `treekraft build app.tk` — скрипт в exe; `treekraft test` — свой раннер |

## Доки

- `docs/treekraft.md` — полная документация: семантика, ошибки, модули
- `docs/COMMANDS.md` — все команды по модулям (`python "tools/gen commands.py"`)
- `installer.iss` — установщик Inno Setup (релиз u1, пакет 1.0.0)
