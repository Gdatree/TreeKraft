use crate::main::vm::{VM, Value};

/// Модуль статистики: описательные метрики, распределения, корреляции.
pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    fn list(vm: &mut VM, s: &str) -> Option<Vec<f64>> {
        match vm.eval_value(s) {
            Ok(Value::List(items)) => {
                let mut out = Vec::new();
                for it in items {
                    match it.as_number() {
                        Some(n) => out.push(n),
                        None => return None,
                    }
                }
                Some(out)
            }
            Ok(Value::Text(t)) => {
                let mut out = Vec::new();
                for p in t.split(|c: char| c.is_whitespace() || c == ',') {
                    if let Ok(n) = p.trim().parse::<f64>() { out.push(n); }
                }
                if out.is_empty() { None } else { Some(out) }
            }
            _ => None,
        }
    }
    fn nums(vm: &mut VM, s: &str) -> Option<Vec<f64>> {
        let p = VM::split_top_level(s, ',');
        let mut out = Vec::new();
        for x in p {
            match vm.eval_value(x.trim()) {
                Ok(v) => match v.as_number() {
                    Some(n) => out.push(n),
                    None => return None,
                },
                Err(_) => return None,
            }
        }
        if out.is_empty() { None } else { Some(out) }
    }

    // БАЗОВЫЕ МЕТРИКИ
    if code.starts_with("~stats_mean(") && code.ends_with(')') {
        let v = nums(vm, &code[12..code.len() - 1])?;
        let m: f64 = v.iter().sum::<f64>() / v.len() as f64;
        vm.echo(m); Some(Ok(Value::Number(m)))
    } else if code.starts_with("~stats_sum(") && code.ends_with(')') {
        let v = nums(vm, &code[11..code.len() - 1])?;
        let s: f64 = v.iter().sum();
        vm.echo(s); Some(Ok(Value::Number(s)))
    } else if code.starts_with("~stats_min(") && code.ends_with(')') {
        let v = nums(vm, &code[11..code.len() - 1])?;
        let m = v.iter().cloned().fold(f64::INFINITY, f64::min);
        vm.echo(m); Some(Ok(Value::Number(m)))
    } else if code.starts_with("~stats_max(") && code.ends_with(')') {
        let v = nums(vm, &code[11..code.len() - 1])?;
        let m = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        vm.echo(m); Some(Ok(Value::Number(m)))
    } else if code.starts_with("~stats_range(") && code.ends_with(')') {
        let v = nums(vm, &code[13..code.len() - 1])?;
        let lo = v.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let r = hi - lo;
        vm.echo(r); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~stats_var(") && code.ends_with(')') {
        let v = nums(vm, &code[11..code.len() - 1])?;
        let m: f64 = v.iter().sum::<f64>() / v.len() as f64;
        let var: f64 = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64;
        vm.echo(&format!("{}", var)); Some(Ok(Value::Number(var)))
    } else if code.starts_with("~stats_std(") && code.ends_with(')') {
        let v = nums(vm, &code[11..code.len() - 1])?;
        let m: f64 = v.iter().sum::<f64>() / v.len() as f64;
        let var: f64 = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64;
        let sd = var.sqrt();
        vm.echo(&format!("{}", sd)); Some(Ok(Value::Number(sd)))
    } else if code.starts_with("~stats_median(") && code.ends_with(')') {
        let mut v = nums(vm, &code[14..code.len() - 1])?;
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = v.len();
        let med = if n % 2 == 1 {
            v[n / 2]
        } else {
            (v[n / 2 - 1] + v[n / 2]) / 2.0
        };
        vm.echo(&format!("{}", med)); Some(Ok(Value::Number(med)))
    } else if code.starts_with("~stats_mode(") && code.ends_with(')') {
        let v = nums(vm, &code[12..code.len() - 1])?;
        let mut counts: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
        for x in &v {
            *counts.entry(*x as i64).or_insert(0) += 1;
        }
        let mode = counts.iter().max_by_key(|(_, c)| *c).map(|(k, _)| *k as f64).unwrap_or(0.0);
        vm.echo(&format!("{}", mode)); Some(Ok(Value::Number(mode)))
    } else if code.starts_with("~stats_quartiles(") && code.ends_with(')') {
        let mut v = nums(vm, &code[17..code.len() - 1])?;
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = v.len();
        let q = |p: f64| -> f64 {
            let idx = ((n as f64 - 1.0) * p).round() as usize;
            v[idx.min(n - 1)]
        };
        let s = format!("Q1={} Q2={} Q3={}", q(0.25), q(0.5), q(0.75));
        vm.echo(&s); Some(Ok(Value::Text(s)))
    } else if code.starts_with("~stats_iqr(") && code.ends_with(')') {
        let mut v = nums(vm, &code[12..code.len() - 1])?;
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = v.len();
        let q = |p: f64| -> f64 {
            let idx = ((n as f64 - 1.0) * p).round() as usize;
            v[idx.min(n - 1)]
        };
        let iqr = q(0.75) - q(0.25);
        vm.echo(&format!("{}", iqr)); Some(Ok(Value::Number(iqr)))
    }
    // РАСПРЕДЕЛЕНИЯ И СВЯЗИ
    else if code.starts_with("~stats_percentile(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[18..code.len() - 1], ',');
        if p.len() >= 2 {
            let mut v = list(vm, &p[0])?;
            let pct = vm.eval_value(&p[1]).ok().and_then(|x| x.as_number())?;
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let n = v.len();
            let idx = ((n as f64 - 1.0) * (pct / 100.0).clamp(0.0, 1.0)).round() as usize;
            let r = v[idx.min(n - 1)];
            vm.echo(r); Some(Ok(Value::Number(r)))
        } else { None }
    } else if code.starts_with("~stats_corr(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() >= 2 {
            let a = list(vm, &p[0])?;
            let b = list(vm, &p[1])?;
            if a.len() != b.len() || a.is_empty() { return Some(Err("stats_corr: mismatched lengths".into())); }
            let n = a.len() as f64;
            let ma: f64 = a.iter().sum::<f64>() / n;
            let mb: f64 = b.iter().sum::<f64>() / n;
            let mut num = 0.0; let mut da = 0.0; let mut db = 0.0;
            for i in 0..a.len() {
                let xa = a[i] - ma; let xb = b[i] - mb;
                num += xa * xb; da += xa * xa; db += xb * xb;
            }
            let r = if da > 0.0 && db > 0.0 { num / (da * db).sqrt() } else { 0.0 };
            vm.echo(r); Some(Ok(Value::Number(r)))
        } else { None }
    } else if code.starts_with("~stats_cov(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() >= 2 {
            let a = list(vm, &p[0])?;
            let b = list(vm, &p[1])?;
            if a.len() != b.len() || a.is_empty() { return Some(Err("stats_cov: mismatched lengths".into())); }
            let n = a.len() as f64;
            let ma: f64 = a.iter().sum::<f64>() / n;
            let mb: f64 = b.iter().sum::<f64>() / n;
            let cov: f64 = a.iter().zip(b.iter()).map(|(x, y)| (x - ma) * (y - mb)).sum::<f64>() / n;
            vm.echo(&format!("{}", cov)); Some(Ok(Value::Number(cov)))
        } else { None }
    } else if code.starts_with("~stats_hist(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() >= 2 {
            let v = list(vm, &p[0])?;
            let bins = vm.eval_value(&p[1]).ok().and_then(|x| x.as_number())? as usize;
            if v.is_empty() || bins == 0 { return Some(Ok(Value::List(vec![]))); }
            let lo = v.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let w = (hi - lo) / bins as f64;
            let mut counts = vec![0usize; bins];
            for x in &v {
                let mut bi = ((x - lo) / w).floor() as usize;
                if bi >= bins { bi = bins - 1; }
                counts[bi] += 1;
            }
            let list: Vec<Value> = counts.into_iter().map(|c| Value::Number(c as f64)).collect();
            let lv = Value::List(list);
            vm.echo(lv.display()); Some(Ok(lv))
        } else { None }
    } else if code.starts_with("~stats_zscore(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len() - 1], ',');
        if p.len() >= 2 {
            let v = list(vm, &p[0])?;
            let m: f64 = v.iter().sum::<f64>() / v.len() as f64;
            let var: f64 = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64;
            let sd = var.sqrt().max(1e-12);
            let x = vm.eval_value(&p[1]).ok().and_then(|y| y.as_number())?;
            let z = (x - m) / sd;
            vm.echo(z); Some(Ok(Value::Number(z)))
        } else { None }
    }
    // НОРМАЛИЗАЦИЯ
    else if code.starts_with("~stats_normalize(") && code.ends_with(')') {
        let v = list(vm, &code[17..code.len() - 1])?;
        if v.is_empty() { return Some(Ok(Value::List(vec![]))); }
        let lo = v.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (hi - lo).max(1e-12);
        let norm: Vec<Value> = v.iter().map(|x| Value::Number((x - lo) / range)).collect();
        let lv = Value::List(norm);
        vm.echo(&format!("{}", lv.display())); Some(Ok(lv))
    } else if code.starts_with("~stats_describe(") && code.ends_with(')') {
        let v = nums(vm, &code[16..code.len() - 1])?;
        let n = v.len() as f64;
        let m: f64 = v.iter().sum::<f64>() / n;
        let var: f64 = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n;
        let sd = var.sqrt();
        let lo = v.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let s = format!("n={} mean={} std={} min={} max={}", v.len(), m, sd, lo, hi);
        vm.echo(&s); Some(Ok(Value::Text(s)))
    } else {
        None
    }
}
