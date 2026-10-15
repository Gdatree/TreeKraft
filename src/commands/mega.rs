use crate::main::vm::{VM, Value};
use std::collections::HashMap;

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // Именованные константы отдаем готовыми чтобы не набирать цифры вручную
    if code == "~tau()" || code == "~tau" { vm.echo(std::f64::consts::TAU); return Some(Ok(Value::Number(std::f64::consts::TAU))); }
    if code == "~phi()" || code == "~phi" { vm.echo(1.618033988749895); return Some(Ok(Value::Number(1.618033988749895))); }
    if code == "~golden()" { vm.echo(1.618033988749895); return Some(Ok(Value::Number(1.618033988749895))); }
    if code == "~euler()" { vm.echo(std::f64::consts::E); return Some(Ok(Value::Number(std::f64::consts::E))); }
    if code == "~inf()" { vm.echo("inf"); return Some(Ok(Value::Number(f64::INFINITY))); }

    // Проверка чисел показывает целое или мусор сразу чтобы не плодить проверки в скрипте
    if code.starts_with("~clamp01(") && code.ends_with(')') {
        let n = vm.eval_value(&code[9..code.len()-1]).ok()?.as_number().unwrap_or(0.0).clamp(0.0, 1.0);
        vm.echo(n); return Some(Ok(Value::Number(n)));
    }
    if code.starts_with("~percent(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ',');
        if p.len() == 2 {
            let a = vm.eval_value(&p[0]).ok()?.as_number().unwrap_or(0.0);
            let b = vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(0.0);
            let r = if b != 0.0 { a / b * 100.0 } else { 0.0 };
            vm.echo(r); return Some(Ok(Value::Number(r)));
        }
        return None;
    }
    if code.starts_with("~is_int(") && code.ends_with(')') {
        let n = vm.eval_value(&code[8..code.len()-1]).ok()?.as_number().unwrap_or(0.0);
        let r = n == n.trunc(); vm.echo(r); return Some(Ok(Value::Bool(r)));
    }
    if code.starts_with("~is_finite(") && code.ends_with(')') {
        let n = vm.eval_value(&code[11..code.len()-1]).ok()?.as_number().unwrap_or(0.0);
        let r = n.is_finite(); vm.echo(r); return Some(Ok(Value::Bool(r)));
    }
    if code.starts_with("~is_nan(") && code.ends_with(')') {
        let n = vm.eval_value(&code[8..code.len()-1]).ok()?.as_number().unwrap_or(0.0);
        let r = n.is_nan(); vm.echo(r); return Some(Ok(Value::Bool(r)));
    }
    if code.starts_with("~digit_sum(") && code.ends_with(')') {
        let n = vm.eval_value(&code[11..code.len()-1]).ok()?.as_number().unwrap_or(0.0) as i64;
        let r: i64 = n.abs().to_string().chars().filter_map(|c| c.to_digit(10)).map(|d| d as i64).sum();
        vm.echo(r); return Some(Ok(Value::Number(r as f64)));
    }
    if code.starts_with("~digit_count(") && code.ends_with(')') {
        let n = vm.eval_value(&code[13..code.len()-1]).ok()?.as_number().unwrap_or(0.0) as i64;
        let r = n.abs().to_string().len(); vm.echo(r); return Some(Ok(Value::Number(r as f64)));
    }
    if code.starts_with("~round_to(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() == 2 {
            let n = vm.eval_value(&p[0]).ok()?.as_number().unwrap_or(0.0);
            let d = vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(0.0) as i32;
            let f = 10f64.powi(d);
            let r = (n * f).round() / f;
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return None;
    }
    if code.starts_with("~signum(") && code.ends_with(')') {
        let n = vm.eval_value(&code[8..code.len()-1]).ok()?.as_number().unwrap_or(0.0);
        let r = n.signum(); vm.echo(r); return Some(Ok(Value::Number(r)));
    }
    if code.starts_with("~mul_add(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ',');
        if p.len() == 3 {
            let a = vm.eval_value(&p[0]).ok()?.as_number().unwrap_or(0.0);
            let b = vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(0.0);
            let c = vm.eval_value(&p[2]).ok()?.as_number().unwrap_or(0.0);
            let r = a.mul_add(b, c); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return None;
    }

    // Мелочь для строк повторяем и режем тут чтобы скрипт оставался коротким
    if code.starts_with("~repeat_text(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text().unwrap_or_default();
            let n = vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(1.0) as usize;
            let r = s.repeat(n.min(1000)); vm.echo(&r); return Some(Ok(Value::Text(r)));
        }
        return None;
    }
    if code.starts_with("~contains_any(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len()-1], ',');
        if p.len() >= 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text().unwrap_or_default();
            let r = p[1..].iter().any(|sub| s.contains(&VM::tq(sub)));
            vm.echo(r); return Some(Ok(Value::Bool(r)));
        }
        return None;
    }
    if code.starts_with("~count_letters(") && code.ends_with(')') {
        let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text().unwrap_or_default();
        let r = s.chars().filter(|c| c.is_alphabetic()).count(); vm.echo(r); return Some(Ok(Value::Number(r as f64)));
    }
    if code.starts_with("~count_digits(") && code.ends_with(')') {
        let s = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text().unwrap_or_default();
        let r = s.chars().filter(|c| c.is_ascii_digit()).count(); vm.echo(r); return Some(Ok(Value::Number(r as f64)));
    }
    if code.starts_with("~count_words(") && code.ends_with(')') {
        let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text().unwrap_or_default();
        let r = s.split_whitespace().count(); vm.echo(r); return Some(Ok(Value::Number(r as f64)));
    }
    if code.starts_with("~first_char(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text().unwrap_or_default();
        let r = s.chars().next().map(|c| c.to_string()).unwrap_or_default(); vm.echo(&r); return Some(Ok(Value::Text(r)));
    }
    if code.starts_with("~last_char(") && code.ends_with(')') {
        let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text().unwrap_or_default();
        let r = s.chars().last().map(|c| c.to_string()).unwrap_or_default(); vm.echo(&r); return Some(Ok(Value::Text(r)));
    }
    if code.starts_with("~suffix(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text().unwrap_or_default();
            let n = vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(0.0) as usize;
            let r: String = s.chars().rev().take(n).collect::<Vec<_>>().into_iter().rev().collect();
            vm.echo(&r); return Some(Ok(Value::Text(r)));
        }
        return None;
    }
    if code.starts_with("~prefix(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text().unwrap_or_default();
            let n = vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(0.0) as usize;
            let r: String = s.chars().take(n).collect(); vm.echo(&r); return Some(Ok(Value::Text(r)));
        }
        return None;
    }
    if code.starts_with("~truncate_text(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text().unwrap_or_default();
            let n = vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(0.0) as usize;
            let r: String = if s.chars().count() > n { format!("{}...", s.chars().take(n.max(3)-3).collect::<String>()) } else { s };
            vm.echo(&r); return Some(Ok(Value::Text(r)));
        }
        return None;
    }
    if code.starts_with("~zfill(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len()-1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text().unwrap_or_default();
            let w = vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(0.0) as usize;
            let r = format!("{:0>width$}", s, width = w); vm.echo(&r); return Some(Ok(Value::Text(r)));
        }
        return None;
    }
    if code.starts_with("~sprintf(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ',');
        if p.len() >= 1 {
            let fmt = VM::tq(&p[0]);
            let mut r = fmt.clone();
            for (i, arg) in p[1..].iter().enumerate() {
                let v = vm.eval_value(arg).unwrap_or(Value::None);
                r = r.replace(&format!("%{}", i+1), &v.display());
            }
            r = r.replace("%s", "%1").replace("%d", "%1").replace("%f", "%1");
            vm.echo(&r); return Some(Ok(Value::Text(r)));
        }
        return None;
    }

    // Первый последний и среднее достаем одним вызовом чтобы не возиться с индексами
    if code.starts_with("~first(") && code.ends_with(')') {
        let name = code[7..code.len()-1].trim();
        let v = vm.get_var(name).unwrap_or(Value::None);
        let r = match v {
            Value::List(l) => l.first().cloned().unwrap_or(Value::None),
            Value::Text(t) => Value::Text(t.chars().next().map(|c| c.to_string()).unwrap_or_default()),
            _ => Value::None,
        };
        vm.echo(r.display()); return Some(Ok(r));
    }
    if code.starts_with("~last(") && code.ends_with(')') {
        let name = code[6..code.len()-1].trim();
        let v = vm.get_var(name).unwrap_or(Value::None);
        let r = match v {
            Value::List(l) => l.last().cloned().unwrap_or(Value::None),
            Value::Text(t) => Value::Text(t.chars().last().map(|c| c.to_string()).unwrap_or_default()),
            _ => Value::None,
        };
        vm.echo(format!("{}", r.display())); return Some(Ok(r));
    }
    if code.starts_with("~list_count_true(") && code.ends_with(')') {
        let name = code[17..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => vec![] };
        let r = list.iter().filter(|v| v.truthy()).count(); vm.echo(r); return Some(Ok(Value::Number(r as f64)));
    }
    if code.starts_with("~list_swap(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() == 3 {
            let name = p[0].trim().to_string();
            let i = vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(0.0) as usize;
            let j = vm.eval_value(&p[2]).ok()?.as_number().unwrap_or(0.0) as usize;
            if let Some(Value::List(mut l)) = vm.get_var(&name) {
                if i < l.len() && j < l.len() { l.swap(i, j); vm.set_var(&name, Value::List(l)); return Some(Ok(Value::Bool(true))); }
            }
        }
        return None;
    }
    if code.starts_with("~list_average(") && code.ends_with(')') {
        let name = code[14..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => vec![] };
        let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
        let r = if nums.is_empty() { 0.0 } else { nums.iter().sum::<f64>() / nums.len() as f64 };
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }
    if code.starts_with("~list_nunique(") && code.ends_with(')') {
        let name = code[14..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => vec![] };
        let mut seen = std::collections::HashSet::new();
        let r = list.iter().filter(|v| seen.insert(v.display())).count();
        vm.echo(r); return Some(Ok(Value::Number(r as f64)));
    }

    // Окружение и железо спрашиваем у системы тут чтобы скрипт не звал шелл напрямую
    if code.starts_with("~get_env(") && code.ends_with(')') {
        let name = VM::tq(&code[9..code.len()-1]);
        if let Ok(v) = std::env::var(&name) { vm.echo(&v); return Some(Ok(Value::Text(v))); }
        return Some(Ok(Value::None));
    }
    if code.starts_with("~set_env(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ',');
        if p.len() == 2 {
            let k = VM::tq(&p[0]);
            let v = VM::tq(&p[1]);
            std::env::set_var(&k, &v);
            return Some(Ok(Value::Bool(true)));
        }
        return None;
    }
    if code.starts_with("~current_dir(") || code == "~current_dir" {
        if let Ok(d) = std::env::current_dir() { let s = d.to_string_lossy().to_string(); vm.echo(&s); return Some(Ok(Value::Text(s))); }
        return Some(Ok(Value::None));
    }
    if code.starts_with("~cpu_cores(") || code == "~cpu_cores" {
        let r = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        vm.echo(r); return Some(Ok(Value::Number(r as f64)));
    }
    if code.starts_with("~uptime(") || code == "~uptime" {
        let r = SystemTime_now_secs() / 3600.0; vm.echo(r); return Some(Ok(Value::Number(r)));
    }
    if code.starts_with("~shell_cmd(") && code.ends_with(')') {
        let cmd = VM::tq(&code[11..code.len()-1]);
        let r = crate::main::vm::run_shell(&cmd);
        vm.echo(&r); return Some(Ok(Value::Text(r)));
    }
    if code.starts_with("~clipboard_set(") && code.ends_with(')') {
        let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text().unwrap_or_default();
        #[cfg(windows)]
        { run_powershell_clip(&s); }
        #[cfg(not(windows))]
        { let _ = crate::main::vm::run_shell(&format!("printf '%s' {} | xclip -selection clipboard", s.replace('\'', "'\\''"))); }
        return Some(Ok(Value::None));
    }

    // identity и тип нужны для отладки чтобы увидеть что реально лежит в переменной
    if code.starts_with("~identity(") && code.ends_with(')') {
        let v = vm.eval_value(&code[10..code.len()-1]).unwrap_or(Value::None);
        vm.echo(format!("{}", v.display())); return Some(Ok(v));
    }
    if code.starts_with("~type_of(") && code.ends_with(')') {
        let v = vm.eval_value(&code[9..code.len()-1]).unwrap_or(Value::None);
        let r = v.type_name().to_string(); vm.echo(&r); return Some(Ok(Value::Text(r)));
    }
    if code.starts_with("~noop") || code == "~noop()" {
        return Some(Ok(Value::None));
    }

    // Диапазон с шагом строим списком сразу чтобы не писать цикл
    if code.starts_with("~range_step(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() == 3 {
            let a = vm.eval_value(&p[0]).ok()?.as_number().unwrap_or(0.0) as i64;
            let b = vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(0.0) as i64;
            let s = vm.eval_value(&p[2]).ok()?.as_number().unwrap_or(1.0) as i64;
            let list: Vec<Value> = if s > 0 { (a..b).step_by(s as usize).map(|n| Value::Number(n as f64)).collect() }
                else if s < 0 { (b..a).rev().step_by((-s) as usize).map(|n| Value::Number(n as f64)).collect() } else { vec![] };
            return Some(Ok(Value::List(list)));
        }
        return None;
    }

    // Кольцо держит лимит и вытесняет старые записи чтобы память не росла
    if code.starts_with("~ring_new(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 1 {
            let name = VM::tq(&p[0]);
            let cap = if p.len() >= 2 { vm.eval_value(&p[1]).ok()?.as_number().unwrap_or(100.0) as usize } else { 100 };
            vm.stacks.insert(format!("ring_{}", name), vec![]);
            vm.config.insert(format!("ringcap_{}", name), cap.to_string());
            return Some(Ok(Value::None));
        }
        return None;
    }
    if code.starts_with("~ring_add(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let val = vm.eval_value(&p[1]).unwrap_or(Value::None);
            let cap: usize = vm.config.get(&format!("ringcap_{}", name)).and_then(|s| s.parse().ok()).unwrap_or(100);
            let key = format!("ring_{}", name);
            let list = vm.stacks.entry(key.clone()).or_default();
            list.push(val);
            while list.len() > cap { list.remove(0); }
            return Some(Ok(Value::Bool(true)));
        }
        return None;
    }
    if code.starts_with("~ring_get(") && code.ends_with(')') {
        let name = VM::tq(&code[10..code.len()-1]);
        let list = vm.stacks.get(&format!("ring_{}", name)).cloned().unwrap_or_default();
        let r = Value::List(list); vm.echo(format!("{}", r.display())); return Some(Ok(r));
    }
    if code.starts_with("~ring_size(") && code.ends_with(')') {
        let name = VM::tq(&code[11..code.len()-1]);
        let n = vm.stacks.get(&format!("ring_{}", name)).map(|l| l.len()).unwrap_or(0);
        vm.echo(n); return Some(Ok(Value::Number(n as f64)));
    }

    // Куча держит наименьшее наверху чтобы забирать готовое без сортировки всего списка
    if code.starts_with("~heap_new(") && code.ends_with(')') {
        let name = VM::tq(&code[10..code.len()-1]);
        vm.stacks.insert(format!("heap_{}", name), vec![]);
        return Some(Ok(Value::None));
    }
    if code.starts_with("~heap_push(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let val = vm.eval_value(&p[1]).unwrap_or(Value::None);
            let key = format!("heap_{}", name);
            let list = vm.stacks.entry(key.clone()).or_default();
            list.push(val);
            list.sort_by(|a, b| a.as_number().unwrap_or(f64::MAX).partial_cmp(&b.as_number().unwrap_or(f64::MAX)).unwrap_or(std::cmp::Ordering::Equal));
            return Some(Ok(Value::None));
        }
        return None;
    }
    if code.starts_with("~heap_pop(") && code.ends_with(')') {
        let name = VM::tq(&code[10..code.len()-1]);
        let key = format!("heap_{}", name);
        if let Some(list) = vm.stacks.get_mut(&key) {
            if !list.is_empty() {
                let v = list.remove(0);
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v));
            }
        }
        return Some(Ok(Value::None));
    }
    if code.starts_with("~heap_peek(") && code.ends_with(')') {
        let name = VM::tq(&code[11..code.len()-1]);
        let list = vm.stacks.get(&format!("heap_{}", name)).cloned().unwrap_or_default();
        if let Some(v) = list.first() { vm.echo(v.display()); return Some(Ok(v.clone())); }
        return Some(Ok(Value::None));
    }
    if code.starts_with("~heap_size(") && code.ends_with(')') {
        let name = VM::tq(&code[11..code.len()-1]);
        let n = vm.stacks.get(&format!("heap_{}", name)).map(|l| l.len()).unwrap_or(0);
        vm.echo(n); return Some(Ok(Value::Number(n as f64)));
    }

    // Быстрый поиск делит упорядоченный список пополам чтобы не перебирать все подряд
    if code.starts_with("~sort_fast(") && code.ends_with(')') {
        let name = code[11..code.len()-1].trim().to_string();
        if let Some(Value::List(list)) = vm.get_var_mut(&name) {
            list.sort_by(|a, b| {
                match (a.as_number(), b.as_number()) {
                    (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
                    _ => a.display().cmp(&b.display()),
                }
            });
            return Some(Ok(Value::Bool(true)));
        }
        return Some(Err("List not found".to_string()));
    }
    if code.starts_with("~binary_search(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let target = vm.eval_value(&p[1]).unwrap_or(Value::None);
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => vec![] };
            let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
            let t = target.as_number().unwrap_or(f64::NAN);
            let mut lo = 0i64; let mut hi = nums.len() as i64 - 1; let mut idx = -1i64;
            while lo <= hi {
                let mid = (lo + hi) / 2;
                if nums[mid as usize] < t { lo = mid + 1; }
                else if nums[mid as usize] > t { hi = mid - 1; }
                else { idx = mid; break; }
            }
            vm.echo(idx); return Some(Ok(Value::Number(idx as f64)));
        }
        return None;
    }
    if code.starts_with("~lower_bound(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let target = vm.eval_value(&p[1]).unwrap_or(Value::None).as_number().unwrap_or(0.0);
            let nums: Vec<f64> = vm.get_var(name).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default().into_iter().filter_map(|v| v.as_number()).collect();
            let idx = nums.partition_point(|&x| x < target);
            vm.echo(idx); return Some(Ok(Value::Number(idx as f64)));
        }
        return None;
    }
    if code.starts_with("~upper_bound(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let target = vm.eval_value(&p[1]).unwrap_or(Value::None).as_number().unwrap_or(0.0);
            let nums: Vec<f64> = vm.get_var(name).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default().into_iter().filter_map(|v| v.as_number()).collect();
            let idx = nums.partition_point(|&x| x <= target);
            vm.echo(idx); return Some(Ok(Value::Number(idx as f64)));
        }
        return None;
    }

    // Граф храним парами ребер чтобы обходить связи без тяжелой матрицы
    if code.starts_with("~graph_new(") && code.ends_with(')') {
        let name = VM::tq(&code[11..code.len()-1]);
        vm.stacks.insert(format!("graph_{}", name), vec![]);
        return Some(Ok(Value::None));
    }
    if code.starts_with("~graph_edge(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() >= 3 {
            let name = VM::tq(&p[0]);
            let a = vm.eval_value(&p[1]).unwrap_or(Value::None).display();
            let b = vm.eval_value(&p[2]).unwrap_or(Value::None).display();
            vm.stacks.entry(format!("graph_{}", name)).or_default().push(Value::List(vec![Value::Text(a), Value::Text(b)]));
            return Some(Ok(Value::None));
        }
        return None;
    }
    if code.starts_with("~graph_bfs(") || code.starts_with("~graph_dfs(") {
        let is_bfs = code.starts_with("~graph_bfs(");
        let start = if is_bfs { 11 } else { 11 };
        let p = VM::split_top_level(&code[start..code.len()-1], ',');
        if p.len() >= 1 {
            let name = VM::tq(&p[0]);
            let start_node = if p.len() >= 2 { vm.eval_value(&p[1]).unwrap_or(Value::None).display() } else { "0".to_string() };
            let edges = vm.stacks.get(&format!("graph_{}", name)).cloned().unwrap_or_default();
            // Соседей собираем заранее чтобы обход не искал связи заново на каждом шаге
            let mut adj: Vec<(String, Vec<String>)> = Vec::new();
            for e in &edges {
                if let Value::List(pair) = e {
                    let a = pair.first().map(|v| v.display()).unwrap_or_default();
                    let b = pair.get(1).map(|v| v.display()).unwrap_or_default();
                    if let Some((_, nbrs)) = adj.iter_mut().find(|(n, _)| *n == a) { nbrs.push(b.clone()); } else { adj.push((a.clone(), vec![b.clone()])); }
                    if let Some((_, nbrs)) = adj.iter_mut().find(|(n, _)| *n == b) { nbrs.push(a.clone()); } else { adj.push((b.clone(), vec![a.clone()])); }
                }
            }
            let order = if is_bfs { bfs_order(&adj, &start_node) } else { dfs_order(&adj, &start_node) };
            let list: Vec<Value> = order.into_iter().map(Value::Text).collect();
            let r = Value::List(list); vm.echo(format!("{}", r.display())); return Some(Ok(r));
        }
        return None;
    }
    if code.starts_with("~graph_shortest(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[16..code.len()-1], ',');
        if p.len() >= 3 {
            let name = VM::tq(&p[0]);
            let from = vm.eval_value(&p[1]).unwrap_or(Value::None).display();
            let to = vm.eval_value(&p[2]).unwrap_or(Value::None).display();
            let edges = vm.stacks.get(&format!("graph_{}", name)).cloned().unwrap_or_default();
            let mut adj: Vec<(String, Vec<String>)> = Vec::new();
            for e in &edges {
                if let Value::List(pair) = e {
                    let a = pair.first().map(|v| v.display()).unwrap_or_default();
                    let b = pair.get(1).map(|v| v.display()).unwrap_or_default();
                    if let Some((_, nbrs)) = adj.iter_mut().find(|(n, _)| *n == a) { nbrs.push(b.clone()); } else { adj.push((a.clone(), vec![b.clone()])); }
                    if let Some((_, nbrs)) = adj.iter_mut().find(|(n, _)| *n == b) { nbrs.push(a.clone()); } else { adj.push((b.clone(), vec![a.clone()])); }
                }
            }
            // Кратчайший ищем обходом в ширину где первая встреча уже дает самый короткий путь
            let mut queue = std::collections::VecDeque::new();
            let mut dist: Vec<(String, i64)> = adj.iter().map(|(n, _)| (n.clone(), -1)).collect();
            queue.push_back(from.clone());
            if let Some((_, d)) = dist.iter_mut().find(|(n, _)| *n == from) { *d = 0; }
            while let Some(node) = queue.pop_front() {
                let d = dist.iter().find(|(n, _)| *n == node).map(|(_, d)| *d).unwrap_or(-1);
                if let Some((_, nbrs)) = adj.iter().find(|(n, _)| *n == node) {
                    for nb in nbrs {
                        let nd = dist.iter().find(|(n, _)| *n == *nb).map(|(_, d)| *d).unwrap_or(-1);
                        if nd < 0 {
                            if let Some((_, dd)) = dist.iter_mut().find(|(n, _)| *n == *nb) { *dd = d + 1; }
                            queue.push_back(nb.clone());
                        }
                    }
                }
            }
            let r = dist.iter().find(|(n, _)| *n == to).map(|(_, d)| *d).unwrap_or(-1);
            vm.echo(r); return Some(Ok(Value::Number(r as f64)));
        }
        return None;
    }

    // Размер и копию меряем тут чтобы понять что тянет память
    if code.starts_with("~mem_size(") && code.ends_with(')') {
        let v = vm.eval_value(&code[10..code.len()-1]).unwrap_or(Value::None);
        let bytes = value_approx_size(&v);
        vm.echo(bytes); return Some(Ok(Value::Number(bytes as f64)));
    }
    if code.starts_with("~mem_copy(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() == 2 {
            let src = p[0].trim();
            let dst = p[1].trim();
            if let Some(v) = vm.get_var(src) {
                vm.set_var(dst, v);
                return Some(Ok(Value::Bool(true)));
            }
        }
        return None;
    }
    if code.starts_with("~mem_compare(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() == 2 {
            let a = vm.eval_value(&p[0]).unwrap_or(Value::None);
            let b = vm.eval_value(&p[1]).unwrap_or(Value::None);
            let eq = format!("{:?}", a) == format!("{:?}", b);
            vm.echo(eq); return Some(Ok(Value::Bool(eq)));
        }
        return None;
    }

    None
}

fn bfs_order(adj: &[(String, Vec<String>)], start: &str) -> Vec<String> {
    let mut order = Vec::new();
    let mut visited: Vec<String> = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start.to_string());
    visited.push(start.to_string());
    while let Some(node) = queue.pop_front() {
        order.push(node.clone());
        if let Some((_, nbrs)) = adj.iter().find(|(n, _)| *n == node) {
            for nb in nbrs {
                if !visited.contains(nb) { visited.push(nb.clone()); queue.push_back(nb.clone()); }
            }
        }
    }
    order
}

fn dfs_order(adj: &[(String, Vec<String>)], start: &str) -> Vec<String> {
    let mut order = Vec::new();
    let mut visited: Vec<String> = Vec::new();
    let mut stack = vec![start.to_string()];
    while let Some(node) = stack.pop() {
        if visited.contains(&node) { continue; }
        visited.push(node.clone());
        order.push(node.clone());
        if let Some((_, nbrs)) = adj.iter().find(|(n, _)| *n == node) {
            for nb in nbrs.iter().rev() { stack.push(nb.clone()); }
        }
    }
    order
}

fn value_approx_size(v: &Value) -> usize {
    match v {
        Value::None => 8,
        Value::Bool(_) => 8,
        Value::Number(_) => 8,
        Value::Text(s) => s.len() + 24,
        Value::Lambda(p, b, _) => p.len() * 8 + b.len() + 24,
        Value::List(l) => 24 + l.iter().map(value_approx_size).sum::<usize>(),
        Value::Dict(d) => 24 + d.iter().map(|(k, v)| k.len() + value_approx_size(v)).sum::<usize>(),
    }
}

fn SystemTime_now_secs() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64()
}

#[cfg(windows)]
fn run_powershell_clip(s: &str) {
    use std::io::Write;
    if let Ok(mut child) = std::process::Command::new("powershell")
        .args(["-Command", "Set-Clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(s.as_bytes());
        }
        let _ = child.wait();
    }
}
