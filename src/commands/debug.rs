use crate::main::vm::{VM, Value};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};

/// Здесь собраны вспомогательные команды для отладки и окружения чтобы не разносить мелкие группы по отдельным файлам
/// Сюда входят алиасы шаблоны события тесты журнал настройки разбор адресов потоки и прочее

/// При провале проверки запоминаем результат чтобы прогон мог учесть ошибку
/// Обычно возвращаем ошибку и останавливаем скрипт чтобы сбой было видно сразу
/// Под командой treekraft test только печатаем крест и идем дальше чтобы собрать весь отчет
fn assert_fail(vm: &mut VM, kind: &str, msg: String) -> Option<Result<Value, String>> {
    vm.test_results.push((kind.to_string(), false, msg.clone()));
    if vm.test_continue {
        println!("  \x1B[91m✗ {} — {}\x1B[0m", kind, msg);
        return Some(Ok(Value::Bool(false)));
    }
    Some(Err(msg))
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // Короткие имена храним в памяти скрипта чтобы длинные команды вызывались одним словом

    if code.starts_with("~alias(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let cmd = VM::tq(&p[1]);
            vm.aliases.insert(name.clone(), cmd);
            vm.echo(format!("Alias '{}' created", name));
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~unalias(") && code.ends_with(')') {
        let ok = vm.aliases.remove(&VM::tq(&code[9..code.len() - 1])).is_some();
        return Some(Ok(Value::Bool(ok)));
    }

    if code == "~aliases()" {
        for (k, v) in &vm.aliases {
            println!("  {} → {}", k, v);
        }
        return Some(Ok(Value::None));
    }

    // Заготовки текста храним отдельно чтобы вызов render подставлял значения в места в фигурных скобках

    if code.starts_with("~template(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let text = VM::tq(&p[1]);
            vm.templates.insert(name, text);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~render(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if !p.is_empty() {
            let name = VM::tq(&p[0]);
            let mut tpl = vm.templates.get(&name).cloned().unwrap_or_default();
            // Общие переменные применяем первыми чтобы они действовали сразу на все шаблоны
            for (k, v) in &vm.template_vars {
                tpl = tpl.replace(&format!("{{{{{}}}}}", k), &v.display());
            }
            // Ключи переданного словаря применяем после общих чтобы уточнить именно этот вызов
            if p.len() > 1 {
                if let Some(Value::Dict(dict)) = vm.get_var(p[1].trim()) {
                    for (k, v) in &dict {
                        tpl = tpl.replace(&format!("{{{{{}}}}}", k), &v.display());
                    }
                }
            }
            // В конце раскрываем переменные скрипта чтобы в тексте работали обычные значения
            tpl = vm.interp(&tpl);
            vm.echo(&format!("{}", tpl));
            return Some(Ok(Value::Text(tpl)));
        }
    }

    if code.starts_with("~template_var(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            vm.template_vars.insert(name, val);
            return Some(Ok(Value::Bool(true)));
        }
    }

    // Подписчиков держим по именам сигналов чтобы вызов emit выполнял их по порядку подписки

    if code.starts_with("~on(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[4..code.len() - 1], ',');
        if p.len() == 2 {
            let ev = VM::tq(&p[0]);
            let cmd = VM::tq(&p[1]);
            vm.events.entry(ev).or_insert_with(Vec::new).push(cmd);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~emit(") && code.ends_with(')') {
        let ev = VM::tq(&code[6..code.len() - 1]);
        if let Some(handlers) = vm.events.get(&ev).cloned() {
            for h in &handlers {
                if let Err(e) = crate::main::eval::evaluate(vm, h) {
                    return Some(Err(e));
                }
            }
        }
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~off(") && code.ends_with(')') {
        let ok = vm.events.remove(&VM::tq(&code[5..code.len() - 1])).is_some();
        return Some(Ok(Value::Bool(ok)));
    }

    if code == "~events_list()" {
        for k in vm.events.keys() {
            println!("  {}", k);
        }
        return Some(Ok(Value::None));
    }

    // Короткий импорт сам дописывает расширение tk чтобы чужие скрипты подключались без полного имени

    if code.starts_with("~import(") && code.ends_with(')') {
        let f = VM::tq(&code[8..code.len() - 1]);
        match fs::read_to_string(&f) {
            Ok(c) => return Some(vm.run_script(&c)),
            Err(e) => return Some(Err(format!("import '{}': {}", f, e))),
        }
    }

    if code.starts_with("~import ") {
        let raw = VM::tq(code[8..].trim());
        let f = if raw.ends_with(".tk") { raw } else { format!("{}.tk", raw) };
        match fs::read_to_string(&f) {
            Ok(c) => return Some(vm.run_script(&c)),
            Err(e) => return Some(Err(format!("import '{}': {}", f, e))),
        }
    }

    if code.starts_with("~eval(") && code.ends_with(')') {
        let inner = vm.interp(&VM::tq(&code[6..code.len() - 1]));
        return Some(crate::main::eval::evaluate(vm, &inner));
    }

    // Здесь узнаем тип и наличие переменной а также приводим значение к нужному виду чтобы проверки писались коротко

    if code.starts_with("~type(") && code.ends_with(')') {
        let name = code[6..code.len() - 1].trim();
        let typ = vm.get_var(name).as_ref().map(|v| v.type_name()).unwrap_or("unknown");
        vm.echo(typ);
        return Some(Ok(Value::Text(typ.to_string())));
    }

    if code.starts_with("~exists(") && code.ends_with(')') {
        let ok = vm.has_var(code[8..code.len() - 1].trim());
        vm.echo(&format!("{}", ok));
        return Some(Ok(Value::Bool(ok)));
    }

    if code.starts_with("~tonum(") && code.ends_with(')') {
        let v = match vm.eval_value(&code[7..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        let n = v.as_number().unwrap_or(0.0);
        vm.echo(format!("{}", n));
        return Some(Ok(Value::Number(n)));
    }

    if code.starts_with("~tostr(") && code.ends_with(')') {
        let v = match vm.eval_value(&code[7..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        let s = v.as_text().unwrap_or_default();
        vm.echo(format!("{}", s));
        return Some(Ok(Value::Text(s)));
    }

    if code.starts_with("~tobool(") && code.ends_with(')') {
        let v = match vm.eval_value(&code[8..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        let b = v.truthy();
        vm.echo(format!("{}", b));
        return Some(Ok(Value::Bool(b)));
    }

    if code.starts_with("~typeof(") && code.ends_with(')') {
        return resolve(vm, &format!("~type{}", &code[8..]));
    }

    if code.starts_with("~inspect(") && code.ends_with(')') {
        return resolve(vm, &format!("~dump{}", &code[9..]));
    }

    // При провале проверки запоминаем результат а останавливать скрипт или идти дальше решает режим описанный в помощнике assert_fail

    if code.starts_with("~assert(") && code.ends_with(')') {
        let cond = &code[8..code.len() - 1];
        if !vm.eval_cond(cond).unwrap_or(false) {
            return assert_fail(vm, "assert", format!("Assertion failed: {}", cond));
        }
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~assert_eq(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let a = match vm.eval_value(&p[0]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            let b = match vm.eval_value(&p[1]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            if a != b {
                let msg = format!("assert_eq failed: {} != {}", a, b);
                return assert_fail(vm, "assert_eq", msg);
            }
            vm.test_results.push(("assert_eq".to_string(), true, "OK".to_string()));
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~assert_ne(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let a = match vm.eval_value(&p[0]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            let b = match vm.eval_value(&p[1]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            if a == b {
                let msg = format!("assert_ne failed: {} == {}", a, b);
                return assert_fail(vm, "assert_ne", msg);
            }
            vm.test_results.push(("assert_ne".to_string(), true, "OK".to_string()));
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~assert_gt(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let a = match vm.eval_expr(&p[0]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let b = match vm.eval_expr(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if a <= b {
                let msg = format!("assert_gt failed: {} <= {}", a, b);
                return assert_fail(vm, "assert_gt", msg);
            }
            vm.test_results.push(("assert_gt".to_string(), true, "OK".to_string()));
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~assert_lt(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let a = match vm.eval_expr(&p[0]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let b = match vm.eval_expr(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if a >= b {
                let msg = format!("assert_lt failed: {} >= {}", a, b);
                return assert_fail(vm, "assert_lt", msg);
            }
            vm.test_results.push(("assert_lt".to_string(), true, "OK".to_string()));
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~assert_true(") && code.ends_with(')') {
        let v = match vm.eval_value(&code[13..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        if !v.truthy() {
            let msg = format!("assert_true failed: {}", v.display());
            return assert_fail(vm, "assert_true", msg);
        }
        vm.test_results.push(("assert_true".to_string(), true, "OK".to_string()));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~assert_false(") && code.ends_with(')') {
        let v = match vm.eval_value(&code[14..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        if v.truthy() {
            let msg = format!("assert_false failed: {}", v.display());
            return assert_fail(vm, "assert_false", msg);
        }
        vm.test_results.push(("assert_false".to_string(), true, "OK".to_string()));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~assert_throws(") && code.ends_with(')') {
        let cmd = &code[15..code.len() - 1];
        let ok = crate::main::eval::evaluate(vm, cmd).is_err();
        if !ok {
            return assert_fail(vm, "assert_throws", "assert_throws failed: no error raised".to_string());
        }
        vm.test_results.push(("assert_throws".to_string(), true, "OK".to_string()));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~test(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let cmd = p[1].trim().to_string();
            match crate::main::eval::evaluate(vm, &cmd) {
                Ok(_) => {
                    vm.test_results.push((name.clone(), true, "PASS".to_string()));
                    println!("  \x1B[92m✓ {}\x1B[0m", name);
                }
                Err(e) => {
                    vm.test_results.push((name.clone(), false, e.clone()));
                    println!("  \x1B[91m✗ {} — {}\x1B[0m", name, e);
                }
            }
            return Some(Ok(Value::None));
        }
    }

    if code == "~test_report()" || code == "~test_summary()" {
        let total = vm.test_results.len();
        let passed = vm.test_results.iter().filter(|r| r.1).count();
        println!("\n  \x1B[1mTest Report\x1B[0m");
        println!("  ─────────────────────────────────");
        for (name, ok, msg) in &vm.test_results {
            if *ok {
                println!("  \x1B[92m✓ {}\x1B[0m", name);
            } else {
                println!("  \x1B[91m✗ {} — {}\x1B[0m", name, msg);
            }
        }
        println!("  ─────────────────────────────────");
        println!("  Passed: {}/{} ({}%)", passed, total, if total > 0 { passed * 100 / total } else { 0 });
        return Some(Ok(Value::None));
    }

    if code == "~test_reset()" {
        vm.test_results.clear();
        return Some(Ok(Value::Bool(true)));
    }

    // Весь список аргументов отдаем через args а один элемент берем по номеру через arg чтобы скрипт видел запуск
    // Нумерацию ведем с нуля поэтому при запуске с foo и bar нулевой элемент даст foo а первый даст bar
    if code == "~args" || code == "~args()" {
        let list: Vec<Value> = vm.args.iter().map(|a| Value::Text(a.clone())).collect();
        let list_val = Value::List(list);
        vm.echo(list_val.display());
        return Some(Ok(list_val));
    }
    if code.starts_with("~arg(") && code.ends_with(')') {
        let idx = match vm.eval_expr(&code[5..code.len() - 1]) {
            Ok(v) => v as usize,
            Err(e) => return Some(Err(e)),
        };
        match vm.args.get(idx) {
            Some(a) => return Some(Ok(Value::Text(a.clone()))),
            None => return Some(Err(format!("arg({}): out of range ({} args)", idx, vm.args.len()))),
        }
    }

    if code.starts_with("~benchmark(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let start = Instant::now();
            match crate::main::eval::evaluate(vm, &p[1]) {
                Ok(_) => {
                    let ms = start.elapsed().as_millis();
                    vm.echo(format!("[bench] {}: {}ms", name, ms));
                    return Some(Ok(Value::Number(ms as f64)));
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }

    if code.starts_with("~benchmark_n(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 3 {
            let name = VM::tq(&p[0]);
            let n = match vm.eval_expr(&p[1]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            let start = Instant::now();
            for _ in 0..n {
                if let Err(e) = crate::main::eval::evaluate(vm, &p[2]) {
                    return Some(Err(e));
                }
            }
            let ms = start.elapsed().as_millis();
            vm.echo(format!("[bench] {} x{}: {}ms ({:.3}ms/run)", name, n, ms, ms as f64 / n as f64));
            return Some(Ok(Value::Number(ms as f64)));
        }
    }

    // Сообщения пишем с уровнем важности чтобы тихие уровни отсекались порогом log_level и не шумели

    if code.starts_with("~log_info(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[10..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        vm.do_log(1, &msg);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~log_warn(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[10..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        vm.do_log(2, &msg);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~log_error(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[11..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        vm.do_log(3, &msg);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~log_debug(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[11..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        vm.do_log(0, &msg);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~log(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[5..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        vm.do_log(1, &msg);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~log_to_file(") && code.ends_with(')') {
        let path = VM::tq(&code[13..code.len() - 1]);
        vm.log_file = Some(path);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~log_level(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[11..code.len() - 1]).unwrap_or(1.0) as u8;
        vm.log_level = n;
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~loglevel(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[10..code.len() - 1]).unwrap_or(1.0) as u8;
        vm.log_level = n;
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~logfile(") && code.ends_with(')') {
        let path = VM::tq(&code[9..code.len() - 1]);
        vm.log_file = Some(path);
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~log_clear()" {
        if let Some(ref f) = vm.log_file.clone() {
            let _ = fs::write(f, "");
        }
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~log_show()" {
        if let Some(ref f) = vm.log_file {
            if let Ok(t) = fs::read_to_string(f) {
                print!("{}", t);
            }
        }
        return Some(Ok(Value::Bool(true)));
    }

    // Ошибка останавливает скрипт а остальные сообщения только красочно печатаем чтобы скрипт шел дальше

    if code.starts_with("~error(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[7..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        return Some(Err(msg));
    }

    if code.starts_with("~warn(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[6..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        println!("\x1B[93m⚠ {}\x1B[0m", msg);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~info(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[6..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        println!("\x1B[96mℹ {}\x1B[0m", msg);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~success(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[9..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        println!("\x1B[92m✓ {}\x1B[0m", msg);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~question(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[10..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        print!("\x1B[95m? {} \x1B[0m", msg);
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        return Some(match std::io::stdin().read_line(&mut input) {
            Ok(_) => Ok(Value::Text(input.trim().to_string())),
            Err(e) => Err(e.to_string()),
        });
    } else if code.starts_with("~confirm(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[9..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let msg = vm.interp(&val);
        print!("{} (y/n) ", msg);
        let _ = std::io::stdout().flush();
        let mut input = String::new();
        return Some(match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                let ok = input.trim().to_lowercase().starts_with('y');
                Ok(Value::Bool(ok))
            }
            Err(e) => Err(e.to_string()),
        });
    } else if code.starts_with("~prompt(") && code.ends_with(')') {
        return resolve(vm, &format!("~in{}", &code[8..]));
    }

    // Одиночные значения секции INI и простое хранилище пар ключ значение держим здесь чтобы настройки читались единообразно

    if code.starts_with("~config(") && code.ends_with(')') {
        let key = VM::tq(&code[8..code.len() - 1]);
        let v = vm.config.get(&key).cloned().unwrap_or_default();
        vm.echo(format!("{}", v));
        return Some(Ok(Value::Text(v)));
    }

    if code.starts_with("~setconfig(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let key = VM::tq(&p[0]);
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v.as_text().unwrap_or_default(),
                Err(e) => return Some(Err(e)),
            };
            vm.config.insert(key, val);
            return Some(Ok(Value::Bool(true)));
        }
    }

    // Файл читаем по секциям с именем в скобках и парам вида ключ равно значение поэтому строки вне секций пропускаем

    if code.starts_with("~ini_load(") && code.ends_with(')') {
        let file = VM::tq(&code[10..code.len() - 1]);
        match fs::read_to_string(&file) {
            Ok(text) => {
                vm.ini_data.clear();
                let mut cur = String::new();
                for line in text.lines() {
                    let line = line.trim();
                    if line.starts_with('[') && line.ends_with(']') {
                        cur = line[1..line.len() - 1].to_string();
                        vm.ini_data.entry(cur.clone()).or_insert_with(HashMap::new);
                    } else if let Some((k, v)) = line.split_once('=') {
                        if !cur.is_empty() {
                            vm.ini_data.entry(cur.clone())
                                .or_insert_with(HashMap::new)
                                .insert(k.trim().to_string(), v.trim().to_string());
                        }
                    }
                }
                println!("INI loaded: {} sections", vm.ini_data.len());
                return Some(Ok(Value::Bool(true)));
            }
            Err(e) => return Some(Err(e.to_string())),
        }
    }

    if code.starts_with("~ini_save(") && code.ends_with(')') {
        let file = VM::tq(&code[10..code.len() - 1]);
        let mut out = String::new();
        for (sec, keys) in &vm.ini_data {
            out.push_str(&format!("[{}]\n", sec));
            for (k, v) in keys {
                out.push_str(&format!("{} = {}\n", k, v));
            }
            out.push('\n');
        }
        match fs::write(&file, out) {
            Ok(_) => return Some(Ok(Value::Bool(true))),
            Err(e) => return Some(Err(e.to_string())),
        }
    }

    if code.starts_with("~ini_get(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 2 {
            let sec = VM::tq(&p[0]);
            let key = VM::tq(&p[1]);
            if let Some(s) = vm.ini_data.get(&sec) {
                if let Some(v) = s.get(&key) {
                    vm.echo(format!("{}", v));
                    return Some(Ok(Value::Text(v.clone())));
                }
            }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~ini_set(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 3 {
            let sec = VM::tq(&p[0]);
            let key = VM::tq(&p[1]);
            let val = match vm.eval_value(&p[2]) {
                Ok(v) => v.as_text().unwrap_or_default(),
                Err(e) => return Some(Err(e)),
            };
            vm.ini_data.entry(sec).or_insert_with(HashMap::new).insert(key, val);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~ini_del(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 2 {
            let sec = VM::tq(&p[0]);
            let key = VM::tq(&p[1]);
            if let Some(s) = vm.ini_data.get_mut(&sec) {
                let ok = s.remove(&key).is_some();
                return Some(Ok(Value::Bool(ok)));
            }
            return Some(Ok(Value::Bool(false)));
        }
    }

    if code.starts_with("~ini_keys(") && code.ends_with(')') {
        let sec = VM::tq(&code[10..code.len() - 1]);
        if let Some(s) = vm.ini_data.get(&sec) {
            let list: Vec<Value> = s.keys().map(|k| Value::Text(k.clone())).collect();
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
        return Some(Ok(Value::List(vec![])));
    }

    if code == "~ini_sections()" {
        let list: Vec<Value> = vm.ini_data.keys().map(|k| Value::Text(k.clone())).collect();
        let list_val = Value::List(list.clone());
        vm.echo(&format!("{}", list_val.display()));
        return Some(Ok(list_val));
    }

    // Весь файл держим в памяти и пишем обратно целиком по команде kv_save чтобы простое хранилище работало без базы

    if code.starts_with("~kv_open(") && code.ends_with(')') {
        let file = VM::tq(&code[9..code.len() - 1]);
        vm.kv_file = Some(file.clone());
        vm.kv_data.clear();
        if let Ok(text) = fs::read_to_string(&file) {
            for line in text.lines() {
                if let Some((k, v)) = line.split_once('\t') {
                    vm.kv_data.insert(k.to_string(), v.to_string());
                }
            }
        }
        println!("KV loaded: {} keys", vm.kv_data.len());
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~kv_get(") && code.ends_with(')') {
        let key = VM::tq(&code[8..code.len() - 1]);
        if let Some(v) = vm.kv_data.get(&key) {
            vm.echo(format!("{}", v));
            return Some(Ok(Value::Text(v.clone())));
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~kv_set(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let key = VM::tq(&p[0]);
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v.as_text().unwrap_or_default(),
                Err(e) => return Some(Err(e)),
            };
            vm.kv_data.insert(key, val);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~kv_del(") && code.ends_with(')') {
        let key = VM::tq(&code[8..code.len() - 1]);
        let ok = vm.kv_data.remove(&key).is_some();
        return Some(Ok(Value::Bool(ok)));
    }

    if code == "~kv_keys()" {
        let list: Vec<Value> = vm.kv_data.keys().map(|k| Value::Text(k.clone())).collect();
        let list_val = Value::List(list.clone());
        vm.echo(&format!("{}", list_val.display()));
        return Some(Ok(list_val));
    }

    if code == "~kv_save()" {
        if let Some(ref file) = vm.kv_file.clone() {
            let out: String = vm.kv_data.iter().map(|(k, v)| format!("{}\t{}\n", k, v)).collect();
            match fs::write(file, out) {
                Ok(_) => return Some(Ok(Value::Bool(true))),
                Err(e) => return Some(Err(e.to_string())),
            }
        }
        return Some(Err("KV file not opened".to_string()));
    }

    // Адрес режем на схему хост путь и строку запроса в словарь где плюсы и проценты уже раскрыты чтобы разбор вызывался одной командой

    fn url_encode(s: &str) -> String {
        s.chars().map(|c| {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            }
        }).collect()
    }

    fn url_decode(s: &str) -> String {
        let mut out = String::new();
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                if let Ok(pair) = std::str::from_utf8(&bytes[i+1..i+3]) {
                    if let Ok(n) = u8::from_str_radix(pair, 16) {
                        out.push(n as char);
                        i += 3;
                        continue;
                    }
                }
            } else if bytes[i] == b'+' {
                out.push(' ');
            } else {
                out.push(bytes[i] as char);
            }
            i += 1;
        }
        out
    }

    if code.starts_with("~urlparse(") && code.ends_with(')') {
        let s = match vm.eval_value(&code[10..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let (scheme, rest) = s.split_once("://").unwrap_or(("", s.as_str()));
        let (host, path) = rest.split_once('/').map(|(h, p)| (h, format!("/{}", p))).unwrap_or((rest, "/".to_string()));
        let d = Value::Dict(std::collections::HashMap::from([
            ("scheme".to_string(), Value::Text(scheme.to_string())),
            ("host".to_string(), Value::Text(host.to_string())),
            ("path".to_string(), Value::Text(path)),
        ]));
        vm.echo(format!("{}", d.display()));
        return Some(Ok(d));
    }

    if code.starts_with("~queryparse(") && code.ends_with(')') {
        let s = match vm.eval_value(&code[12..code.len() - 1]) {
            Ok(v) => v.as_text().unwrap_or_default(),
            Err(e) => return Some(Err(e)),
        };
        let s = s.trim_start_matches('?');
        let dict: std::collections::HashMap<String, Value> = s
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .map(|(k, v)| (url_decode(k), Value::Text(url_decode(v))))
            .collect();
        let d = Value::Dict(dict);
        vm.echo(format!("{}", d.display()));
        return Some(Ok(d));
    }

    if code.starts_with("~querystringify(") && code.ends_with(')') {
        let name = code[16..code.len() - 1].trim();
        if let Some(Value::Dict(dict)) = vm.get_var(name) {
            let s = dict
                .iter()
                .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(&v.display())))
                .collect::<Vec<_>>()
                .join("&");
            vm.echo(format!("{}", s));
            return Some(Ok(Value::Text(s)));
        }
    }

    // Настоящей параллельности здесь нет поэтому команды выполняем сразу друг за другом как эмуляцию чтобы скрипт не усложнялся

    if code.starts_with("~thread(") && code.ends_with(')') {
        let cmd = &code[8..code.len() - 1];
        vm.thread_counter += 1;
        let id = vm.thread_counter;
        vm.thread_log.push(id);
        // Тело выполняем сразу так как это эмуляция а не настоящий поток и обычно там лежит команда
        // Простое значение считаем через eval_value чтобы не падать с ошибкой про неизвестную команду
        let res = crate::main::eval::evaluate(vm, cmd).or_else(|e| {
            if e.contains("Unknown command") {
                vm.eval_value(cmd).map(|_| Value::None).map_err(|_| e)
            } else {
                Err(e)
            }
        });
        match res {
            Ok(_) => {
                vm.echo(format!("[thread {} completed]", id));
                return Some(Ok(Value::Number(id as f64)));
            }
            Err(e) => return Some(Err(e)),
        }
    }

    if code == "~threads()" {
        let list: Vec<Value> = vm.thread_log.iter().map(|&n| Value::Number(n as f64)).collect();
        let list_val = Value::List(list.clone());
        vm.echo(list_val.display());
        return Some(Ok(list_val));
    }

    if code.starts_with("~thread_wait(") && code.ends_with(')') {
        // Потоки идут последовательно поэтому к моменту ожидания работа уже сделана и проверяем лишь что номер известен
        let id = match vm.eval_expr(&code[13..code.len() - 1]) { Ok(v) => v as u64, Err(e) => return Some(Err(e)) };
        if vm.thread_log.contains(&id) {
            return Some(Ok(Value::Bool(true)));
        }
        return Some(Err(format!("thread_wait: unknown thread id {}", id)));
    }

    if code.starts_with("~thread_stop(") && code.ends_with(')') {
        return Some(Err("thread_stop: threads execute sequentially, nothing to stop".to_string()));
    }

    if code == "~channel()" {
        vm.chan_counter += 1;
        vm.echo(vm.chan_counter);
        return Some(Ok(Value::Number(vm.chan_counter as f64)));
    }

    if code.starts_with("~mutex(") && code.ends_with(')') {
        let name = VM::tq(&code[7..code.len() - 1]);
        vm.mutexes.insert(name, false);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~mutex_lock(") && code.ends_with(')') {
        let name = VM::tq(&code[12..code.len() - 1]);
        vm.mutexes.insert(name, true);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~mutex_unlock(") && code.ends_with(')') {
        let name = VM::tq(&code[14..code.len() - 1]);
        vm.mutexes.insert(name, false);
        return Some(Ok(Value::Bool(true)));
    }

    // Справку переменные историю и память читаем здесь чтобы команды состояния лежали в одном месте

    if code == "~exporthelp" || code == "~exporthelp()" {
        match fs::write("help.txt", "TreeKraft u1\nhttps://treekraft.tilda.ws/\n") {
            Ok(_) => {
                println!("Saved to help.txt");
                return Some(Ok(Value::Bool(true)));
            }
            Err(e) => return Some(Err(e.to_string())),
        }
    }

    if code == "~credits()" || code == "~credits" {
        println!("TreeKraft Team — https://treekraft.tilda.ws/");
        return Some(Ok(Value::None));
    }

    if code == "~vars" || code == "~vars()" {
        vm.print_vars_table();
        return Some(Ok(Value::None));
    }

    if code == "~trace on" {
        vm.trace = true;
        return Some(Ok(Value::Bool(true)));
    }
    if code == "~trace off" {
        vm.trace = false;
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~noecho on" || code == "~noecho" {
        vm.noecho = true;
        return Some(Ok(Value::Bool(true)));
    }
    if code == "~noecho off" {
        vm.noecho = false;
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~history" {
        if vm.history.is_empty() {
            println!("  (empty)");
        } else {
            for (i, cmd) in vm.history.iter().enumerate() {
                println!("  {:>3}: {}", i + 1, cmd);
            }
        }
        return Some(Ok(Value::None));
    }

    if code == "~historyclear()" {
        vm.history.clear();
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~savehistory(") && code.ends_with(')') {
        let path = VM::tq(&code[13..code.len() - 1]);
        match fs::write(&path, vm.history.join("\n")) {
            Ok(_) => return Some(Ok(Value::Bool(true))),
            Err(e) => return Some(Err(e.to_string())),
        }
    }

    if code.starts_with("~loadhistory(") && code.ends_with(')') {
        let path = VM::tq(&code[13..code.len() - 1]);
        match fs::read_to_string(&path) {
            Ok(t) => {
                vm.history = t.lines().map(|s| s.to_string()).collect();
                return Some(Ok(Value::Bool(true)));
            }
            Err(e) => return Some(Err(e.to_string())),
        }
    }

    if code.starts_with("~backup(") && code.ends_with(')') {
        return resolve(vm, &format!("~save{}", &code[8..]));
    }

    if code.starts_with("~restore(") && code.ends_with(')') {
        return resolve(vm, &format!("~load{}", &code[9..]));
    }

    if code == "~mem" {
        let vars: usize = vm.scopes.iter().map(|s| s.len()).sum();
        println!(
            "Variables: {}\nFunctions: {}\nStacks: {}\nQueues: {}\nSets: {}\nCache: {}\nSprites: {}\nAnimations: {}\nImages: {}\nTables: {}",
            vars,
            vm.functions.len(),
            vm.stacks.len(),
            vm.queues.len(),
            vm.sets.len(),
            vm.cache.len(),
            vm.sprites.len(),
            vm.animations.len(),
            vm.images.len(),
            vm.tables.len()
        );
        return Some(Ok(Value::Number(vars as f64)));
    }

    if code == "~memclear" {
        if let Some(g) = vm.scopes.first_mut() {
            g.clear();
        }
        vm.constants.clear();
        println!("Memory cleared");
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~reload" {
        *vm = VM::new();
        println!("TreeKraft reloaded");
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~funcs()" {
        for k in vm.functions.keys() {
            println!("  {}", k);
        }
        return Some(Ok(Value::None));
    }

    if code == "~consts()" {
        let mut c: Vec<_> = vm.constants.iter().collect();
        c.sort();
        for k in c {
            println!("  {}", k);
        }
        return Some(Ok(Value::None));
    }

    if code == "~clear()" || code == "~clear" {
        return resolve(vm, "~cls");
    }

    if code == "~reset()" {
        return resolve(vm, "~reload");
    }

    // Старт запоминает момент а стоп считает миллисекунды чтобы отчет можно было отсортировать от долгого к быстрому
    if code.starts_with("~profile_start(") && code.ends_with(')') || code == "~profile_start" {
        let name = if code.len() > 15 { VM::tq(&code[15..code.len()-1]) } else { "_profile".to_string() };
        vm.timers.insert(format!("_p_{}", name), std::time::Instant::now());
        println!("Profile '{}' started", name); return Some(Ok(Value::None));
    }
    if code.starts_with("~profile_stop(") && code.ends_with(')') || code == "~profile_stop" {
        let name = if code.len() > 14 { VM::tq(&code[14..code.len()-1]) } else { "_profile".to_string() };
        let key = format!("_p_{}", name);
        if let Some(start) = vm.timers.remove(&key) {
            let ms = start.elapsed().as_millis() as f64; vm.echo(format!("{}: {}ms", name, Value::fmt_num(ms)));
            return Some(Ok(Value::Number(ms)));
        } else {
            return Some(Err(format!("Profile '{}' not started", name)));
        }
    }
    if code.starts_with("~profile_report(") && code.ends_with(')') || code == "~profile_report" {
        let mut entries: Vec<(String, f64)> = vm.timers.iter().filter(|(k,_)| k.starts_with("_p_")).filter_map(|(k, v)| { let ms = v.elapsed().as_millis() as f64; Some((k[3..].to_string(), ms)) }).collect();
        entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        println!("\n  Profile Report");
        println!("  ─────────────────────────────");
        for (nm, ms) in &entries { println!("  {:<20} {:>8}ms", nm, Value::fmt_num(*ms)); }
        if entries.is_empty() { println!("  (no profiles)"); }
        return Some(Ok(Value::None));
    }

    // Первый вызов запускает часы а повторный показывает прошедшие миллисекунды чтобы замер делался одной командой
    if code.starts_with("~stopwatch(") && code.ends_with(')') || code == "~stopwatch" {
        let name = if code.len() > 11 { VM::tq(&code[11..code.len()-1]) } else { "_sw".to_string() };
        if let Some(start) = vm.timers.get(&format!("_sw_{}", name)) {
            let ms = start.elapsed().as_millis() as f64; vm.echo(format!("{}", Value::fmt_num(ms)));
            return Some(Ok(Value::Number(ms)));
        } else {
            vm.timers.insert(format!("_sw_{}", name), std::time::Instant::now());
            vm.echo("0"); return Some(Ok(Value::Number(0.0)));
        }
    }

    if code == "~funcs" || code == "~funcs()" { for k in vm.functions.keys() { println!("  {}", k); } return Some(Ok(Value::None)); }
    if code == "~consts" || code == "~consts()" { let mut c: Vec<_> = vm.constants.iter().collect(); c.sort(); for k in c { println!("  {}", k); } return Some(Ok(Value::None)); }
    if code == "~modules" || code == "~modules()" { for k in vm.modules.keys() { println!("  {}", k); } return Some(Ok(Value::None)); }
    if code.starts_with("~module_vars(") && code.ends_with(')') { let name = code[13..code.len()-1].trim(); if let Some(m) = vm.modules.get(name) { for (k, v) in m { println!("  {} = {}", k, v.display()); } } return Some(Ok(Value::None)); }
    if code == "~env" || code == "~env()" { for (k, v) in std::env::vars() { println!("  {}={}", k, v); } return Some(Ok(Value::None)); }
    if code.starts_with("~getenv(") && code.ends_with(')') { let k = VM::tq(&code[8..code.len()-1]); let v = std::env::var(&k).unwrap_or_default(); vm.echo(format!("{}", v)); return Some(Ok(Value::Text(v))); }
    if code.starts_with("~setenv(") && code.ends_with(')') { let p = VM::split_top_level(&code[8..code.len()-1], ','); if p.len() >= 2 { let k = VM::tq(&p[0]); let v = VM::tq(&p[1]); std::env::set_var(k, v); } return Some(Ok(Value::None)); }
    if code.starts_with("~which(") && code.ends_with(')') { let cmd = VM::tq(&code[7..code.len()-1]); let r = std::process::Command::new("where").arg(&cmd).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default(); vm.echo(format!("{}", r)); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~echo_on(") && code.ends_with(')') || code == "~echo_on" { vm.noecho = false; return Some(Ok(Value::None)); }
    if code.starts_with("~echo_off(") && code.ends_with(')') || code == "~echo_off" { vm.noecho = true; return Some(Ok(Value::None)); }
    if code.starts_with("~bool(") && code.ends_with(')') { let v = vm.eval_value(&code[6..code.len()-1]).ok()?; vm.echo(format!("{}", v.truthy())); return Some(Ok(Value::Bool(v.truthy()))); }
    if code.starts_with("~not(") && code.ends_with(')') { let v = vm.eval_value(&code[5..code.len()-1]).ok()?; vm.echo(format!("{}", !v.truthy())); return Some(Ok(Value::Bool(!v.truthy()))); }
    if code.starts_with("~eq(") && code.ends_with(')') { let p = VM::split_top_level(&code[4..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_value(&p[0]).ok()?.display(); let b = vm.eval_value(&p[1]).ok()?.display(); vm.echo(format!("{}", a == b)); return Some(Ok(Value::Bool(a == b))); } else { return None; } }
    if code.starts_with("~neq(") && code.ends_with(')') { let p = VM::split_top_level(&code[5..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_value(&p[0]).ok()?.display(); let b = vm.eval_value(&p[1]).ok()?.display(); vm.echo(format!("{}", a != b)); return Some(Ok(Value::Bool(a != b))); } else { return None; } }
    if code.starts_with("~gt(") && code.ends_with(')') { let p = VM::split_top_level(&code[4..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_expr(&p[0]).ok()?; let b = vm.eval_expr(&p[1]).ok()?; vm.echo(format!("{}", a > b)); return Some(Ok(Value::Bool(a > b))); } else { return None; } }
    if code.starts_with("~lt(") && code.ends_with(')') { let p = VM::split_top_level(&code[4..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_expr(&p[0]).ok()?; let b = vm.eval_expr(&p[1]).ok()?; vm.echo(format!("{}", a < b)); return Some(Ok(Value::Bool(a < b))); } else { return None; } }
    if code.starts_with("~gte(") && code.ends_with(')') { let p = VM::split_top_level(&code[5..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_expr(&p[0]).ok()?; let b = vm.eval_expr(&p[1]).ok()?; vm.echo(format!("{}", a >= b)); return Some(Ok(Value::Bool(a >= b))); } else { return None; } }
    if code.starts_with("~lte(") && code.ends_with(')') { let p = VM::split_top_level(&code[5..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_expr(&p[0]).ok()?; let b = vm.eval_expr(&p[1]).ok()?; vm.echo(format!("{}", a <= b)); return Some(Ok(Value::Bool(a <= b))); } else { return None; } }
    if code.starts_with("~and(") && code.ends_with(')') { let p = VM::split_top_level(&code[5..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_value(&p[0]).ok()?.truthy(); let b = vm.eval_value(&p[1]).ok()?.truthy(); vm.echo(format!("{}", a && b)); return Some(Ok(Value::Bool(a && b))); } else { return None; } }
    if code.starts_with("~or(") && code.ends_with(')') { let p = VM::split_top_level(&code[4..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_value(&p[0]).ok()?.truthy(); let b = vm.eval_value(&p[1]).ok()?.truthy(); vm.echo(format!("{}", a || b)); return Some(Ok(Value::Bool(a || b))); } else { return None; } }
    if code.starts_with("~xor(") && code.ends_with(')') { let p = VM::split_top_level(&code[5..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_value(&p[0]).ok()?.truthy(); let b = vm.eval_value(&p[1]).ok()?.truthy(); vm.echo(format!("{}", a ^ b)); return Some(Ok(Value::Bool(a ^ b))); } else { return None; } }
    if code.starts_with("~ternary(") && code.ends_with(')') { let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() >= 3 { let cond = vm.eval_cond(&p[0]).ok()?; let r = if cond { vm.eval_value(&p[1]).ok()? } else { vm.eval_value(&p[2]).ok()? }; vm.echo(format!("{}", r.display())); return Some(Ok(r)); } else { return None; } }

    if code == "~error_info" || code == "~error_info()" {
        if vm.last_error.is_empty() { vm.echo("  (no error)"); } else { vm.echo(format!("{}", vm.last_error)); }
        return Some(Ok(Value::Text(vm.last_error.clone())));
    }
    if code == "~error_clear" || code == "~error_clear()" { vm.last_error.clear(); return Some(Ok(Value::None)); }
    if code == "~backtrace" || code == "~backtrace()" {
        if vm.last_error.is_empty() { println!("  (no error)"); }
        else { vm.echo(format!("{}", vm.last_error)); }
        return Some(Ok(Value::Text(vm.last_error.clone())));
    }

    // За переменными следим через наблюдение а стек вызовов и точки останова помним чтобы было видно состояние
    if code.starts_with("~watch(") && code.ends_with(')') {
        let name = VM::tq(&code[7..code.len()-1]);
        if !vm.watches.contains(&name) { vm.watches.push(name.clone()); }
        let v = vm.get_var(&name).unwrap_or(Value::None);
        vm.echo(format!("  watch {} = {}", name, v.display()));
        return Some(Ok(v));
    }
    if code.starts_with("~unwatch(") && code.ends_with(')') {
        let name = VM::tq(&code[9..code.len()-1]);
        vm.watches.retain(|w| w != &name);
        return Some(Ok(Value::None));
    }
    if code == "~watches" || code == "~watches()" {
        if vm.watches.is_empty() { println!("  (no watches)"); }
        for name in &vm.watches {
            let v = vm.get_var(name).unwrap_or(Value::None);
            println!("  {} = {}", name, v.display());
        }
        return Some(Ok(Value::None));
    }
    if code == "~call_stack" || code == "~call_stack()" {
        if vm.call_stack.is_empty() { println!("  (empty call stack)"); }
        for (i, f) in vm.call_stack.iter().enumerate() {
            vm.echo(format!("  #{} {}", i, f));
        }
        return Some(Ok(Value::List(vm.call_stack.iter().map(|s| Value::Text(s.clone())).collect())));
    }
    if code.starts_with("~breakpoint(") && code.ends_with(')') {
        let name = VM::tq(&code[12..code.len()-1]);
        vm.breakpoints.push(name.clone());
        println!("  breakpoint set: {}", name);
        return Some(Ok(Value::None));
    }
    if code == "~continue" || code == "~continue()" {
        return Some(Ok(Value::None));
    }

    // Опасные команды режем а разрешенные держим в списке allow чтобы скрипт не мог выйти за границы
    if code == "~sandbox_on" || code == "~sandbox_on()" {
        vm.sandbox_on = true; println!("Sandbox ON"); Some(Ok(Value::None))
    } else if code == "~sandbox_off" || code == "~sandbox_off()" {
        if vm.sandbox_on {
            // Выход из песочницы разрешаем только через список allow чтобы нельзя было сбежать незаметно
            if vm.sandbox_allow.contains("sandbox_off") { vm.sandbox_on = false; println!("Sandbox OFF"); Some(Ok(Value::None)) }
            else { Some(Err("Sandbox: sandbox_off is blocked".to_string())) }
        } else { vm.sandbox_on = false; Some(Ok(Value::None)) }
    } else if code == "~sandbox_status" || code == "~sandbox_status()" {
        let s = if vm.sandbox_on { "on" } else { "off" };
        vm.echo(format!("Sandbox: {}", s));
        Some(Ok(Value::Text(s.to_string())))
    } else if code.starts_with("~sandbox_allow(") && code.ends_with(')') {
        let cmd = VM::tq(&code[16..code.len()-1]);
        vm.sandbox_allow.insert(cmd.clone());
        println!("  sandbox allows: {}", cmd);
        Some(Ok(Value::None))
    } else if code.starts_with("~sandbox_deny(") && code.ends_with(')') {
        let cmd = VM::tq(&code[15..code.len()-1]);
        vm.sandbox_allow.remove(&cmd);
        println!("  sandbox denies: {}", cmd);
        Some(Ok(Value::None))
    } else if code.starts_with("~validate(") && code.ends_with(')') {
        let script = vm.eval_value(&code[10..code.len()-1]).ok()?.as_text()?;
        let dangerous = ["exit","quit","shell","exec","execoutput","shell_cmd","prockill",
            "setenv","set_env","db_open","write_file","append_file","delete_file","delete_dir",
            "tcp_listen","tcp_connect","ws_server","ws_connect","clipboard_set","download",
            "sandbox_off","process"];
        let mut found: Vec<String> = Vec::new();
        for d in dangerous {
            let pattern = format!("~{}", d);
            if script.contains(&pattern) || script.split(|c: char| !c.is_alphanumeric() && c != '_').any(|w| w == d) {
                found.push(d.to_string());
            }
        }
        if found.is_empty() { println!("  OK (no dangerous commands)"); return Some(Ok(Value::Bool(true))); }
        vm.echo(format!("  Dangerous: {}", found.join(", ")));
        Some(Ok(Value::List(found.into_iter().map(Value::Text).collect())))
    } else {
        None
    }
}