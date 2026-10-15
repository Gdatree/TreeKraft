use crate::main::vm::{VM, Value};

/// Модуль генераторов и итераторов: range, enumerate, zip, cycle, take, drop, step.
pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    fn num(vm: &mut VM, s: &str) -> Option<f64> {
        vm.eval_value(s).ok().and_then(|v| v.as_number())
    }
    fn list(vm: &mut VM, s: &str) -> Option<Vec<Value>> {
        match vm.eval_value(s) {
            Ok(Value::List(items)) => Some(items),
            _ => None,
        }
    }

    // ГЕНЕРАТОРЫ
    if code.starts_with("~iter_range(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() >= 2 {
            let a = num(vm, &p[0])?;
            let b = num(vm, &p[1])?;
            let step = if p.len() >= 3 { num(vm, &p[2])? } else { 1.0 };
            let mut out = Vec::new();
            if step > 0.0 {
                let mut x = a;
                while x <= b { out.push(Value::Number(x)); x += step; }
            } else if step < 0.0 {
                let mut x = a;
                while x >= b { out.push(Value::Number(x)); x += step; }
            }
            let lv = Value::List(out);
            Some(Ok(lv))
        } else { None }
    } else if code.starts_with("~iter_repeat(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() >= 2 {
            let val = vm.eval_value(&p[0]).ok()?;
            let n = num(vm, &p[1])? as usize;
            let out: Vec<Value> = (0..n).map(|_| val.clone()).collect();
            let lv = Value::List(out);
            Some(Ok(lv))
        } else { None }
    } else if code.starts_with("~iter_cycle(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() >= 2 {
            let src = list(vm, &p[0])?;
            let n = num(vm, &p[1])? as usize;
            if src.is_empty() { return Some(Ok(Value::List(vec![]))); }
            let out: Vec<Value> = (0..n).map(|i| src[i % src.len()].clone()).collect();
            let lv = Value::List(out);
            Some(Ok(lv))
        } else { None }
    }
    // ТРАНСФОРМАЦИИ
    else if code.starts_with("~iter_enumerate(") && code.ends_with(')') {
        let src = list(vm, &code[16..code.len() - 1])?;
        let out: Vec<Value> = src.into_iter()
            .enumerate()
            .map(|(i, v)| Value::List(vec![Value::Number(i as f64), v]))
            .collect();
        let lv = Value::List(out);
        Some(Ok(lv))
    } else if code.starts_with("~iter_zip(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() >= 2 {
            let a = list(vm, &p[0])?;
            let b = list(vm, &p[1])?;
            let n = a.len().min(b.len());
            let out: Vec<Value> = (0..n)
                .map(|i| Value::List(vec![a[i].clone(), b[i].clone()]))
                .collect();
            let lv = Value::List(out);
            Some(Ok(lv))
        } else { None }
    } else if code.starts_with("~iter_take(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() >= 2 {
            let src = list(vm, &p[0])?;
            let n = num(vm, &p[1])? as usize;
            let out = src.into_iter().take(n).collect::<Vec<_>>();
            let lv = Value::List(out);
            Some(Ok(lv))
        } else { None }
    } else if code.starts_with("~iter_drop(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() >= 2 {
            let src = list(vm, &p[0])?;
            let n = num(vm, &p[1])? as usize;
            let out = src.into_iter().skip(n).collect::<Vec<_>>();
            let lv = Value::List(out);
            Some(Ok(lv))
        } else { None }
    } else if code.starts_with("~iter_step(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() >= 2 {
            let src = list(vm, &p[0])?;
            let step = (num(vm, &p[1])?).max(1.0) as usize;
            let out: Vec<Value> = src.into_iter().step_by(step).collect();
            let lv = Value::List(out);
            Some(Ok(lv))
        } else { None }
    } else if code.starts_with("~iter_chunk(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() >= 2 {
            let src = list(vm, &p[0])?;
            let size = (num(vm, &p[1])?).max(1.0) as usize;
            let out: Vec<Value> = src.chunks(size).map(|c| Value::List(c.to_vec())).collect();
            let lv = Value::List(out);
            Some(Ok(lv))
        } else { None }
    } else if code.starts_with("~iter_windows(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len() - 1], ',');
        if p.len() >= 2 {
            let src = list(vm, &p[0])?;
            let w = (num(vm, &p[1])?).max(2.0) as usize;
            let out: Vec<Value> = src.windows(w).map(|c| Value::List(c.to_vec())).collect();
            let lv = Value::List(out);
            Some(Ok(lv))
        } else { None }
    } else if code.starts_with("~iter_rev(") && code.ends_with(')') {
        let mut src = list(vm, &code[12..code.len() - 1])?;
        src.reverse();
        let lv = Value::List(src);
        Some(Ok(lv))
    } else if code.starts_with("~iter_pairwise(") && code.ends_with(')') {
        let src = list(vm, &code[15..code.len() - 1])?;
        let out: Vec<Value> = src.windows(2).map(|c| Value::List(c.to_vec())).collect();
        let lv = Value::List(out);
        Some(Ok(lv))
    } else {
        None
    }
}
