use crate::main::vm::{VM, Value};
use crate::main::screen::parse_color;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

/// Вывод и цвета живут тут чтобы скрипт показывал человеку понятный результат
pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // Текст печатаем сразу чтобы результат скрипта был виден человеку

    if code.starts_with("~print(") && code.ends_with(')') {
        let inner = &code[7..code.len() - 1];
        let parts = VM::split_top_level(inner, ',');
        if parts.len() <= 1 {
            let val = match vm.eval_value(inner) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let text = match &val { Value::Text(t) => vm.interp(t), _ => val.display() };
            println!("{}", text);
            return Some(Ok(Value::Text(text)));
        }
        // Несколько значений клеим через пробел чтобы печатать все одним вызовом без плюсиков
        let mut out = Vec::with_capacity(parts.len());
        for p in &parts {
            let v = match vm.eval_value(p.trim()) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            out.push(v.display());
        }
        let text = vm.interp(&out.join(" "));
        println!("{}", text);
        return Some(Ok(Value::Text(text)));
    }

    if code.starts_with("~println(") && code.ends_with(')') {
        let inner = &code[9..code.len() - 1];
        let parts = VM::split_top_level(inner, ',');
        if parts.len() <= 1 {
            let val = match vm.eval_value(inner) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let text = match &val { Value::Text(t) => vm.interp(t), _ => val.display() };
            println!("{}", text);
            return Some(Ok(Value::Text(text)));
        }
        let mut out = Vec::with_capacity(parts.len());
        for p in &parts {
            let v = match vm.eval_value(p.trim()) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            out.push(v.display());
        }
        let text = vm.interp(&out.join(" "));
        println!("{}", text);
        return Some(Ok(Value::Text(text)));
    }

    if code.starts_with("~echo(") && code.ends_with(')') {
        let name = code[6..code.len() - 1].trim();
        let v = vm.get_var(name).unwrap_or(Value::None);
        vm.echo(format!("{} = {}", name, v.display()));
        return Some(Ok(v));
    }

    if code.starts_with("~dump(") && code.ends_with(')') {
        let name = code[6..code.len() - 1].trim();
        let v = vm.get_var(name).unwrap_or(Value::None);
        vm.echo(format!("{} : {} = {}", name, v.type_name(), v.display()));
        return Some(Ok(v));
    }

    if code.starts_with("~format(") && code.ends_with(')') {
        return resolve(vm, &format!("~printf{}", &code[8..]));
    }

    // По шаблону подставляем значения в плейсхолдеры чтобы не собирать строку руками
    if code.starts_with("~printf(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if !p.is_empty() {
            let fmt = match vm.eval_value(p[0].trim()) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            let mut args: Vec<Value> = Vec::new();
            for a in p.iter().skip(1) {
                match vm.eval_value(a.trim()) {
                    Ok(v) => args.push(v),
                    Err(e) => return Some(Err(e)),
                }
            };
            let mut out = String::new();
            let mut ai = 0usize;
            let mut it = fmt.chars().peekable();
            while let Some(c) = it.next() {
                if c == '%' {
                    match it.peek() {
                        Some('%') => { out.push('%'); it.next(); }
                        Some('s') => { it.next(); let v = if ai < args.len() { &args[ai] } else { &Value::None }; out.push_str(&v.display()); ai += 1; }
                        Some('d') => { it.next(); let v = if ai < args.len() { &args[ai] } else { &Value::None }; out.push_str(&Value::fmt_num(v.as_number().unwrap_or(0.0).trunc())); ai += 1; }
                        Some('f') => { it.next(); let v = if ai < args.len() { &args[ai] } else { &Value::None }; out.push_str(&format!("{:.3}", v.as_number().unwrap_or(0.0))); ai += 1; }
                        Some(_) => { out.push('%'); }
                        None => out.push('%'),
                    }
                } else {
                    out.push(c);
                }
            }
            println!("{}", out);
            return Some(Ok(Value::Text(out)));
        }
        return Some(Err("~printf: expected format string".to_string()));
    }

    // Текст красим чтобы важное бросалось в глаза

    macro_rules! color_cmd {
        ($p:expr, $l:expr, $ansi:expr) => {
            if code.starts_with($p) && code.ends_with(')') {
                let text = match vm.eval_value(&code[$p.len()..code.len() - 1]) {
                    Ok(v) => v.as_text().unwrap_or_default(),
                    Err(e) => return Some(Err(e)),
                };
                let out = format!("{}{}\x1B[0m", $ansi, text);
                println!("{}", out);
                return Some(Ok(Value::Text(text)));
            }
        };
    }

    color_cmd!("~red(", 5, "\x1B[91m");
    color_cmd!("~green(", 7, "\x1B[92m");
    color_cmd!("~blue(", 6, "\x1B[94m");
    color_cmd!("~yellow(", 8, "\x1B[93m");
    color_cmd!("~cyan(", 6, "\x1B[96m");
    color_cmd!("~magenta(", 9, "\x1B[95m");

    if code.starts_with("~fg(") && code.ends_with(')') {
        let color = VM::tq(&code[4..code.len() - 1]);
        print!("{}", VM::ansi_fg(&color));
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

if code.starts_with("~bg(") && code.ends_with(')') {
        let color = VM::tq(&code[4..code.len() - 1]);
        print!("{}", VM::ansi_bg(&color));
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code.starts_with("~color(") && code.ends_with(')') {
        let color = VM::tq(&code[7..code.len() - 1]);
        print!("{}", VM::ansi_fg(&color));
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code.starts_with("~color ") {
        let color = code[7..].trim();
        print!("{}", VM::ansi_fg(color));
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code.starts_with("~bgcolor(") && code.ends_with(')') {
        let color = VM::tq(&code[9..code.len() - 1]);
        print!("{}", VM::ansi_bg(&color));
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code == "~bold()" || code == "~bold" {
        print!("\x1B[1m");
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code == "~italic()" || code == "~italic" {
        print!("\x1B[3m");
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code == "~underline()" || code == "~underline" {
        print!("\x1B[4m");
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code == "~strikethrough()" || code == "~strikethrough" {
        print!("\x1B[9m");
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code == "~blink()" || code == "~blink" {
        print!("\x1B[5m");
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code == "~hidden()" || code == "~hidden" {
        print!("\x1B[8m");
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code == "~resetcolor()" || code == "~resetcolor" {
        print!("\x1B[0m");
        let _ = io::stdout().flush();
        return Some(Ok(Value::None));
    }

    if code.starts_with("~rgb(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() >= 3 {
            let r = vm.eval_expr(&p[0]).unwrap_or(0.0) as u8;
            let g = vm.eval_expr(&p[1]).unwrap_or(0.0) as u8;
            let b = vm.eval_expr(&p[2]).unwrap_or(0.0) as u8;
            if p.len() >= 4 {
                let text = match vm.eval_value(&p[3]) {
                    Ok(v) => v.as_text().unwrap_or_default(),
                    Err(e) => return Some(Err(e)),
                };
                let out = format!("\x1B[38;2;{};{};{}m{}\x1B[0m", r, g, b, text);
                println!("{}", out);
                return Some(Ok(Value::Text(text)));
            } else {
                print!("\x1B[38;2;{};{};{}m", r, g, b);
                let _ = io::stdout().flush();
                return Some(Ok(Value::None));
            }
        }
    }

    if code.starts_with("~rgb_bg(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 3 {
            let r = vm.eval_expr(&p[0]).unwrap_or(0.0) as u8;
            let g = vm.eval_expr(&p[1]).unwrap_or(0.0) as u8;
            let b = vm.eval_expr(&p[2]).unwrap_or(0.0) as u8;
            print!("\x1B[48;2;{};{};{}m", r, g, b);
            let _ = io::stdout().flush();
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~gradient(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() >= 3 {
            let text = match vm.eval_value(&p[0]) {
                Ok(v) => v.as_text().unwrap_or_default(),
                Err(e) => return Some(Err(e)),
            };
            let c1 = parse_color(&VM::tq(&p[1]));
            let c2 = parse_color(&VM::tq(&p[2]));
            if let (Some((r1, g1, b1)), Some((r2, g2, b2))) = (c1, c2) {
                let chars: Vec<char> = text.chars().collect();
                let n = chars.len().max(1) as f64;
                let mut out = String::new();
                for (i, ch) in chars.iter().enumerate() {
                    let t = i as f64 / (n - 1.0).max(1.0);
                    let r = (r1 as f64 + (r2 as f64 - r1 as f64) * t) as u8;
                    let g = (g1 as f64 + (g2 as f64 - g1 as f64) * t) as u8;
                    let b = (b1 as f64 + (b2 as f64 - b1 as f64) * t) as u8;
                    out.push_str(&format!("\x1B[38;2;{};{};{}m{}", r, g, b, ch));
                }
                out.push_str("\x1B[0m");
                vm.echo(format!("{}", out));
                return Some(Ok(Value::Text(text)));
            }
        }
    }

if code.starts_with("~rainbow(") && code.ends_with(')') {
        let text = match vm.eval_value(&code[9..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let colors = [(255, 0, 0), (255, 165, 0), (255, 255, 0), (0, 255, 0), (0, 0, 255), (148, 0, 211)];
        let mut out = String::new();
        for (i, ch) in text.chars().enumerate() {
            let (r, g, b) = colors[i % colors.len()];
            out.push_str(&format!("\x1B[38;2;{};{};{}m{}", r, g, b, ch));
        }
        out.push_str("\x1B[0m");
        vm.echo(format!("{}", out));
        return Some(Ok(Value::Text(text)));
    }

    if code.starts_with("~cur(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() == 2 {
            let x = vm.eval_expr(&p[0]).unwrap_or(1.0) as usize;
            let y = vm.eval_expr(&p[1]).unwrap_or(1.0) as usize;
            print!("\x1B[{};{}H", y, x);
            let _ = io::stdout().flush();
            return Some(Ok(Value::None));
        }
    }

    // Прогресс и спиннер показывают что скрипт жив пока идет долгая работа

    if code.starts_with("~progressbar(") && code.ends_with(')') {
        return resolve(vm, &format!("~progress{}", &code[13..]));
    }

    if code == "~spinner()" {
        return resolve(vm, "~spinner_step()");
    }

    // Рамки выделяют заголовок чтобы его заметили сразу

    if code.starts_with("~banner(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[8..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let text = vm.interp(&val);
        let w = text.len() + 4;
        vm.echo(format!("  \x1B[1;92m╔{}╗\x1B[0m", "═".repeat(w)));
        vm.echo(format!("  \x1B[1;92m║\x1B[0m  {:<w$}\x1B[1;92m║\x1B[0m", text, w = w));
        vm.echo(format!("  \x1B[1;92m╚{}╝\x1B[0m", "═".repeat(w)));
        return Some(Ok(Value::Bool(true)));
    }

if code.starts_with("~box(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if !p.is_empty() {
            let val = match vm.eval_value(&p[0]) {
                Ok(v) => v.as_text().unwrap_or_default(),
                Err(e) => return Some(Err(e)),
            };
            let text = vm.interp(&val);
            let ch = if p.len() > 1 {
                VM::tq(&p[1]).chars().next().unwrap_or('─')
            } else {
                '─'
            };
            let w = text.len() + 2;
            vm.echo(format!("  {}", ch.to_string().repeat(w)));
            vm.echo(format!("  {} {}", ch, text));
            vm.echo(format!("  {}", ch.to_string().repeat(w)));
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~hr(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[4..code.len() - 1], ',');
        let ch = if !p.is_empty() && !p[0].trim().is_empty() {
            VM::tq(&p[0]).chars().next().unwrap_or('─')
        } else {
            '─'
        };
        let w = if p.len() > 1 {
            vm.eval_expr(&p[1]).unwrap_or(60.0) as usize
        } else {
            60
        };
        vm.echo(format!("  {}", ch.to_string().repeat(w)));
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~hr" {
        vm.echo(format!("  {}", "─".repeat(60)));
        return Some(Ok(Value::Bool(true)));
    }

    // Всплывашка сама прячется через пару секунд чтобы короткое сообщение не мешало
    if code.starts_with("~al(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[4..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let text = vm.interp(&val);
        let w = text.len() + 4;
        vm.echo(format!("\x1B[1;93m╔{}╗\x1B[0m", "═".repeat(w)));
        vm.echo(format!("\x1B[1;93m║  {}  ║\x1B[0m", text));
        vm.echo(format!("\x1B[1;93m╚{}╝\x1B[0m", "═".repeat(w)));
        thread::sleep(Duration::from_secs(2));
        return Some(Ok(Value::Bool(true)));
    }

    // Колонки таблицы ровняем чтобы строки легко сравнивались глазом

    if code.starts_with("~table(") && code.ends_with(')') {
        let name = code[7..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let rows: Vec<Vec<String>> = list.iter().filter_map(|v| match v { Value::List(r) => Some(r.iter().map(|c| c.display()).collect()), _ => None }).collect();
        if rows.is_empty() { return Some(Ok(Value::None)); }
        let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let widths: Vec<usize> = (0..cols).map(|c| rows.iter().map(|r| r.get(c).map(|s| s.len()).unwrap_or(0)).max().unwrap_or(3).max(3)).collect();
        let sep = format!("+{}+", widths.iter().map(|w| "-".repeat(w+2)).collect::<Vec<_>>().join("+"));
        println!("{}", sep);
        for row in &rows {
            let cells: Vec<String> = (0..cols).map(|c| format!(" {:<w$} ", row.get(c).map(|s| s.as_str()).unwrap_or(""), w=widths[c])).collect();
            println!("|{}|", cells.join("|")); println!("{}", sep);
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~chart_bar(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 2 {
            let name = p[0].trim();
            let max_w = vm.eval_expr(&p[1]).ok()? as usize;
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
            if nums.is_empty() { return Some(Ok(Value::None)); }
            let mx = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let label_w = nums.len().to_string().len().max(4);
            for (i, &v) in nums.iter().enumerate() {
                let bar_len = if mx > 0.0 { ((v / mx) * max_w as f64).round() as usize } else { 0 };
                let bar = "█".repeat(bar_len);
                println!("  {:<lw$} │ {} {}", i, bar, Value::fmt_num(v), lw=label_w);
            }
            return Some(Ok(Value::None));
        }
    }

    // Полоска процентов рисуется прямо в строке чтобы не спамить новыми строками

    if code.starts_with("~progress(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 2 {
            let cur = vm.eval_expr(&p[0]).ok()? as usize;
            let max = vm.eval_expr(&p[1]).ok()? as usize;
            let w = if p.len() >= 3 { vm.eval_expr(&p[2]).ok()? as usize } else { 20 };
            let pct = if max > 0 { cur * 100 / max } else { 0 };
            let filled = if max > 0 { cur * w / max } else { 0 };
            let bar = format!("{}{}", "█".repeat(filled), "░".repeat(w.saturating_sub(filled)));
            print!("\r  {} {:>3}%", bar, pct);
            let _ = std::io::stdout().flush();
            if cur >= max { println!(); }
            return Some(Ok(Value::Number(pct as f64)));
        }
    }

    // Галочки и стрелки делают список похожим на вывод системной утилиты

    if code.starts_with("~quote(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len()-1]).ok()?.as_text()?;
        vm.echo(format!("  \x1B[2;37m\"{}\"\x1B[0m", s));
        return Some(Ok(Value::Text(s)));
    }

    if code.starts_with("~bullet(") && code.ends_with(')') {
        let s = vm.eval_value(&code[8..code.len()-1]).ok()?.as_text()?;
        vm.echo(format!("  \x1B[97m• {}\x1B[0m", s));
        return Some(Ok(Value::Text(s)));
    }

    if code.starts_with("~check(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len()-1]).ok()?.as_text()?;
        vm.echo(format!("  \x1B[92m✓ {}\x1B[0m", s));
        return Some(Ok(Value::Text(s)));
    }

    if code.starts_with("~cross(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len()-1]).ok()?.as_text()?;
        vm.echo(format!("  \x1B[91m✗ {}\x1B[0m", s));
        return Some(Ok(Value::Text(s)));
    }

    if code.starts_with("~arrow(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len()-1]).ok()?.as_text()?;
        vm.echo(format!("  \x1B[96m→ {}\x1B[0m", s));
        return Some(Ok(Value::Text(s)));
    }

    if code == "~line" || code.starts_with("~line(") && code.ends_with(')') {
        let w = if code.len() > 7 { vm.eval_expr(&code[6..code.len()-1]).unwrap_or(50.0) as usize } else { 50 };
        println!("{}", "─".repeat(w));
        return Some(Ok(Value::None));
    }

    if code == "~sep" || code == "~separator" {
        println!("{}", "═".repeat(60));
        return Some(Ok(Value::None));
    }

    if code.starts_with("~indent(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ',');
        if p.len() >= 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let n = (vm.eval_expr(&p[1]).ok()? as usize).min(10_000);
            let r = format!("{}{}", " ".repeat(n), s);
            vm.echo(format!("{}", r));
            return Some(Ok(Value::Text(r)));
        }
    }

    if code.starts_with("~repeat_str(") && code.ends_with(')') { let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = (vm.eval_expr(&p[1]).ok()? as usize).min(100_000); let r = s.repeat(n); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~justify_left(") && code.ends_with(')') { let p = VM::split_top_level(&code[14..code.len()-1], ','); if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = (vm.eval_expr(&p[1]).ok()? as usize).min(10_000); let r = format!("{:width$}", s, width=n); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~justify_right(") && code.ends_with(')') { let p = VM::split_top_level(&code[15..code.len()-1], ','); if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = (vm.eval_expr(&p[1]).ok()? as usize).min(10_000); let r = format!("{:>width$}", s, width=n); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~justify_center(") && code.ends_with(')') { let p = VM::split_top_level(&code[16..code.len()-1], ','); if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = (vm.eval_expr(&p[1]).ok()? as usize).min(10_000); let pad = n.saturating_sub(s.len()); let l = pad/2; let mut r = format!("{}{}", " ".repeat(l), s); if r.len() < n { r += &" ".repeat(n - r.len()); } vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~print_table(") && code.ends_with(')') { let name = code[13..code.len()-1].trim(); let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) }; for row in &list { if let Value::List(cells) = row { println!("{}", cells.iter().map(|c| format!("{:<12}", c.display())).collect::<Vec<_>>().join("")); } } return Some(Ok(Value::None)); }
    if code.starts_with("~color_red(") && code.ends_with(')') { let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; vm.echo(format!("\x1B[91m{}\x1B[0m", s)); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~color_green(") && code.ends_with(')') { let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?; vm.echo(format!("\x1B[92m{}\x1B[0m", s)); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~color_blue(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; vm.echo(format!("\x1B[94m{}\x1B[0m", s)); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~color_yellow(") && code.ends_with(')') { let s = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text()?; vm.echo(format!("\x1B[93m{}\x1B[0m", s)); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~color_cyan(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; vm.echo(format!("\x1B[96m{}\x1B[0m", s)); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~color_magenta(") && code.ends_with(')') { let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text()?; vm.echo(format!("\x1B[95m{}\x1B[0m", s)); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~scroll_text(") && code.ends_with(')') { let p = VM::split_top_level(&code[13..code.len()-1], ','); if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = (vm.eval_expr(&p[1]).ok()? as usize).min(10_000); let r = format!("{:>width$}", s, width=n); vm.echo(format!("{}", r)); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~typewriter(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; for c in s.chars() { print!("{}", c); std::io::stdout().flush().ok(); std::thread::sleep(std::time::Duration::from_millis(50)); } println!(); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~loading_bar(") && code.ends_with(')') { let p = VM::split_top_level(&code[13..code.len()-1], ','); if p.len() >= 2 { let cur = vm.eval_expr(&p[0]).ok()? as usize; let max = vm.eval_expr(&p[1]).ok()? as usize; let w = if p.len() >= 3 { vm.eval_expr(&p[2]).ok()? as usize } else { 20 }; let pct = cur * 100 / max.max(1); let f = cur * w / max.max(1); print!("\r[{}{}] {:>3}%", "=".repeat(f), " ".repeat(w.saturating_sub(f)), pct); std::io::stdout().flush().ok(); if cur >= max { println!(); } return Some(Ok(Value::None)); } else { return None; } }
    if code == "~clear_line" { print!("\r\x1B[K"); std::io::stdout().flush().ok(); return Some(Ok(Value::None)); }
    if code == "~cursor_up" { print!("\x1B[1A"); std::io::stdout().flush().ok(); return Some(Ok(Value::None)); }
    if code == "~cursor_down" { print!("\x1B[1B"); std::io::stdout().flush().ok(); return Some(Ok(Value::None)); }
    if code == "~cursor_save" { print!("\x1B[s"); std::io::stdout().flush().ok(); return Some(Ok(Value::None)); }
    if code == "~cursor_restore" { print!("\x1B[u"); std::io::stdout().flush().ok(); return Some(Ok(Value::None)); }
    if code.starts_with("~cursor_col(") && code.ends_with(')') { let n = vm.eval_expr(&code[12..code.len()-1]).ok()? as usize; print!("\x1B[{}G", n); std::io::stdout().flush().ok(); return Some(Ok(Value::None)); }
    if code.starts_with("~bg_color(") && code.ends_with(')') { let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 3 { let r = vm.eval_expr(&p[0]).ok()? as u8; let g = vm.eval_expr(&p[1]).ok()? as u8; let b = vm.eval_expr(&p[2]).ok()? as u8; print!("\x1B[48;2;{};{};{}m", r, g, b); std::io::stdout().flush().ok(); return Some(Ok(Value::None)); } else { return None; } }
    if code.starts_with("~fg_color(") && code.ends_with(')') { let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 3 { let r = vm.eval_expr(&p[0]).ok()? as u8; let g = vm.eval_expr(&p[1]).ok()? as u8; let b = vm.eval_expr(&p[2]).ok()? as u8; print!("\x1B[38;2;{};{};{}m", r, g, b); std::io::stdout().flush().ok(); return Some(Ok(Value::None)); } else { return None; } }
    if code == "~reset_styles" { print!("\x1B[0m"); std::io::stdout().flush().ok(); return Some(Ok(Value::None)); }
    if code.starts_with("~dim_text(") && code.ends_with(')') { let s = vm.eval_value(&code[10..code.len()-1]).ok()?.as_text()?; vm.echo(format!("\x1B[2m{}\x1B[0m", s)); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~blink_text(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; vm.echo(format!("\x1B[5m{}\x1B[0m", s)); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~reverse_text(") && code.ends_with(')') { let s = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text()?; vm.echo(format!("\x1B[7m{}\x1B[0m", s)); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~overline(") && code.ends_with(')') { let s = vm.eval_value(&code[10..code.len()-1]).ok()?.as_text()?; vm.echo(format!("{}\n{}", "\u{203E}".repeat(s.len()), s)); return Some(Ok(Value::Text(s))); }
    if code.starts_with("~strike(") && code.ends_with(')') { let s = vm.eval_value(&code[8..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().map(|c| format!("{}\u{0336}", c)).collect(); vm.echo(format!("{}", r)); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~ascii_border(") && code.ends_with(')') { let s = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text()?; let w = s.len() + 2; vm.echo(format!("+{}+", "-".repeat(w))); println!("| {} |", s); println!("+{}+", "-".repeat(w)); return Some(Ok(Value::Text(s))); }

    None
}