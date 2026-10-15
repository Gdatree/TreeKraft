use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~bin(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[5..code.len() - 1]).ok()? as u64;
        let r = format!("{:b}", n); vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~oct(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[5..code.len() - 1]).ok()? as u64;
        let r = format!("{:o}", n); vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~hexnum(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[8..code.len()-1]).ok()? as u64;
        let r = format!("{:X}", n);
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~dec(") && code.ends_with(')') {
        let s = vm.eval_value(&code[5..code.len() - 1]).ok()?.as_text()?;
        let r = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) { i64::from_str_radix(hex, 16).ok().map(|n| n as f64) }
            else if let Some(bin) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) { i64::from_str_radix(bin, 2).ok().map(|n| n as f64) }
            else { s.parse::<f64>().ok() };
        match r { Some(v) => { vm.echo(Value::fmt_num(v)); Some(Ok(Value::Number(v))) } None => Some(Err("Bad number".to_string())) }
    } else if code.starts_with("~int(") && code.ends_with(')') {
        let v = vm.eval_value(&code[5..code.len() - 1]).ok()?;
        let n = v.as_number().map(|x| x.trunc() as i64 as f64).unwrap_or(0.0);
        vm.echo(Value::fmt_num(n)); Some(Ok(Value::Number(n)))
    } else if code.starts_with("~float(") && code.ends_with(')') {
        let v = vm.eval_value(&code[7..code.len() - 1]).ok()?;
        let n = v.as_number().unwrap_or(0.0);
        vm.echo(Value::fmt_num(n)); Some(Ok(Value::Number(n)))
    } else if code.starts_with("~bool(") && code.ends_with(')') {
        let v = vm.eval_value(&code[6..code.len() - 1]).ok()?;
        let r = v.truthy(); vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~str(") && code.ends_with(')') {
        let v = vm.eval_value(&code[5..code.len() - 1]).ok()?;
        let r = v.display(); vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~ord(") && code.ends_with(')') {
        let s = vm.eval_value(&code[5..code.len() - 1]).ok()?.as_text()?;
        let r = s.chars().next().map(|c| c as u32 as f64).unwrap_or(-1.0);
        vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~chr(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[5..code.len() - 1]).ok()? as u32;
        let r = char::from_u32(n).map(|c| c.to_string()).unwrap_or_default();
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~round_to(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let n = vm.eval_expr(&p[0]).ok()?;
            let d = vm.eval_expr(&p[1]).ok()? as u32;
            let f = 10f64.powi(d as i32);
            let r = (n * f).round() / f;
            vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
        } else { None }
    } else if code.starts_with("~clamp(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() == 3 {
            let (x, lo, hi) = (vm.eval_expr(&p[0]).ok()?, vm.eval_expr(&p[1]).ok()?, vm.eval_expr(&p[2]).ok()?);
            let r = x.max(lo).min(hi);
            vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
        } else { None }
    } else if code.starts_with("~lerp(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 3 {
            let (a, b, t) = (vm.eval_expr(&p[0]).ok()?, vm.eval_expr(&p[1]).ok()?, vm.eval_expr(&p[2]).ok()?);
            let r = a + (b - a) * t.max(0.0).min(1.0);
            vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
        } else { None }
    } else if code.starts_with("~range(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() >= 2 {
            let a = vm.eval_expr(&p[0]).ok()? as i64;
            let b = vm.eval_expr(&p[1]).ok()? as i64;
            let step = if p.len() >= 3 { vm.eval_expr(&p[2]).ok()? as i64 } else { 1 };
            if step == 0 {
                return Some(Err("~range: step cannot be 0".to_string()));
            }
            let mut list = Vec::new();
            if step > 0 { let mut i = a; while i <= b { list.push(Value::Number(i as f64)); i += step; } }
            else { let mut i = a; while i >= b { list.push(Value::Number(i as f64)); i += step; } }
            let r = Value::List(list); vm.echo(r.display()); Some(Ok(r))
        } else { None }
    } else if code.starts_with("~roman(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[7..code.len() - 1]).ok()? as u64;
        let vals = [(1000,"M"),(900,"CM"),(500,"D"),(400,"CD"),(100,"C"),(90,"XC"),(50,"L"),(40,"XL"),(10,"X"),(9,"IX"),(5,"V"),(4,"IV"),(1,"I")];
        let mut r = String::new(); let mut x = n;
        for &(v, s) in &vals { while x >= v { r.push_str(s); x -= v; } }
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~ascii(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len() - 1]).ok()?.as_text()?;
        let r: String = s.chars().map(|c| format!("{}", c as u8)).collect::<Vec<_>>().join(" ");
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~from_ascii(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len() - 1]).ok()?.as_text()?;
        let r: String = s.split_whitespace().filter_map(|n| n.parse::<u8>().ok().map(|b| b as char)).collect();
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~roman_parse(") && code.ends_with(')') {
        let s = vm.eval_value(&code[13..code.len() - 1]).ok()?.as_text()?;
        let vals = [('I',1),('V',5),('X',10),('L',50),('C',100),('D',500),('M',1000)];
        let mut total = 0i64; let mut prev = 0i64;
        for ch in s.chars().rev() {
            let v = vals.iter().find(|&&(c, _)| c == ch).map(|&(_, n)| n).unwrap_or(0);
            if v < prev { total -= v; } else { total += v; }
            prev = v;
        }
        let r = total as f64; vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~byte_size(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[11..code.len() - 1]).ok()? as f64;
        let units = ["B","KB","MB","GB","TB","PB"];
        let mut v = n; let mut i = 0;
        while v >= 1024.0 && i < units.len()-1 { v /= 1024.0; i += 1; }
        let r = if i == 0 { format!("{} B", n as u64) } else { format!("{:.1} {}", v, units[i]) };
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else {
        None
    }
}
