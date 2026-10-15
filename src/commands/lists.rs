use crate::main::vm::{VM, Value};
use std::collections::{HashSet, VecDeque};
use rand::Rng;

/// Работа со списками собрана здесь чтобы скрипты оставались короткими без ручных циклов
pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! ln1 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let s = VM::tq(&code[$p.len()..code.len()-1]); if let Some(list) = vm.eval_value(&s).ok().and_then(|v| if let Value::List(l) = v { Some(l) } else { None }) { let r = ($f)(&list) as f64; vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } } }; }
    macro_rules! lb1 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let s = VM::tq(&code[$p.len()..code.len()-1]); if let Some(list) = vm.eval_value(&s).ok().and_then(|v| if let Value::List(l) = v { Some(l) } else { None }) { let r = ($f)(&list); vm.echo(r); return Some(Ok(Value::Bool(r))); } } }; }
    macro_rules! ll1 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let s = VM::tq(&code[$p.len()..code.len()-1]); if let Some(list) = vm.eval_value(&s).ok().and_then(|v| if let Value::List(l) = v { Some(l) } else { None }) { let r = Value::List(($f)(&list)); vm.echo(r.display()); return Some(Ok(r)); } } }; }
    macro_rules! ln2 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let s = VM::tq(&p[0]); if let Some(list) = vm.eval_value(&s).ok().and_then(|v| if let Value::List(l) = v { Some(l) } else { None }) { let n = vm.eval_expr(&p[1]).ok()?; let r = ($f)(&list, n) as f64; vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } } } }; }
    macro_rules! ll2 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let s = VM::tq(&p[0]); if let Some(list) = vm.eval_value(&s).ok().and_then(|v| if let Value::List(l) = v { Some(l) } else { None }) { let n = vm.eval_expr(&p[1]).ok()? as usize; let r = Value::List(($f)(&list, n)); vm.echo(r.display()); return Some(Ok(r)); } } } }; }
    macro_rules! llv { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let s = VM::tq(&p[0]); if let Some(list) = vm.eval_value(&s).ok().and_then(|v| if let Value::List(l) = v { Some(l) } else { None }) { let v = vm.eval_value(&p[1]).ok()?; let r = ($f)(&list, &v); vm.echo(r); return Some(Ok(Value::Bool(r))); } } } }; }
    // Сводки считаются тут чтобы скрипт получал ответ одним вызовом без своих циклов

    ln1!("~list_geomean(", 14, |l: &Vec<Value>| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); if nums.is_empty() { 0.0 } else { let n = nums.len() as f64; nums.iter().fold(1.0, |acc, x| acc * x).powf(1.0 / n) } });
    ln1!("~list_harmonic(", 15, |l: &Vec<Value>| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); if nums.is_empty() { 0.0 } else { let s: f64 = nums.iter().map(|x| 1.0 / x).sum(); if s == 0.0 { 0.0 } else { nums.len() as f64 / s } } });
    ln1!("~list_range_span(", 17, |l: &Vec<Value>| { let a = l.iter().filter_map(|v| v.as_number()).fold(f64::INFINITY, f64::min); let b = l.iter().filter_map(|v| v.as_number()).fold(f64::NEG_INFINITY, f64::max); b - a });
    ln1!("~list_midrange(", 15, |l: &Vec<Value>| { let a = l.iter().filter_map(|v| v.as_number()).fold(f64::INFINITY, f64::min); let b = l.iter().filter_map(|v| v.as_number()).fold(f64::NEG_INFINITY, f64::max); (a + b) / 2.0 });
    ln1!("~list_mad(", 10, |l: &Vec<Value>| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); if nums.is_empty() { 0.0 } else { let mean = nums.iter().sum::<f64>() / nums.len() as f64; nums.iter().map(|x| (x - mean).abs()).sum::<f64>() / nums.len() as f64 } });
    ln1!("~list_rms(", 10, |l: &Vec<Value>| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); if nums.is_empty() { 0.0 } else { (nums.iter().map(|x| x * x).sum::<f64>() / nums.len() as f64).sqrt() } });
    ln1!("~list_mode_num(", 15, |l: &Vec<Value>| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); let mut best = 0.0; let mut bcount = 0usize; for x in &nums { let c = nums.iter().filter(|y| (*y - x).abs() < 1e-9).count(); if c > bcount { bcount = c; best = *x; } } best });
    ln1!("~list_sum_abs(", 14, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).map(|x| x.abs()).sum::<f64>());
    ln1!("~list_sum_sq(", 13, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).map(|x| x * x).sum::<f64>());
    ln1!("~list_count_even(", 17, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).filter(|x| (x.round() as i64) % 2 == 0).count());
    ln1!("~list_count_odd(", 16, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).filter(|x| (x.round() as i64).abs() % 2 == 1).count());
    ln1!("~list_count_neg(", 16, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).filter(|x| *x < 0.0).count());
    ln1!("~list_count_pos(", 16, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).filter(|x| *x > 0.0).count());
    ln1!("~list_count_zero(", 17, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).filter(|x| *x == 0.0).count());
    ln1!("~list_min_abs(", 14, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).fold(f64::INFINITY, |a, x| a.min(x.abs())));
    ln1!("~list_max_abs(", 14, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).fold(0.0f64, |a, x| a.max(x.abs())));
    ln1!("~list_first_pos(", 16, |l: &Vec<Value>| { for v in l { if let Some(n) = v.as_number() { return n; } } 0.0 });
    ln1!("~list_last_pos(", 15, |l: &Vec<Value>| { for v in l.iter().rev() { if let Some(n) = v.as_number() { return n; } } 0.0 });
    ln1!("~list_count_numeric(", 20, |l: &Vec<Value>| l.iter().filter(|v| v.as_number().is_some()).count() as f64);

    lb1!("~list_is_sorted(", 16, |l: &Vec<Value>| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); nums.windows(2).all(|w| w[0] <= w[1]) });
    lb1!("~list_is_desc(", 14, |l: &Vec<Value>| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); nums.windows(2).all(|w| w[0] >= w[1]) });
    lb1!("~list_is_unique2(", 17, |l: &Vec<Value>| l.len() == l.iter().map(|v| v.display()).collect::<std::collections::HashSet<_>>().len());
    lb1!("~list_all_num(", 14, |l: &Vec<Value>| l.iter().all(|v| v.as_number().is_some()));
    lb1!("~list_all_pos(", 14, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).all(|x| x > 0.0) && l.iter().filter_map(|v| v.as_number()).count() == l.len());
    lb1!("~list_any_neg(", 14, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).any(|x| x < 0.0));
    lb1!("~list_has_dups(", 15, |l: &Vec<Value>| l.len() != l.iter().map(|v| v.display()).collect::<std::collections::HashSet<_>>().len());
    lb1!("~list_all_even(", 15, |l: &Vec<Value>| l.iter().filter_map(|v| v.as_number()).all(|x| (x.round() as i64) % 2 == 0));

    ll1!("~list_squared(", 14, |l: &Vec<Value>| l.iter().map(|v| Value::Number(v.as_number().unwrap_or(0.0).powi(2))).collect());
    ll1!("~list_cubed(", 12, |l: &Vec<Value>| l.iter().map(|v| Value::Number(v.as_number().unwrap_or(0.0).powi(3))).collect());
    ll1!("~list_negate(", 13, |l: &Vec<Value>| l.iter().map(|v| Value::Number(-v.as_number().unwrap_or(0.0))).collect());
    ll1!("~list_abs(", 10, |l: &Vec<Value>| l.iter().map(|v| Value::Number(v.as_number().unwrap_or(0.0).abs())).collect());
    ll1!("~list_round(", 12, |l: &Vec<Value>| l.iter().map(|v| Value::Number(v.as_number().unwrap_or(0.0).round())).collect());
    ll1!("~list_floor(", 12, |l: &Vec<Value>| l.iter().map(|v| Value::Number(v.as_number().unwrap_or(0.0).floor())).collect());
    ll1!("~list_ceil(", 11, |l: &Vec<Value>| l.iter().map(|v| Value::Number(v.as_number().unwrap_or(0.0).ceil())).collect());
    ll1!("~list_doubled(", 14, |l: &Vec<Value>| l.iter().map(|v| Value::Number(v.as_number().unwrap_or(0.0) * 2.0)).collect());
    ll1!("~list_halved(", 13, |l: &Vec<Value>| l.iter().map(|v| Value::Number(v.as_number().unwrap_or(0.0) / 2.0)).collect());
    ll1!("~list_sorted_abs(", 17, |l: &Vec<Value>| { let mut v: Vec<Value> = l.clone(); v.sort_by(|a, b| (a.as_number().unwrap_or(0.0).abs()).partial_cmp(&b.as_number().unwrap_or(0.0).abs()).unwrap_or(std::cmp::Ordering::Equal)); v });
    ll1!("~list_cummax(", 13, |l: &Vec<Value>| { let mut m = f64::NEG_INFINITY; l.iter().map(|v| { m = m.max(v.as_number().unwrap_or(0.0)); Value::Number(m) }).collect() });
    ll1!("~list_cummin(", 13, |l: &Vec<Value>| { let mut m = f64::INFINITY; l.iter().map(|v| { m = m.min(v.as_number().unwrap_or(0.0)); Value::Number(m) }).collect() });
    ll1!("~list_adjacent_diff(", 20, |l: &Vec<Value>| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); nums.windows(2).map(|w| Value::Number(w[1] - w[0])).collect() });
    ll1!("~list_sign(", 11, |l: &Vec<Value>| l.iter().map(|v| { let x = v.as_number().unwrap_or(0.0); if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 } }).map(Value::Number).collect());
    ll1!("~list_fraction(", 15, |l: &Vec<Value>| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); let s: f64 = nums.iter().sum(); let s = if s == 0.0 { 1.0 } else { s }; nums.into_iter().map(|x| Value::Number(x / s)).collect() });
    ll1!("~list_even_only(", 16, |l: &Vec<Value>| l.iter().filter(|v| (v.as_number().unwrap_or(0.0).round() as i64) % 2 == 0).cloned().collect());
    ll1!("~list_odd_only(", 15, |l: &Vec<Value>| l.iter().filter(|v| (v.as_number().unwrap_or(0.0).round() as i64).abs() % 2 == 1).cloned().collect());
    ll1!("~list_ints_only(", 16, |l: &Vec<Value>| l.iter().filter(|v| v.as_number().is_some()).cloned().collect());
    ll1!("~list_truthy(", 13, |l: &Vec<Value>| l.iter().filter(|v| v.truthy()).cloned().collect());

    ln2!("~list_count_less_than(", 22, |l: &Vec<Value>, n: f64| l.iter().filter_map(|v| v.as_number()).filter(|x| *x < n).count() as f64);
    ln2!("~list_count_greater_than(", 25, |l: &Vec<Value>, n: f64| l.iter().filter_map(|v| v.as_number()).filter(|x| *x > n).count() as f64);
    ln2!("~list_count_equal(", 18, |l: &Vec<Value>, n: f64| l.iter().filter_map(|v| v.as_number()).filter(|x| (*x - n).abs() < 1e-9).count() as f64);
    ln2!("~list_min_above(", 16, |l: &Vec<Value>, n: f64| l.iter().filter_map(|v| v.as_number()).filter(|x| *x > n).fold(f64::INFINITY, f64::min));
    ln2!("~list_max_below(", 16, |l: &Vec<Value>, n: f64| l.iter().filter_map(|v| v.as_number()).filter(|x| *x < n).fold(f64::NEG_INFINITY, f64::max));
    ln2!("~list_sum_first_n(", 18, |l: &Vec<Value>, n: f64| l.iter().take(n as usize).filter_map(|v| v.as_number()).sum::<f64>());
    ln2!("~list_sum_last_n(", 17, |l: &Vec<Value>, n: f64| { let len = l.len(); l.iter().skip(len.saturating_sub(n as usize)).filter_map(|v| v.as_number()).sum::<f64>() });
    ln2!("~list_frequency_of(", 19, |l: &Vec<Value>, v: f64| l.iter().filter(|x| (x.as_number().unwrap_or(f64::NAN) - v).abs() < 1e-9).count() as f64);

    ll2!("~list_keep_nth(", 15, |l: &Vec<Value>, n: usize| l.iter().enumerate().filter(|(i, _)| (i + 1) % n == 0).map(|(_, v)| v.clone()).collect());
    ll2!("~list_pad_left_num(", 19, |l: &Vec<Value>, n: usize| { let mut r: Vec<Value> = std::iter::repeat(Value::Number(0.0)).take(n.saturating_sub(l.len())).collect(); r.extend(l.iter().cloned()); r });
    ll2!("~list_pad_right_num(", 20, |l: &Vec<Value>, n: usize| { let mut r: Vec<Value> = l.clone(); r.extend(std::iter::repeat(Value::Number(0.0)).take(n.saturating_sub(l.len()))); r });
    ll2!("~list_cycle_n(", 14, |l: &Vec<Value>, n: usize| std::iter::repeat(l).take(n).flat_map(|x| x.clone()).collect());
    ll2!("~list_window_sum(", 17, |l: &Vec<Value>, n: usize| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); nums.windows(n).map(|w| Value::Number(w.iter().sum())).collect() });
    ll2!("~list_window_avg(", 17, |l: &Vec<Value>, n: usize| { let nums: Vec<f64> = l.iter().filter_map(|v| v.as_number()).collect(); nums.windows(n).map(|w| Value::Number(w.iter().sum::<f64>() / n as f64)).collect() });
    ll2!("~list_take_last_n(", 18, |l: &Vec<Value>, n: usize| { let len = l.len(); l.iter().skip(len.saturating_sub(n)).cloned().collect() });
    ll2!("~list_drop_last_n(", 18, |l: &Vec<Value>, n: usize| l.iter().take(l.len().saturating_sub(n)).cloned().collect());
    if code.starts_with("~list(") && code.ends_with(')') {
        let items = match VM::split_top_level(&code[6..code.len() - 1], ',')
            .into_iter()
            .map(|s| vm.eval_value(&s))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        let list = Value::List(items.clone());
        vm.echo(list.display());
        return Some(Ok(list));
    }

    if code.starts_with("~add(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim().to_string();
            let item = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var_mut(&name) {
                list.push(item);
                return Some(Ok(Value::Bool(true)));
            }
            return Some(Err(format!("List '{}' not found", name)));
        }
    }

    if code.starts_with("~get(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let idx = match vm.eval_expr(&p[1]) {
                Ok(v) => v as isize,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var(name) {
                // Отрицательный индекс читаем с конца чтобы не требовать длину списка заранее
                let i = if idx < 0 { list.len() as isize + idx } else { idx };
                if i >= 0 && (i as usize) < list.len() {
                    let v = list[i as usize].clone();
                    vm.echo(v.display());
                    return Some(Ok(v));
                }
                return Some(Err("Index out of range".to_string()));
            }
            return Some(Err(format!("List '{}' not found", name)));
        }
    }

    if code.starts_with("~set(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() == 3 {
            let name = p[0].trim().to_string();
            let idx = match vm.eval_expr(&p[1]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            let val = match vm.eval_value(&p[2]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var_mut(&name) {
                if idx < list.len() {
                    list[idx] = val;
                    return Some(Ok(Value::Bool(true)));
                }
                return Some(Err("Index out of range".to_string()));
            }
            return Some(Err(format!("List '{}' not found", name)));
        }
    }

    if code.starts_with("~remove(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim().to_string();
            let idx = match vm.eval_expr(&p[1]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var_mut(&name) {
                if idx < list.len() {
                    list.remove(idx);
                    return Some(Ok(Value::Bool(true)));
                }
                return Some(Err("Index out of range".to_string()));
            }
            return Some(Err(format!("List '{}' not found", name)));
        }
    }

    if code.starts_with("~insert(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 3 {
            let name = p[0].trim().to_string();
            let idx = match vm.eval_expr(&p[1]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            let val = match vm.eval_value(&p[2]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var_mut(&name) {
                list.insert(idx.min(list.len()), val);
                return Some(Ok(Value::Bool(true)));
            }
            return Some(Err(format!("List '{}' not found", name)));
        }
    }

    if code.starts_with("~pop(") && code.ends_with(')') {
        let name = code[5..code.len() - 1].trim().to_string();
        if let Some(Value::List(list)) = vm.get_var_mut(&name) {
            if let Some(v) = list.pop() {
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v));
            }
            return Some(Err("List is empty".to_string()));
        }
        return Some(Err(format!("List '{}' not found", name)));
    }

    if code.starts_with("~sort(") && code.ends_with(')') {
        let name = code[6..code.len() - 1].trim().to_string();
        if let Some(Value::List(list)) = vm.get_var_mut(&name) {
            list.sort_by(|a, b| {
                match (a.as_number(), b.as_number()) {
                    (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
                    _ => a.display().cmp(&b.display()),
                }
            });
            return Some(Ok(Value::Bool(true)));
        }
        return Some(Err(format!("List '{}' not found", name)));
    }

    if code.starts_with("~sort_desc(") && code.ends_with(')') {
        let name = code[11..code.len() - 1].trim().to_string();
        if let Some(Value::List(list)) = vm.get_var_mut(&name) {
            list.sort_by(|a, b| {
                match (b.as_number(), a.as_number()) {
                    (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
                    _ => b.display().cmp(&a.display()),
                }
            });
            return Some(Ok(Value::Bool(true)));
        }
        return Some(Err(format!("List '{}' not found", name)));
    }

    if code.starts_with("~sort_by_len(") && code.ends_with(')') {
        let name = code[13..code.len() - 1].trim().to_string();
        if let Some(Value::List(list)) = vm.get_var_mut(&name) {
            list.sort_by_key(|v| v.display().len());
            return Some(Ok(Value::Bool(true)));
        }
        return Some(Err(format!("List '{}' not found", name)));
    }

    if code.starts_with("~rev(") && code.ends_with(')') {
        let v = match vm.eval_value(&code[5..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        match v {
            Value::Text(t) => {
                let r: String = t.chars().rev().collect();
                vm.echo(format!("{}", r));
                return Some(Ok(Value::Text(r)));
            }
            Value::List(mut l) => {
                l.reverse();
                let list = Value::List(l.clone());
                vm.echo(&format!("{}", list.display()));
                return Some(Ok(list));
            }
            _ => return Some(Err("~rev expects text or list".to_string())),
        }
    }

    if code.starts_with("~range(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() >= 2 {
            let a = match vm.eval_expr(&p[0]) {
                Ok(v) => v as i64,
                Err(e) => return Some(Err(e)),
            };
            let b = match vm.eval_expr(&p[1]) {
                Ok(v) => v as i64,
                Err(e) => return Some(Err(e)),
            };
            let step = if p.len() > 2 {
                match vm.eval_expr(&p[2]) {
                    Ok(v) => v as i64,
                    Err(e) => return Some(Err(e)),
                }
            } else {
                1
            };
            let list: Vec<Value> = if step > 0 {
                (a..=b).step_by(step as usize).map(|n| Value::Number(n as f64)).collect()
            } else if step < 0 {
                (b..=a).rev().step_by((-step) as usize).map(|n| Value::Number(n as f64)).collect()
            } else {
                vec![]
            };
            let list_val = Value::List(list.clone());
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~slice(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() >= 3 {
            let name = p[0].trim();
            let a = match vm.eval_expr(&p[1]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            let b = match vm.eval_expr(&p[2]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            match vm.get_var(name) {
                Some(Value::List(list)) => {
                    let (a, b) = (a.min(list.len()), b.min(list.len()));
                    let sl: Vec<Value> = if a >= b { vec![] } else { list[a..b].to_vec() };
                    let list_val = Value::List(sl.clone());
                    return Some(Ok(list_val));
                }
                Some(Value::Text(t)) => {
                    let chars: Vec<char> = t.chars().collect();
                    let (a, b) = (a.min(chars.len()), b.min(chars.len()));
                    let out: String = if a >= b { String::new() } else { chars[a..b].iter().collect() };
                    return Some(Ok(Value::Text(out)));
                }
                _ => {}
            }
        }
    }

    if code.starts_with("~join(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() >= 2 {
            let name = p[0].trim();
            let sep = VM::tq(&p[1]);
            if let Some(Value::List(list)) = vm.get_var(name) {
                let out = list.iter().map(|v| v.display()).collect::<Vec<_>>().join(&sep);
                vm.echo(&format!("{}", out));
                return Some(Ok(Value::Text(out)));
            }
        }
    }

    if code.starts_with("~shuffle(") && code.ends_with(')') {
        let name = code[9..code.len() - 1].trim().to_string();
        match vm.get_var(&name) {
            Some(Value::List(mut list)) => {
                let len = list.len();
                for i in (1..len).rev() {
                    let j = rand::thread_rng().gen_range(0..=i);
                    list.swap(i, j);
                }
                vm.set_var(&name, Value::List(list));
                return Some(Ok(Value::Bool(true)));
            }
            Some(Value::Text(t)) => {
                let mut chars: Vec<char> = t.chars().collect();
                let len = chars.len();
                for i in (1..len).rev() {
                    let j = rand::thread_rng().gen_range(0..=i);
                    chars.swap(i, j);
                }
                let out: String = chars.into_iter().collect();
                vm.echo(&format!("{}", out));
                return Some(Ok(Value::Text(out)));
            }
            _ => {}
        }
    }

    if code.starts_with("~choice(") && code.ends_with(')') {
        let name = code[8..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            if !list.is_empty() {
                let v = list[rand::thread_rng().gen_range(0..list.len())].clone();
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v));
            }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~sample(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let n = match vm.eval_expr(&p[1]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var(name) {
                let mut indices: Vec<usize> = (0..list.len()).collect();
                for i in (1..indices.len().min(n + 1)).rev() {
                    let j = rand::thread_rng().gen_range(0..=i);
                    indices.swap(i, j);
                }
                let sample: Vec<Value> = indices[..n.min(list.len())]
                    .iter()
                    .map(|&i| list[i].clone())
                    .collect();
                let sample_val = Value::List(sample.clone());
                vm.echo(&format!("{}", sample_val.display()));
                return Some(Ok(sample_val));
            }
        }
    }

    if code.starts_with("~unique(") && code.ends_with(')') {
        let name = code[8..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let mut seen = HashSet::new();
            let unique: Vec<Value> = list
                .into_iter()
                .filter(|v| seen.insert(v.display()))
                .collect();
            let unique_val = Value::List(unique.clone());
            vm.echo(&format!("{}", unique_val.display()));
            return Some(Ok(unique_val));
        }
    }

    if code.starts_with("~flatten(") && code.ends_with(')') {
        let name = code[9..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let mut flat = Vec::new();
            for v in list {
                match v {
                    Value::List(inner) => flat.extend(inner),
                    other => flat.push(other),
                }
            }
            let flat_val = Value::List(flat.clone());
            vm.echo(&format!("{}", flat_val.display()));
            return Some(Ok(flat_val));
        }
    }

    if code.starts_with("~list_fill(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let n = match vm.eval_expr(&p[0]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let list: Vec<Value> = (0..n).map(|_| val.clone()).collect();
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~list_zip(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let a = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let b = if let Some(Value::List(l)) = vm.get_var(p[1].trim()) { l } else { vec![] };
            let list: Vec<Value> = a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| Value::List(vec![x.clone(), y.clone()]))
                .collect();
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~list_take(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let n = match vm.eval_expr(&p[1]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var(name) {
                let sl: Vec<Value> = list[..n.min(list.len())].to_vec();
                let list_val = Value::List(sl.clone());
                vm.echo(&format!("{}", list_val.display()));
                return Some(Ok(list_val));
            }
        }
    }

    if code.starts_with("~list_drop(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let n = match vm.eval_expr(&p[1]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var(name) {
                let sl: Vec<Value> = list[n.min(list.len())..].to_vec();
                let list_val = Value::List(sl.clone());
                vm.echo(&format!("{}", list_val.display()));
                return Some(Ok(list_val));
            }
        }
    }

    if code.starts_with("~list_head(") && code.ends_with(')') {
        let name = code[11..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            if let Some(v) = list.first() {
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v.clone()));
            }
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~list_last(") && code.ends_with(')') {
        let name = code[11..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            if let Some(v) = list.last() {
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v.clone()));
            }
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~list_tail(") && code.ends_with(')') {
        let name = code[11..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let sl: Vec<Value> = if list.len() > 1 { list[1..].to_vec() } else { vec![] };
            let list_val = Value::List(sl.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~list_init(") && code.ends_with(')') {
        let name = code[11..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let sl: Vec<Value> = if list.len() > 1 { list[..list.len() - 1].to_vec() } else { vec![] };
            let list_val = Value::List(sl.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~list_first(") && code.ends_with(')') {
        return resolve(vm, &format!("~list_head{}", &code[12..]));
    }

    if code.starts_with("~list_rotate(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim().to_string();
            let n = match vm.eval_expr(&p[1]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var_mut(&name) {
                if !list.is_empty() {
                    let n = n % list.len();
                    list.rotate_left(n);
                }
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code.starts_with("~list_chunk(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let n = match vm.eval_expr(&p[1]) {
                Ok(v) => v as usize,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var(name) {
                let chunks: Vec<Value> = list
                    .chunks(n.max(1))
                    .map(|c| Value::List(c.to_vec()))
                    .collect();
                let chunks_val = Value::List(chunks.clone());
                vm.echo(&format!("{}", chunks_val.display()));
                return Some(Ok(chunks_val));
            }
        }
    }

    if code.starts_with("~list_contains(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var(name) {
                let ok = list.iter().any(|v| v.display() == val);
                vm.echo(&format!("{}", ok));
                return Some(Ok(Value::Bool(ok)));
            }
            return Some(Ok(Value::Bool(false)));
        }
    }

    if code.starts_with("~list_index(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var(name) {
                let idx = list
                    .iter()
                    .position(|v| v.display() == val)
                    .map(|i| i as i64)
                    .unwrap_or(-1);
                vm.echo(&format!("{}", idx));
                return Some(Ok(Value::Number(idx as f64)));
            }
            return Some(Ok(Value::Number(-1.0)));
        }
    }

    if code.starts_with("~list_count(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var(name) {
                let n = list.iter().filter(|v| v.display() == val).count();
                vm.echo(format!("{}", n));
                return Some(Ok(Value::Number(n as f64)));
            }
            return Some(Ok(Value::Number(0.0)));
        }
    }

    if code.starts_with("~list_diff(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let a = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let b: HashSet<String> = if let Some(Value::List(l)) = vm.get_var(p[1].trim()) {
                l.iter().map(|v| v.display()).collect()
            } else {
                HashSet::new()
            };
            let diff: Vec<Value> = a
                .into_iter()
                .filter(|v| !b.contains(&v.display()))
                .collect();
            let diff_val = Value::List(diff.clone());
            vm.echo(&format!("{}", diff_val.display()));
            return Some(Ok(diff_val));
        }
    }

    if code.starts_with("~list_inter(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let a = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let b: HashSet<String> = if let Some(Value::List(l)) = vm.get_var(p[1].trim()) {
                l.iter().map(|v| v.display()).collect()
            } else {
                HashSet::new()
            };
            let inter: Vec<Value> = a
                .into_iter()
                .filter(|v| b.contains(&v.display()))
                .collect();
            let inter_val = Value::List(inter.clone());
            vm.echo(&format!("{}", inter_val.display()));
            return Some(Ok(inter_val));
        }
    }

    if code.starts_with("~list_union(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let a = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let b = if let Some(Value::List(l)) = vm.get_var(p[1].trim()) { l } else { vec![] };
            let mut seen = HashSet::new();
            let mut union = Vec::new();
            for v in a.iter().chain(b.iter()) {
                if seen.insert(v.display()) {
                    union.push(v.clone());
                }
            }
            let union_val = Value::List(union.clone());
            vm.echo(&format!("{}", union_val.display()));
            return Some(Ok(union_val));
        }
    }

    if code.starts_with("~list_shift(") && code.ends_with(')') {
        let name = code[12..code.len() - 1].trim().to_string();
        if let Some(Value::List(list)) = vm.get_var_mut(&name) {
            if !list.is_empty() {
                let v = list.remove(0);
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v));
            }
            return Some(Err("List empty".to_string()));
        }
    }

    if code.starts_with("~list_unshift(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim().to_string();
            let v = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(list)) = vm.get_var_mut(&name) {
                list.insert(0, v);
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code.starts_with("~list_removeval(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[16..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim().to_string();
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(mut list)) = vm.get_var(&name) {
                if let Some(idx) = list.iter().position(|v| v.display() == val) {
                    list.remove(idx);
                }
                vm.set_var(&name, Value::List(list));
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code.starts_with("~list_concat(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let a = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let b = if let Some(Value::List(l)) = vm.get_var(p[1].trim()) { l } else { vec![] };
            let mut out = a;
            out.extend(b);
            let out_val = Value::List(out.clone());
            vm.echo(&format!("{}", out_val.display()));
            return Some(Ok(out_val));
        }
    }

    // map/filter/reduce идут целиком за один вызов чтобы скрипт обходился без ручного цикла

    if code.starts_with("~list_map(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let list = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let fn_val = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let mut out = Vec::new();
            for item in list {
                let r = match fn_val.clone() {
                    Value::Lambda(params, body, _) => {
                        match vm.call_lambda(params, body, vec![item], 0) {
                            Ok(v) => v,
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    _ => item,
                };
                out.push(r);
            }
            let out_val = Value::List(out.clone());
            vm.echo(&format!("{}", out_val.display()));
            return Some(Ok(out_val));
        }
    }

    if code.starts_with("~list_filter(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let list = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let fn_val = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let mut out = Vec::new();
            for item in list {
                let keep = match fn_val.clone() {
                    Value::Lambda(params, body, _) => {
                        match vm.call_lambda(params, body, vec![item.clone()], 0) {
                            Ok(v) => v.truthy(),
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    _ => item.truthy(),
                };
                if keep {
                    out.push(item);
                }
            }
            let out_val = Value::List(out.clone());
            vm.echo(&format!("{}", out_val.display()));
            return Some(Ok(out_val));
        }
    }

    if code.starts_with("~list_reduce(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 3 {
            let list = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let fn_val = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let mut acc = match vm.eval_value(&p[2]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            for item in list {
                acc = match fn_val.clone() {
                    Value::Lambda(params, body, _) => {
                        match vm.call_lambda(params, body, vec![acc, item], 0) {
                            Ok(v) => v,
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    _ => acc,
                };
            }
            vm.echo(&format!("{}", acc.display()));
            return Some(Ok(acc));
        }
    }

    if code.starts_with("~list_find(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let list = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let fn_val = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            for item in list {
                let ok = match fn_val.clone() {
                    Value::Lambda(params, body, _) => {
                        match vm.call_lambda(params, body, vec![item.clone()], 0) {
                            Ok(v) => v.truthy(),
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    _ => item.truthy(),
                };
                if ok {
                    vm.echo(&format!("{}", item.display()));
                    return Some(Ok(item));
                }
            }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~list_findall(") && code.ends_with(')') {
        return resolve(vm, &format!("~list_filter{}", &code[14..]));
    }

    if code.starts_with("~list_sortby(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim().to_string();
            let fn_val = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::List(mut list)) = vm.get_var(&name) {
                let mut keyed: Vec<(f64, Value)> = Vec::new();
                for item in list.drain(..) {
                    let k = match fn_val.clone() {
                        Value::Lambda(params, body, _) => {
                            match vm.call_lambda(params, body, vec![item.clone()], 0) {
                                Ok(v) => v.as_number().unwrap_or(0.0),
                                Err(e) => return Some(Err(e)),
                            }
                        }
                        _ => item.as_number().unwrap_or(0.0),
                    };
                    keyed.push((k, item));
                }
                keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                let sorted: Vec<Value> = keyed.into_iter().map(|(_, v)| v).collect();
                vm.set_var(&name, Value::List(sorted));
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    // Статистика считается одним вызовом чтобы скрипт не гонял свои циклы ради суммы и разброса

    fn list_stats(list: &[Value]) -> (f64, f64, f64, f64, f64) {
        let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
        if nums.is_empty() {
            return (0.0, 0.0, 0.0, 0.0, 0.0);
        }
        let sum: f64 = nums.iter().sum();
        let mean = sum / nums.len() as f64;
        let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
        (sum, mean, min, max, variance.sqrt())
    }

    if code.starts_with("~list_sum(") && code.ends_with(')') {
        let name = code[10..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let s: f64 = list.iter().filter_map(|v| v.as_number()).sum();
            vm.echo(format!("{}", s));
            return Some(Ok(Value::Number(s)));
        }
        return Some(Ok(Value::Number(0.0)));
    }

    if code.starts_with("~list_avg(") && code.ends_with(')') {
        let name = code[10..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
            if !nums.is_empty() {
                let avg = nums.iter().sum::<f64>() / nums.len() as f64;
                vm.echo(&format!("{}", avg));
                return Some(Ok(Value::Number(avg)));
            }
        }
        return Some(Ok(Value::Number(0.0)));
    }

    if code.starts_with("~list_min(") && code.ends_with(')') {
        let name = code[10..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let min = list
                .iter()
                .filter_map(|v| v.as_number())
                .fold(f64::INFINITY, f64::min);
            vm.echo(&format!("{}", min));
            return Some(Ok(Value::Number(min)));
        }
        return Some(Ok(Value::Number(0.0)));
    }

    if code.starts_with("~list_max(") && code.ends_with(')') {
        let name = code[10..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let max = list
                .iter()
                .filter_map(|v| v.as_number())
                .fold(f64::NEG_INFINITY, f64::max);
            vm.echo(&format!("{}", max));
            return Some(Ok(Value::Number(max)));
        }
        return Some(Ok(Value::Number(0.0)));
    }

    if code.starts_with("~list_std(") && code.ends_with(')') {
        let name = code[10..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let (_, _, _, _, std) = list_stats(&list);
            vm.echo(&format!("{}", std));
            return Some(Ok(Value::Number(std)));
        }
        return Some(Ok(Value::Number(0.0)));
    }

    if code.starts_with("~stats(") && code.ends_with(')') {
        let name = code[7..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let (sum, mean, min, max, std) = list_stats(&list);
            vm.echo(format!("  sum={} mean={:.4} min={} max={} std={:.4}", sum, mean, min, max, std));
            let d = Value::Dict(std::collections::HashMap::from([
                ("sum".to_string(), Value::Number(sum)),
                ("mean".to_string(), Value::Number(mean)),
                ("min".to_string(), Value::Number(min)),
                ("max".to_string(), Value::Number(max)),
                ("std".to_string(), Value::Number(std)),
            ]));
            return Some(Ok(d));
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~cumsum(") && code.ends_with(')') {
        let name = code[8..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let mut acc = 0.0;
            let cs: Vec<Value> = list
                .iter()
                .map(|v| {
                    acc += v.as_number().unwrap_or(0.0);
                    Value::Number(acc)
                })
                .collect();
            let cs_val = Value::List(cs.clone());
            vm.echo(&format!("{}", cs_val.display()));
            return Some(Ok(cs_val));
        }
    }

    if code.starts_with("~normalize(") && code.ends_with(')') {
        let name = code[11..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
            let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let range = max - min;
            let normalized: Vec<Value> = nums
                .iter()
                .map(|&n| {
                    Value::Number(if range.abs() < 1e-10 { 0.0 } else { (n - min) / range })
                })
                .collect();
            let norm_val = Value::List(normalized.clone());
            vm.echo(&format!("{}", norm_val.display()));
            return Some(Ok(norm_val));
        }
    }

    if code.starts_with("~most_freq(") && code.ends_with(')') {
        let name = code[11..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let mut freq = std::collections::HashMap::new();
            for v in &list {
                *freq.entry(v.display()).or_insert(0usize) += 1;
            }
            if let Some((k, _)) = freq.iter().max_by_key(|(_, &v)| v) {
                let v = list.iter().find(|x| x.display() == *k).cloned().unwrap_or(Value::None);
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v));
            }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~freq_map(") && code.ends_with(')') {
        let name = code[10..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let mut m: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            for v in &list {
                *m.entry(v.display()).or_insert(0) += 1;
            }
            let mut freq: Vec<(String, i64)> = m.into_iter().collect();
            freq.sort_by(|(a, _), (b, _)| a.cmp(b));
            let mut d2 = std::collections::HashMap::new();
            for (k, n) in freq { d2.insert(k, Value::Number(n as f64)); }
            let d = Value::Dict(d2);
            vm.echo(format!("{}", d.display()));
            return Some(Ok(d));
        }
    }

    if code.starts_with("~running_sum(") && code.ends_with(')') {
        let name = code[13..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let sum: f64 = list.iter().map(|v| v.as_number().unwrap_or(0.0)).sum();
            vm.echo(&format!("{}", sum));
            return Some(Ok(Value::Number(sum)));
        }
        return Some(Ok(Value::Number(0.0)));
    }

    // Лямбда нужна чтобы передавать свое правило в map/filter без лишнего кода
    if code.starts_with("~lambda(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ',');
        if p.len() == 2 {
            let params: Vec<String> = VM::split_top_level(&p[0], ',').into_iter().map(|s| s.trim().to_string()).collect();
            let body = p[1].trim().to_string();
            let lam = Value::Lambda(params, body, 0);
            vm.echo(&format!("{}", lam.display()));
            return Some(Ok(lam));
        }
        return None;
    }

    // Словари ищут по имени а не по индексу поэтому храним пары ключей и значений тут

    if code == "~dict()" || code == "~dict" {
        return Some(Ok(Value::Dict(std::collections::HashMap::new())));
    }

    if code.starts_with("~dset(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() >= 3 {
            let name = p[0].trim().to_string();
            let key = vm.eval_value(&p[1]).map(|v| v.display()).unwrap_or_default();
            let val = match vm.eval_value(&p[2]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::Dict(dict)) = vm.get_var_mut(&name) {
                dict.insert(key, val);
                return Some(Ok(Value::Bool(true)));
            }
            return Some(Err(format!("Dict '{}' not found", name)));
        }
    }

    if code.starts_with("~dget(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() >= 2 {
            let name = p[0].trim();
            let key = vm.eval_value(&p[1]).map(|v| v.display()).unwrap_or_default();
            if let Some(Value::Dict(dict)) = vm.get_var(name) {
                if let Some(v) = dict.get(&key) {
                    vm.echo(v.display());
                    return Some(Ok(v.clone()));
                }
                return Some(Err(format!("Key '{}' not found", key)));
            }
        }
    }

    if code.starts_with("~ddel(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() >= 2 {
            let name = p[0].trim().to_string();
            let key = vm.eval_value(&p[1]).map(|v| v.display()).unwrap_or_default();
            if let Some(Value::Dict(dict)) = vm.get_var_mut(&name) {
                let ok = dict.remove(&key).is_some();
                return Some(Ok(Value::Bool(ok)));
            }
        }
    }

    if code.starts_with("~dhas(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() >= 2 {
            let name = p[0].trim();
            let key = vm.eval_value(&p[1]).map(|v| v.display()).unwrap_or_default();
            if let Some(Value::Dict(dict)) = vm.get_var(name) {
                let ok = dict.contains_key(&key);
                vm.echo(&format!("{}", ok));
                return Some(Ok(Value::Bool(ok)));
            }
            return Some(Ok(Value::Bool(false)));
        }
    }

    if code.starts_with("~dkeys(") && code.ends_with(')') {
        let name = code[7..code.len() - 1].trim();
        if let Some(Value::Dict(dict)) = vm.get_var(name) {
            let list: Vec<Value> = dict.iter().map(|(k, _)| Value::Text(k.clone())).collect();
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~dvals(") && code.ends_with(')') {
        let name = code[7..code.len() - 1].trim();
        if let Some(Value::Dict(dict)) = vm.get_var(name) {
            let list: Vec<Value> = dict.iter().map(|(_, v)| v.clone()).collect();
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~dlen(") && code.ends_with(')') {
        let name = code[6..code.len() - 1].trim();
        if let Some(Value::Dict(dict)) = vm.get_var(name) {
            let n = dict.len();
            vm.echo(format!("{}", n));
            return Some(Ok(Value::Number(n as f64)));
        }
    }

    if code.starts_with("~ditems(") && code.ends_with(')') {
        let name = code[8..code.len() - 1].trim();
        if let Some(Value::Dict(dict)) = vm.get_var(name) {
            let list: Vec<Value> = dict
                .iter()
                .map(|(k, v)| Value::List(vec![Value::Text(k.clone()), v.clone()]))
                .collect();
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~dmerge(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let na = p[0].trim().to_string();
            let nb = p[1].trim().to_string();
            // Требуем оба словаря чтобы случайная склейка мусора не пряталась за пустой заглушкой
            let da = match vm.get_var(&na) { Some(Value::Dict(d)) => d, _ => return Some(Err(format!("Dict '{}' not found", na))) };
            let db = match vm.get_var(&nb) { Some(Value::Dict(d)) => d, _ => return Some(Err(format!("Dict '{}' not found", nb))) };
            let mut merged = da;
            for (k, v) in db {
                merged.entry(k).or_insert(v);
            }
            vm.set_var(&na, Value::Dict(merged));
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~dcopy(") && code.ends_with(')') {
        let name = code[7..code.len() - 1].trim();
        if let Some(Value::Dict(dict)) = vm.get_var(name) {
            return Some(Ok(Value::Dict(dict.clone())));
        }
    }

    if code.starts_with("~dclear(") && code.ends_with(')') {
        let name = code[8..code.len() - 1].trim().to_string();
        if let Some(Value::Dict(_)) = vm.get_var(&name) {
            vm.set_var(&name, Value::Dict(std::collections::HashMap::new()));
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~ddefault(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 3 {
            let name = p[0].trim();
            let key = vm.eval_value(&p[1]).map(|v| v.display()).unwrap_or_default();
            if let Some(Value::Dict(dict)) = vm.get_var(name) {
                if let Some(v) = dict.get(&key) {
                    vm.echo(format!("{}", v.display()));
                    return Some(Ok(v.clone()));
                }
            }
            let default = match vm.eval_value(&p[2]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            vm.echo(&format!("{}", default.display()));
            return Some(Ok(default));
        }
    }

    if code.starts_with("~dupdate(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 3 {
            let name = p[0].trim().to_string();
            let key = vm.eval_value(&p[1]).map(|v| v.display()).unwrap_or_default();
            let fn_val = match vm.eval_value(&p[2]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if let Some(Value::Dict(mut dict)) = vm.get_var(&name) {
                let new_val = if let Some(old) = dict.get(&key) {
                    let old = old.clone();
                    match fn_val {
                        Value::Lambda(params, body, _) => {
                            match vm.call_lambda(params, body, vec![old], 0) {
                                Ok(v) => v,
                                Err(e) => return Some(Err(e)),
                            }
                        }
                        other => other,
                    }
                } else { fn_val };
                dict.insert(key, new_val);
                vm.set_var(&name, Value::Dict(dict));
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code.starts_with("~dgroupby(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let list = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let key_fn = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let mut groups: std::collections::HashMap<String, Vec<Value>> = std::collections::HashMap::new();
            for item in list {
                let k = match key_fn.clone() {
                    Value::Lambda(params, body, _) => {
                        match vm.call_lambda(params, body, vec![item.clone()], 0) {
                            Ok(v) => v.display(),
                            Err(e) => return Some(Err(e)),
                        }
                    }
                    _ => item.display(),
                };
                groups.entry(k).or_default().push(item);
            }
            let d = Value::Dict(
                groups.into_iter().map(|(k, v)| (k, Value::List(v))).collect(),
            );
            vm.echo(format!("{}", d.display()));
            return Some(Ok(d));
        }
    }

    if code.starts_with("~dcountby(") && code.ends_with(')') {
        let name = code[10..code.len() - 1].trim();
        if let Some(Value::List(list)) = vm.get_var(name) {
            let mut m: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            for v in &list {
                let k = v.display();
                *m.entry(k).or_insert(0) += 1;
            }
            let d = Value::Dict(
                m.into_iter()
                    .map(|(k, n)| (k, Value::Number(n as f64)))
                    .collect(),
            );
            vm.echo(format!("{}", d.display()));
            return Some(Ok(d));
        }
    }

    if code.starts_with("~dict_from_lists(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len() - 1], ',');
        if p.len() == 2 {
            let keys = if let Some(Value::List(l)) = vm.get_var(p[0].trim()) { l } else { vec![] };
            let vals = if let Some(Value::List(l)) = vm.get_var(p[1].trim()) { l } else { vec![] };
            let dict: std::collections::HashMap<String, Value> = keys
                .iter()
                .zip(vals.iter())
                .map(|(k, v)| (k.display(), v.clone()))
                .collect();
            let d = Value::Dict(dict);
            vm.echo(format!("{}", d.display()));
            return Some(Ok(d));
        }
    }

    if code.starts_with("~dict_invert(") && code.ends_with(')') {
        let name = code[13..code.len() - 1].trim();
        if let Some(Value::Dict(dict)) = vm.get_var(name) {
            let inv: std::collections::HashMap<String, Value> = dict
                .iter()
                .map(|(k, v)| (v.display(), Value::Text(k.clone())))
                .collect();
            let d = Value::Dict(inv);
            vm.echo(format!("{}", d.display()));
            return Some(Ok(d));
        }
    }

    if code.starts_with("~dict_to_list(") && code.ends_with(')') {
        let name = code[14..code.len() - 1].trim();
        if let Some(Value::Dict(dict)) = vm.get_var(name) {
            let list: Vec<Value> = dict
                .iter()
                .map(|(k, v)| Value::List(vec![Value::Text(k.clone()), v.clone()]))
                .collect();
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~keys(") && code.ends_with(')') {
        return resolve(vm, &format!("~dkeys{}", &code[5..]));
    }

    if code.starts_with("~values(") && code.ends_with(')') {
        return resolve(vm, &format!("~dvals{}", &code[7..]));
    }

    // Стек отдает с вершины поэтому удобно держать тут историю и отмены

    if code.starts_with("~stack(") && code.ends_with(')') {
        let name = VM::tq(&code[7..code.len() - 1]);
        vm.stacks.entry(name).or_insert_with(Vec::new);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~push(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim().to_string();
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            vm.stacks.entry(name).or_insert_with(Vec::new).push(val);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~pop_stack(") && code.ends_with(')') {
        let name = code[11..code.len() - 1].trim().to_string();
        if let Some(stack) = vm.stacks.get_mut(&name) {
            if let Some(v) = stack.pop() {
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v));
            }
            return Some(Err(format!("Stack '{}' is empty", name)));
        }
        return Some(Err(format!("Stack '{}' not found", name)));
    }

    if code.starts_with("~peek_stack(") && code.ends_with(')') {
        let name = code[12..code.len() - 1].trim();
        if let Some(stack) = vm.stacks.get(name) {
            if let Some(v) = stack.last() {
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v.clone()));
            }
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~stack_size(") && code.ends_with(')') {
        let name = code[12..code.len() - 1].trim();
        let n = vm.stacks.get(name).map(|s| s.len()).unwrap_or(0);
        vm.echo(format!("{}", n));
        return Some(Ok(Value::Number(n as f64)));
    }

    if code.starts_with("~stack_empty(") && code.ends_with(')') {
        let name = code[13..code.len() - 1].trim();
        let ok = vm.stacks.get(name).map(|s| s.is_empty()).unwrap_or(true);
        vm.echo(&format!("{}", ok));
        return Some(Ok(Value::Bool(ok)));
    }

    // Очередь разбирает задачи по порядку прихода поэтому честно держит порядок обработки

    if code.starts_with("~queue(") && code.ends_with(')') {
        let name = VM::tq(&code[7..code.len() - 1]);
        vm.queues.entry(name).or_insert_with(VecDeque::new);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~enqueue(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim().to_string();
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            vm.queues.entry(name).or_insert_with(VecDeque::new).push_back(val);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~dequeue(") && code.ends_with(')') {
        let name = code[9..code.len() - 1].trim().to_string();
        if let Some(q) = vm.queues.get_mut(&name) {
            if let Some(v) = q.pop_front() {
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v));
            }
            return Some(Err(format!("Queue '{}' is empty", name)));
        }
        return Some(Err(format!("Queue '{}' not found", name)));
    }

    if code.starts_with("~queue_peek(") && code.ends_with(')') {
        let name = code[12..code.len() - 1].trim();
        if let Some(q) = vm.queues.get(name) {
            if let Some(v) = q.front() {
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v.clone()));
            }
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~queue_size(") && code.ends_with(')') {
        let name = code[12..code.len() - 1].trim();
        let n = vm.queues.get(name).map(|q| q.len()).unwrap_or(0);
        vm.echo(format!("{}", n));
        return Some(Ok(Value::Number(n as f64)));
    }

    if code.starts_with("~queue_empty(") && code.ends_with(')') {
        let name = code[13..code.len() - 1].trim();
        let ok = vm.queues.get(name).map(|q| q.is_empty()).unwrap_or(true);
        vm.echo(&format!("{}", ok));
        return Some(Ok(Value::Bool(ok)));
    }

    // Множество хранит только уникальное поэтому вопрос уже видели решается быстрой проверкой

    if code.starts_with("~set_new(") && code.ends_with(')') {
        let name = VM::tq(&code[9..code.len() - 1]);
        vm.sets.insert(name, HashSet::new());
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~set_add(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim().to_string();
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            vm.sets.entry(name).or_insert_with(HashSet::new).insert(val);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~set_remove(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            let ok = vm.sets.get_mut(name).map(|s| s.remove(&val)).unwrap_or(false);
            return Some(Ok(Value::Bool(ok)));
        }
    }

    if code.starts_with("~set_has(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v.display(),
                Err(e) => return Some(Err(e)),
            };
            let ok = vm.sets.get(name).map(|s| s.contains(&val)).unwrap_or(false);
            vm.echo(&format!("{}", ok));
            return Some(Ok(Value::Bool(ok)));
        }
    }

    if code.starts_with("~set_size(") && code.ends_with(')') {
        let name = code[10..code.len() - 1].trim();
        let n = vm.sets.get(name).map(|s| s.len()).unwrap_or(0);
        vm.echo(format!("{}", n));
        return Some(Ok(Value::Number(n as f64)));
    }

    if code.starts_with("~set_list(") && code.ends_with(')') {
        let name = code[10..code.len() - 1].trim();
        if let Some(set) = vm.sets.get(name) {
            let mut list: Vec<Value> = set.iter().map(|s| Value::Text(s.clone())).collect();
            list.sort_by(|a, b| a.display().cmp(&b.display()));
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
        return Some(Ok(Value::List(vec![])));
    }

    if code.starts_with("~set_union(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let a = vm.sets.get(p[0].trim()).cloned().unwrap_or_default();
            let b = vm.sets.get(p[1].trim()).cloned().unwrap_or_default();
            let u: HashSet<String> = a.union(&b).cloned().collect();
            let mut list: Vec<Value> = u.iter().map(|s| Value::Text(s.clone())).collect();
            list.sort_by(|a, b| a.display().cmp(&b.display()));
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~set_inter(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let a = vm.sets.get(p[0].trim()).cloned().unwrap_or_default();
            let b = vm.sets.get(p[1].trim()).cloned().unwrap_or_default();
            let i: HashSet<String> = a.intersection(&b).cloned().collect();
            let mut list: Vec<Value> = i.iter().map(|s| Value::Text(s.clone())).collect();
            list.sort_by(|a, b| a.display().cmp(&b.display()));
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    if code.starts_with("~set_diff(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let a = vm.sets.get(p[0].trim()).cloned().unwrap_or_default();
            let b = vm.sets.get(p[1].trim()).cloned().unwrap_or_default();
            let d: HashSet<String> = a.difference(&b).cloned().collect();
            let mut list: Vec<Value> = d.iter().map(|s| Value::Text(s.clone())).collect();
            list.sort_by(|a, b| a.display().cmp(&b.display()));
            let list_val = Value::List(list.clone());
            vm.echo(&format!("{}", list_val.display()));
            return Some(Ok(list_val));
        }
    }

    // Кэш держит дорогое рядом пока жив срок чтобы не считать то же самое заново

    if code.starts_with("~cache_set(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() >= 2 {
            let key = VM::tq(&p[0]);
            let val = match vm.eval_value(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let expiry = if p.len() > 2 {
                match vm.eval_expr(&p[2]) {
                    Ok(secs) => Some(std::time::Instant::now() + std::time::Duration::from_secs_f64(secs)),
                    Err(e) => return Some(Err(e)),
                }
            } else {
                None
            };
            let ttl = if p.len() > 2 {
                match vm.eval_expr(&p[2]) {
                    Ok(secs) => Some(std::time::Duration::from_secs_f64(secs)),
                    Err(e) => return Some(Err(e)),
                }
            } else {
                None
            };
            vm.cache.insert(key, (val, expiry, ttl));
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~cache_get(") && code.ends_with(')') {
        let key = VM::tq(&code[11..code.len() - 1]);
        if let Some((val, expiry, _)) = vm.cache.get(&key) {
            if expiry.map(|e| std::time::Instant::now() < e).unwrap_or(true) {
                let v = val.clone();
                vm.echo(format!("{}", v.display()));
                return Some(Ok(v));
            } else {
                vm.cache.remove(&key);
            }
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~cache_del(") && code.ends_with(')') {
        let ok = vm.cache.remove(&VM::tq(&code[11..code.len() - 1])).is_some();
        return Some(Ok(Value::Bool(ok)));
    }

    if code.starts_with("~cache_has(") && code.ends_with(')') {
        let key = VM::tq(&code[11..code.len() - 1]);
        let ok = if let Some((_, expiry, _)) = vm.cache.get(&key) {
            expiry.map(|e| std::time::Instant::now() < e).unwrap_or(true)
        } else {
            false
        };
        vm.echo(&format!("{}", ok));
        return Some(Ok(Value::Bool(ok)));
    }

    if code == "~cache_clear()" {
        vm.cache.clear();
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~cache_size()" {
        let n = vm.cache.len();
        vm.echo(format!("{}", n));
        return Some(Ok(Value::Number(n as f64)));
    }

    if code == "~cache_list()" {
        let keys: Vec<Value> = vm.cache.keys().map(|k| Value::Text(k.clone())).collect();
        let list_val = Value::List(keys.clone());
        vm.echo(&format!("{}", list_val.display()));
        return Some(Ok(list_val));
    }

    if code.starts_with("~list_minmax(") && code.ends_with(')') {
        let name = code[13..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
        if nums.is_empty() { return Some(Err("Empty list".to_string())); }
        let mn = nums.iter().cloned().fold(f64::INFINITY, f64::min);
        let mx = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let r = Value::List(vec![Value::Number(mn), Value::Number(mx)]);
        vm.echo(r.display()); Some(Ok(r))
    } else if code.starts_with("~list_argmin(") && code.ends_with(')') {
        let name = code[13..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
        if nums.is_empty() { return Some(Err("Empty list".to_string())); }
        let idx = nums.iter().enumerate().fold(0, |i, (j, &v)| if v < nums[i] { j } else { i });
        vm.echo(&format!("{}", idx)); Some(Ok(Value::Number(idx as f64)))
    } else if code.starts_with("~list_argmax(") && code.ends_with(')') {
        let name = code[13..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
        if nums.is_empty() { return Some(Err("Empty list".to_string())); }
        let idx = nums.iter().enumerate().fold(0, |i, (j, &v)| if v > nums[i] { j } else { i });
        vm.echo(&format!("{}", idx)); Some(Ok(Value::Number(idx as f64)))
    } else if code.starts_with("~list_repeat(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() == 2 {
            let v = vm.eval_value(&p[0]).ok()?;
            let n = vm.eval_expr(&p[1]).ok()? as usize;
            let r = Value::List(vec![v; n]);
            vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_enumerate(") && code.ends_with(')') {
        let name = code[16..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let r: Vec<Value> = list.iter().enumerate().map(|(i, v)| Value::List(vec![Value::Number(i as f64), v.clone()])).collect();
        let r = Value::List(r); vm.echo(format!("{}", r.display())); Some(Ok(r))
    } else if code.starts_with("~list_intersperse(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[18..code.len()-1], ',');
        if p.len() == 2 {
            let name = p[0].trim(); let sep = vm.eval_value(&p[1]).ok()?;
            let mut list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let mut r = Vec::with_capacity(list.len() * 2 - 1);
            for (i, v) in list.into_iter().enumerate() { if i > 0 { r.push(sep.clone()); } r.push(v); }
            let r = Value::List(r); vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_partition(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[16..code.len()-1], ',');
        if p.len() == 2 {
            let name = p[0].trim(); let predicate_lambda = p[1].trim().to_string();
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let mut yes = Vec::new(); let mut no = Vec::new();
            for v in &list {
                let result = eval_pred(vm, &predicate_lambda, v);
                if result.map(|r| r.truthy()).unwrap_or(false) { yes.push(v.clone()); } else { no.push(v.clone()); }
            }
            let r = Value::List(vec![Value::List(yes), Value::List(no)]);
            vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_flatten(") && code.ends_with(')') {
        let name = code[14..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        fn flatten(items: &[Value]) -> Vec<Value> { let mut r = Vec::new(); for v in items { match v { Value::List(inner) => r.extend(flatten(inner)), other => r.push(other.clone()) } } r }
        let r = Value::List(flatten(&list)); vm.echo(format!("{}", r.display())); Some(Ok(r))
    } else if code.starts_with("~list_group_by(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ',');
        if p.len() == 2 {
            let name = p[0].trim(); let key_fn = p[1].trim().to_string();
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let mut groups: std::collections::HashMap<String, Vec<Value>> = std::collections::HashMap::new();
            let mut keys: Vec<String> = Vec::new();
            for v in &list {
                let key = eval_pred(vm, &key_fn, v).ok().map(|r| r.display()).unwrap_or_default();
                if !groups.contains_key(&key) { keys.push(key.clone()); }
                groups.entry(key).or_default().push(v.clone());
            }
            let r: Vec<Value> = keys.into_iter().filter_map(|k| groups.remove(&k).map(|v| Value::List(vec![Value::Text(k), Value::List(v)]))).collect();
            let r = Value::List(r); vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_window(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() == 2 {
            let name = p[0].trim(); let size = vm.eval_expr(&p[1]).ok()? as usize;
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let r: Vec<Value> = list.windows(size).map(|w| Value::List(w.to_vec())).collect();
            let r = Value::List(r); vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_pairs(") && code.ends_with(')') {
        let name = code[12..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let r: Vec<Value> = list.chunks(2).map(|c| if c.len() == 2 { Value::List(vec![c[0].clone(), c[1].clone()]) } else { Value::List(vec![c[0].clone()]) }).collect();
        let r = Value::List(r); vm.echo(format!("{}", r.display())); Some(Ok(r))
    } else if code.starts_with("~list_zip_with(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ','); if p.len() == 3 {
            let a = match vm.get_var(p[0].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let b = match vm.get_var(p[1].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let fn_name = p[2].trim().to_string();
            let r: Vec<Value> = a.iter().zip(b.iter()).map(|(x, y)| { vm.push_scope(); vm.set_var("_a", x.clone()); vm.set_var("_b", y.clone()); let r = crate::main::eval::evaluate(vm, &fn_name).unwrap_or(Value::None); vm.pop_scope(); r }).collect();
            let r = Value::List(r); vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_cartesian(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[16..code.len()-1], ','); if p.len() == 2 {
            let a = match vm.get_var(p[0].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let b = match vm.get_var(p[1].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let r: Vec<Value> = a.iter().flat_map(|x| b.iter().map(move |y| Value::List(vec![x.clone(), y.clone()]))).collect();
            let r = Value::List(r); vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_permutations(") && code.ends_with(')') {
        let name = code[19..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let n = list.len(); if n > 7 { return Some(Err("Too many permutations (max 7)".to_string())); }
        let mut result = Vec::new(); let mut indices: Vec<usize> = (0..n).collect();
        fn permute(items: &[Value], idx: &mut Vec<usize>, start: usize, result: &mut Vec<Value>) { if start >= idx.len() { result.push(Value::List(idx.iter().map(|&i| items[i].clone()).collect())); } else { for i in start..idx.len() { idx.swap(start, i); permute(items, idx, start+1, result); idx.swap(start, i); } } }
        permute(&list, &mut indices, 0, &mut result);
        let r = Value::List(result); vm.echo(format!("{}", r.display())); Some(Ok(r))
    } else if code.starts_with("~list_combinations(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[19..code.len()-1], ','); if p.len() == 2 {
            let name = p[0].trim(); let k = vm.eval_expr(&p[1]).ok()? as usize;
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let n = list.len(); if n > 20 { return Some(Err("Too many combinations".to_string())); }
            let mut result = Vec::new();
            fn combine(items: &[Value], k: usize, start: usize, curr: &mut Vec<Value>, result: &mut Vec<Value>) { if curr.len() == k { result.push(Value::List(curr.clone())); return; } for i in start..items.len() { curr.push(items[i].clone()); combine(items, k, i+1, curr, result); curr.pop(); } }
            combine(&list, k, 0, &mut Vec::new(), &mut result);
            let r = Value::List(result); vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_shuffle(") && code.ends_with(')') {
        let name = code[14..code.len()-1].trim();
        let mut list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        use rand::seq::SliceRandom; list.shuffle(&mut rand::thread_rng());
        vm.set_var(name, Value::List(list.clone())); let r = Value::List(list); vm.echo(format!("{}", r.display())); Some(Ok(r))
    } else if code.starts_with("~list_sample(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ','); if p.len() == 2 {
            let name = p[0].trim(); let k = vm.eval_expr(&p[1]).ok()? as usize;
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            if k > list.len() { return Some(Err("k > list size".to_string())); }
            let mut indices: Vec<usize> = (0..list.len()).collect();
            use rand::seq::SliceRandom; indices.shuffle(&mut rand::thread_rng());
            let sampled: Vec<Value> = indices.iter().take(k).map(|&i| list[i].clone()).collect();
            let r = Value::List(sampled); vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_fill(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ','); if p.len() == 2 {
            let v = vm.eval_value(&p[0]).ok()?; let n = vm.eval_expr(&p[1]).ok()? as usize;
            let r = Value::List(vec![v; n]); vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_range(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 2 {
            let a = vm.eval_expr(&p[0]).ok()? as i64; let b = vm.eval_expr(&p[1]).ok()? as i64;
            let step = if p.len() >= 3 { vm.eval_expr(&p[2]).ok()? as i64 } else { 1 };
            let mut items = Vec::new();
            if step > 0 { let mut i = a; while i <= b { items.push(Value::Number(i as f64)); i += step; } }
            else { let mut i = a; while i >= b { items.push(Value::Number(i as f64)); i += step; } }
            let r = Value::List(items); vm.echo(format!("{}", r.display())); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~list_any(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim(); let pred = p[1].trim().to_string();
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            for v in &list { let r = eval_pred(vm, &pred, v).map(|v| v.truthy()).unwrap_or(false); if r { vm.echo("true"); return Some(Ok(Value::Bool(true))); } }
            vm.echo("false"); Some(Ok(Value::Bool(false)))
        } else { None }
    } else if code.starts_with("~list_all(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim(); let pred = p[1].trim().to_string();
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            for v in &list { let r = eval_pred(vm, &pred, v).map(|v| v.truthy()).unwrap_or(false); if !r { vm.echo("false"); return Some(Ok(Value::Bool(false))); } }
            vm.echo("true"); Some(Ok(Value::Bool(true)))
        } else { None }
    } else if code.starts_with("~list_notany(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim(); let pred = p[1].trim().to_string();
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            for v in &list { let r = eval_pred(vm, &pred, v).map(|v| v.truthy()).unwrap_or(false); if r { vm.echo("false"); return Some(Ok(Value::Bool(false))); } }
            vm.echo("true"); Some(Ok(Value::Bool(true)))
        } else { None }
    } else if code.starts_with("~list_count_by(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim(); let pred = p[1].trim().to_string();
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let mut count = 0u64; for v in &list { let r = eval_pred(vm, &pred, v).map(|v| v.truthy()).unwrap_or(false); if r { count += 1; } }
            vm.echo(&format!("{}", count)); Some(Ok(Value::Number(count as f64)))
        } else { None }
    } else if code.starts_with("~list_append(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim().to_string(); let val = vm.eval_value(&p[1]).ok()?;
            let mut list = match vm.get_var(&name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            list.push(val); vm.set_var(&name, Value::List(list)); Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~list_prepend(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim().to_string(); let val = vm.eval_value(&p[1]).ok()?;
            let mut list = match vm.get_var(&name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            list.insert(0, val); vm.set_var(&name, Value::List(list)); Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~list_clear(") && code.ends_with(')') {
        let name = code[12..code.len()-1].trim().to_string();
        vm.set_var(&name, Value::List(Vec::new())); Some(Ok(Value::None))
    } else if code.starts_with("~list_is_empty(") && code.ends_with(')') {
        let name = code[15..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let r = list.is_empty(); vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~list_contains_val(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[18..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim(); let val = vm.eval_value(&p[1]).ok()?;
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let r = list.iter().any(|v| format!("{:?}", v) == format!("{:?}", val));
            vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
        } else { None }
    } else if code.starts_with("~list_index_of(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim(); let val = vm.eval_value(&p[1]).ok()?;
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let r = list.iter().position(|v| format!("{:?}", v) == format!("{:?}", val)).map(|i| i as f64).unwrap_or(-1.0);
            vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
        } else { None }
    } else if code.starts_with("~list_last_index(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim(); let val = vm.eval_value(&p[1]).ok()?;
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let r = list.iter().rposition(|v| format!("{:?}", v) == format!("{:?}", val)).map(|i| i as f64).unwrap_or(-1.0);
            vm.echo(format!("{}", Value::fmt_num(r))); Some(Ok(Value::Number(r)))
        } else { None }
    } else if code.starts_with("~list_remove_val(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim().to_string(); let val = vm.eval_value(&p[1]).ok()?;
            let mut list = match vm.get_var(&name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            list.retain(|v| format!("{:?}", v) != format!("{:?}", val));
            vm.set_var(&name, Value::List(list)); Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~list_dedup(") && code.ends_with(')') {
        let name = code[12..code.len()-1].trim().to_string();
        let list = match vm.get_var(&name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let mut seen = std::collections::HashSet::new(); let mut deduped = Vec::new();
        for v in list { let s = format!("{:?}", v); if seen.insert(s) { deduped.push(v); } }
        vm.set_var(&name, Value::List(deduped)); Some(Ok(Value::None))
    } else if code.starts_with("~list_take_while(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim(); let pred = p[1].trim().to_string();
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let mut r = Vec::new(); for v in &list { let ok = eval_pred(vm, &pred, v).map(|v| v.truthy()).unwrap_or(false); if ok { r.push(v.clone()); } else { break; } }
            let rv = Value::List(r); vm.echo(format!("{}", rv.display())); Some(Ok(rv))
        } else { None }
    } else if code.starts_with("~list_drop_while(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim(); let pred = p[1].trim().to_string();
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let mut r = Vec::new(); let mut dropping = true; for v in &list { let ok = eval_pred(vm, &pred, v).map(|v| v.truthy()).unwrap_or(false); if dropping && ok { continue; } dropping = false; r.push(v.clone()); }
            let rv = Value::List(r); vm.echo(format!("{}", rv.display())); Some(Ok(rv))
        } else { None }
    } else {
        None
    }
}

// Понимаем и лямбды и старые команды через _ чтобы прежние скрипты продолжали работать
fn eval_pred(vm: &mut VM, pred: &str, arg: &Value) -> Result<Value, String> {
    let t = pred.trim();
    if t.starts_with('|') || t.starts_with("lambda ") || t.starts_with("lambda(") || t.starts_with("fn(") || t.contains("=>") {
        let lam = vm.eval_value(t)?;
        if let Value::Lambda(params, body, _) = lam {
            return vm.call_lambda(params, body, vec![arg.clone()], 0);
        }
    }
    vm.push_scope();
    vm.set_var("_", arg.clone());
    let r = crate::main::eval::evaluate(vm, t);
    vm.pop_scope();
    r
}
