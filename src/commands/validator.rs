use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    fn val(vm: &mut VM, s: &str) -> Option<String> {
        vm.eval_value(s).ok().and_then(|v| v.as_text())
    }
    fn num(vm: &mut VM, s: &str) -> Option<f64> {
        vm.eval_value(s).ok().and_then(|v| v.as_number())
    }

    if code.starts_with("~valid_email(") && code.ends_with(')') {
        let s = val(vm, &code[13..code.len()-1])?;
        let r = s.contains('@') && s.contains('.') && s.len() >= 5;
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_url(") && code.ends_with(')') {
        let s = val(vm, &code[11..code.len()-1])?;
        let r = s.starts_with("http://") || s.starts_with("https://");
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_ip(") && code.ends_with(')') {
        let s = val(vm, &code[10..code.len()-1])?;
        let parts: Vec<&str> = s.split('.').collect();
        let r = parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok());
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_port(") && code.ends_with(')') {
        let n = num(vm, &code[12..code.len()-1])? as u32;
        let r = n >= 1 && n <= 65535;
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_number(") && code.ends_with(')') {
        let s = val(vm, &code[14..code.len()-1])?;
        let r = s.parse::<f64>().is_ok();
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_int(") && code.ends_with(')') {
        let s = val(vm, &code[11..code.len()-1])?;
        let r = s.parse::<i64>().is_ok();
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_hex(") && code.ends_with(')') {
        let s = val(vm, &code[11..code.len()-1])?;
        let r = s.len() >= 2 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit());
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_uuid(") && code.ends_with(')') {
        let s = val(vm, &code[12..code.len()-1])?;
        let r = s.len() == 36 && s.chars().filter(|&c| c == '-').count() == 4 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_date(") && code.ends_with(')') {
        let s = val(vm, &code[12..code.len()-1])?;
        let parts: Vec<&str> = s.split('-').collect();
        let r = parts.len() == 3 && parts[0].len() == 4 && parts[0].parse::<u32>().is_ok()
            && parts[1].len() == 2 && parts[1].parse::<u32>().map(|m| m >= 1 && m <= 12).unwrap_or(false)
            && parts[2].len() == 2 && parts[2].parse::<u32>().map(|d| d >= 1 && d <= 31).unwrap_or(false);
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_empty(") && code.ends_with(')') {
        let s = val(vm, &code[13..code.len()-1])?;
        let r = !s.is_empty();
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_len(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 3 {
            let s = val(vm, &p[0])?; let lo = num(vm, &p[1])? as usize; let hi = num(vm, &p[2])? as usize;
            let r = s.len() >= lo && s.len() <= hi;
            vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
        } else { None }
    } else if code.starts_with("~valid_range(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() >= 3 {
            let n = num(vm, &p[0])?; let lo = num(vm, &p[1])?; let hi = num(vm, &p[2])?;
            let r = n >= lo && n <= hi;
            vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
        } else { None }
    } else if code.starts_with("~valid_alpha(") && code.ends_with(')') {
        let s = val(vm, &code[13..code.len()-1])?;
        let r = !s.is_empty() && s.chars().all(|c| c.is_alphabetic());
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_alnum(") && code.ends_with(')') {
        let s = val(vm, &code[13..code.len()-1])?;
        let r = !s.is_empty() && s.chars().all(|c| c.is_alphanumeric());
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~valid_digit(") && code.ends_with(')') {
        let s = val(vm, &code[13..code.len()-1])?;
        let r = !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~match_glob(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() >= 2 {
            let pat = val(vm, &p[0])?; let text = val(vm, &p[1])?;
            let r = match_glob(&pat, &text);
            vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
        } else { None }
    } else {
        None
    }
}

/// Glob-совпадение: `*` любое число символов, `?` ровно один, `[...]` класс.
/// Раньше жило в utils.rs (модуль удалён как мёртвый — это был его единственный
/// пользователь). Семантика зафиксирована тестами: *.txt/he?lo.txt.
fn match_glob(pat: &str, text: &str) -> bool {
    fn go(px: &[char], tx: &[char]) -> bool {
        if px.is_empty() { return tx.is_empty(); }
        match px[0] {
            '*' => go(&px[1..], tx) || (!tx.is_empty() && go(px, &tx[1..])),
            '?' => !tx.is_empty() && go(&px[1..], &tx[1..]),
            '[' => {
                let end = match px.iter().position(|&c| c == ']') { Some(e) => e, None => return false };
                if tx.is_empty() { return false; }
                let cls: Vec<char> = px[1..end].iter().cloned().collect();
                let (neg, items) = match cls.first() {
                    Some('^') => (true, &cls[1..]),
                    _ => (false, &cls[..]),
                };
                // Элементы: одиночные символы и диапазоны a-z.
                let mut hit = false;
                let mut j = 0;
                while j < items.len() {
                    if j + 2 < items.len() && items[j + 1] == '-' {
                        if items[j] <= tx[0] && tx[0] <= items[j + 2] { hit = true; break; }
                        j += 3;
                    } else {
                        if items[j] == tx[0] { hit = true; break; }
                        j += 1;
                    }
                }
                if neg { hit = !hit; }
                hit && go(&px[end + 1..], &tx[1..])
            }
            c => !tx.is_empty() && tx[0] == c && go(&px[1..], &tx[1..]),
        }
    }
    let p: Vec<char> = pat.chars().collect();
    let t: Vec<char> = text.chars().collect();
    go(&p, &t)
}
