# 🌲 TreeKraft 3.3.0

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Open Source](https://badges.frapsoft.com/os/v1/open-source.svg?v=103)](https://github.com/Gdatree/TreeKraft)

**TreeKraft** — консольный язык программирования с ASCII-графикой (KGame) и интерактивным REPL.  
Работает без Python. Всё в одном `.exe`. Код открыт — никаких троянов.

## 📦 Скачать установщик

👉 [TreeKraft_Setup.exe](https://github.com/Gdatree/TreeKraft/releases/latest)  

(Windows 10/11, x64)

## ✨ Что есть в 3.3.0

- Интерактивный REPL с многострочным вводом (`if/end`, `while/end`, `for/end`)
- Функции с параметрами, замыкания, интерполяция строк `"Hello {name}"`
- Математика: `sin, cos, atan2, sqrt, log, exp, clamp, pi, e`
- Списки, сортировка, строковые операции
- **ASCII-дисплей**: `~ldsc()`, `~ldbox()`, `~ldm()` (линии), `~ldtext()`, `~ldshow()`
- **KGame**: консольная графика с цветами ANSI (`kg.rect`, `kg.circ`, `kg.pixel`, `kg.text`)
- Работа с файлами: `read, write, append, readlines, dir, mkdir, cp, mv`
- Конфиги `.dtam`: `~dcfg set/get/del`, сериализация `~save/~load`
- Трассировка, таймеры, системные вызовы, переменные окружения
- Помощь по темам: `help math`, `help strings`, `help display` и др.

## 🔧 Собрать из исходников

```bash
git clone https://github.com/Gdatree/TreeKraft.git
cd TreeKraft
cargo build --release
