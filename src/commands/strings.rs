use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! st1 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let s = vm.eval_value(&code[$p.len()..code.len()-1]).ok()?.as_text()?; let r = ($f)(&s); vm.echo(&r); return Some(Ok(Value::Text(r))); } }; }
    macro_rules! st1n { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let s = vm.eval_value(&code[$p.len()..code.len()-1]).ok()?.as_text()?; let r = ($f)(&s) as f64; vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } }; }
    macro_rules! st1b { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let s = vm.eval_value(&code[$p.len()..code.len()-1]).ok()?.as_text()?; let r = ($f)(&s); vm.echo(r); return Some(Ok(Value::Bool(r))); } }; }
    macro_rules! st1l { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let s = vm.eval_value(&code[$p.len()..code.len()-1]).ok()?.as_text()?; let list: Vec<Value> = ($f)(&s).into_iter().map(Value::Text).collect(); let r = Value::List(list); vm.echo(r.display()); return Some(Ok(r)); } }; }
    macro_rules! st2 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_value(&p[0]).ok()?.as_text()?; let b = vm.eval_value(&p[1]).ok()?.as_text()?; let r = ($f)(&a, &b); vm.echo(&r); return Some(Ok(Value::Text(r))); } } }; }
    macro_rules! st2b { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_value(&p[0]).ok()?.as_text()?; let b = vm.eval_value(&p[1]).ok()?.as_text()?; let r = ($f)(&a, &b); vm.echo(r); return Some(Ok(Value::Bool(r))); } } }; }
    macro_rules! st2n { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_value(&p[0]).ok()?.as_text()?; let n = vm.eval_expr(&p[1]).ok()? as usize; let r = ($f)(&a, n); vm.echo(&r); return Some(Ok(Value::Text(r))); } } }; }
    if code.starts_with("~len(") && code.ends_with(')') {
        let val = match vm.eval_value(&code[5..code.len() - 1]) {
            Ok(v) => v,
            Err(e) => return Some(Err(e)),
        };
        let n = match &val {
            Value::Text(s) => s.chars().count() as f64,
            Value::List(l) => l.len() as f64,
            Value::Dict(d) => d.len() as f64,
            _ => return Some(Err("~len expects string, list, or dict".to_string())),
        };
        vm.echo(Value::fmt_num(n));
        return Some(Ok(Value::Number(n)));
    }

    if code.starts_with("~upper(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len() - 1]).ok()?.as_text()?;
        let r = s.to_uppercase();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~lower(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len() - 1]).ok()?.as_text()?;
        let r = s.to_lowercase();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~trim(") && code.ends_with(')') {
        let s = vm.eval_value(&code[6..code.len() - 1]).ok()?.as_text()?;
        let r = s.trim().to_string();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~rep(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            // Ограничиваем число повторов сверху, так как огромный повтор вешает интерпретатор
            let n = (vm.eval_expr(&p[1]).ok()? as usize).min(100_000);
            let r = s.repeat(n);
            vm.echo(&r);
            return Some(Ok(Value::Text(r)));
        }
    }

    if code.starts_with("~split(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let d = vm.eval_value(&p[1]).ok()?.as_text()?;
            let parts: Vec<Value> = s.split(&d).map(|x| Value::Text(x.to_string())).collect();
            let list = Value::List(parts);
            vm.echo(list.display());
            return Some(Ok(list));
        }
    }

    if code.starts_with("~concat(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let a = vm.eval_value(&p[0]).ok()?.as_text()?;
            let b = vm.eval_value(&p[1]).ok()?.as_text()?;
            let r = format!("{}{}", a, b);
            vm.echo(&r);
            return Some(Ok(Value::Text(r)));
        }
    }

    if code.starts_with("~sub(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() == 3 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let start = vm.eval_expr(&p[1]).ok()? as usize;
            let len = vm.eval_expr(&p[2]).ok()? as usize;
            let r: String = s.chars().skip(start).take(len).collect();
            vm.echo(&r);
            return Some(Ok(Value::Text(r)));
        }
    }

    if code.starts_with("~find(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let sub = vm.eval_value(&p[1]).ok()?.as_text()?;
            let pos = s.find(&sub).map(|i| i as f64).unwrap_or(-1.0);
            vm.echo(Value::fmt_num(pos));
            return Some(Ok(Value::Number(pos)));
        }
    }

    if code.starts_with("~replace(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 3 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let old = vm.eval_value(&p[1]).ok()?.as_text()?;
            let new = vm.eval_value(&p[2]).ok()?.as_text()?;
            let r = s.replace(&old, &new);
            vm.echo(&r);
            return Some(Ok(Value::Text(r)));
        }
    }

    if code.starts_with("~startswith(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let prefix = vm.eval_value(&p[1]).ok()?.as_text()?;
            let r = s.starts_with(&prefix);
            vm.echo(r);
            return Some(Ok(Value::Bool(r)));
        }
    }

    if code.starts_with("~endswith(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let suffix = vm.eval_value(&p[1]).ok()?.as_text()?;
            let r = s.ends_with(&suffix);
            vm.echo(r);
            return Some(Ok(Value::Bool(r)));
        }
    }

    if code.starts_with("~isspace(") && code.ends_with(')') {
        let s = vm.eval_value(&code[9..code.len() - 1]).ok()?.as_text()?;
        let r = s.chars().all(|c| c.is_whitespace());
        vm.echo(r);
        return Some(Ok(Value::Bool(r)));
    }

    if code.starts_with("~swapcase(") && code.ends_with(')') {
        let s = vm.eval_value(&code[10..code.len() - 1]).ok()?.as_text()?;
        let r: String = s.chars().map(|c| {
            if c.is_uppercase() {
                c.to_lowercase().next().unwrap_or(c)
            } else {
                c.to_uppercase().next().unwrap_or(c)
            }
        }).collect();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~isnum(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len() - 1]).ok()?.as_text()?;
        let r = s.parse::<f64>().is_ok();
        vm.echo(r);
        return Some(Ok(Value::Bool(r)));
    }

    if code.starts_with("~isalpha(") && code.ends_with(')') {
        let s = vm.eval_value(&code[9..code.len() - 1]).ok()?.as_text()?;
        let r = !s.is_empty() && s.chars().all(|c| c.is_alphabetic());
        vm.echo(r);
        return Some(Ok(Value::Bool(r)));
    }

    if code.starts_with("~isdigit(") && code.ends_with(')') {
        let s = vm.eval_value(&code[9..code.len() - 1]).ok()?.as_text()?;
        let r = !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
        vm.echo(r);
        return Some(Ok(Value::Bool(r)));
    }

    if code.starts_with("~isalnum(") && code.ends_with(')') {
        let s = vm.eval_value(&code[9..code.len() - 1]).ok()?.as_text()?;
        let r = !s.is_empty() && s.chars().all(|c| c.is_alphanumeric());
        vm.echo(r);
        return Some(Ok(Value::Bool(r)));
    }

    if code.starts_with("~lstrip(") && code.ends_with(')') {
        let s = vm.eval_value(&code[8..code.len() - 1]).ok()?.as_text()?;
        let r = s.trim_start().to_string();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~rstrip(") && code.ends_with(')') {
        let s = vm.eval_value(&code[8..code.len() - 1]).ok()?.as_text()?;
        let r = s.trim_end().to_string();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~count(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let sub = vm.eval_value(&p[1]).ok()?.as_text()?;
            let n = s.matches(&sub).count() as f64;
        vm.echo(Value::fmt_num(n));
        return Some(Ok(Value::Number(n)));
        }
    }

    if code.starts_with("~contains(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let sub = vm.eval_value(&p[1]).ok()?.as_text()?;
            let r = s.contains(&sub);
            vm.echo(r);
            return Some(Ok(Value::Bool(r)));
        }
    }

    if code.starts_with("~empty(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len() - 1]).ok()?.as_text()?;
        let r = s.is_empty();
        vm.echo(r);
        return Some(Ok(Value::Bool(r)));
    }

    if code.starts_with("~isupper(") && code.ends_with(')') {
        let s = vm.eval_value(&code[9..code.len() - 1]).ok()?.as_text()?;
        let r = !s.is_empty() && s.chars().all(|c| c.is_uppercase());
        vm.echo(r);
        return Some(Ok(Value::Bool(r)));
    }

    if code.starts_with("~islower(") && code.ends_with(')') {
        let s = vm.eval_value(&code[9..code.len() - 1]).ok()?.as_text()?;
        let r = !s.is_empty() && s.chars().all(|c| c.is_lowercase());
        vm.echo(r);
        return Some(Ok(Value::Bool(r)));
    }

    if code.starts_with("~capitalize(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len() - 1]).ok()?.as_text()?;
        let mut ch = s.chars();
        let r = match ch.next() {
            None => s,
            Some(f) => format!("{}{}", f.to_uppercase(), ch.as_str()),
        };
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~title(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len() - 1]).ok()?.as_text()?;
        let r = s.split_whitespace().map(|w| {
            let mut ch = w.chars();
            match ch.next() {
                None => w.to_string(),
                Some(f) => format!("{}{}", f.to_uppercase(), ch.as_str()),
            }
        }).collect::<Vec<_>>().join(" ");
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~reverse(") && code.ends_with(')') {
        let s = vm.eval_value(&code[9..code.len() - 1]).ok()?.as_text()?;
        let r: String = s.chars().rev().collect();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~center(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() >= 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let w = vm.eval_expr(&p[1]).ok()? as usize;
            let c = if p.len() > 2 { vm.eval_value(&p[2]).ok()?.as_text()?.chars().next().unwrap_or(' ') } else { ' ' };
            let pad = w.saturating_sub(s.len());
            let left = pad / 2;
            let right = pad - left;
            let r = format!("{}{}{}", c.to_string().repeat(left), s, c.to_string().repeat(right));
            vm.echo(&r);
            return Some(Ok(Value::Text(r)));
        }
    }

    if code.starts_with("~ljust(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() >= 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let w = vm.eval_expr(&p[1]).ok()? as usize;
            let c = if p.len() > 2 { vm.eval_value(&p[2]).ok()?.as_text()?.chars().next().unwrap_or(' ') } else { ' ' };
            let r = format!("{}{}", s, c.to_string().repeat(w.saturating_sub(s.len())));
            vm.echo(&r);
            return Some(Ok(Value::Text(r)));
        }
    }

    if code.starts_with("~rjust(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() >= 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let w = vm.eval_expr(&p[1]).ok()? as usize;
            let c = if p.len() > 2 { vm.eval_value(&p[2]).ok()?.as_text()?.chars().next().unwrap_or(' ') } else { ' ' };
            let r = format!("{}{}", c.to_string().repeat(w.saturating_sub(s.len())), s);
            vm.echo(&r);
            return Some(Ok(Value::Text(r)));
        }
    }

    if code.starts_with("~pad(") && code.ends_with(')') {
        return resolve(vm, &format!("~ljust{}", &code[4..]));
    }

    if code.starts_with("~chunk(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let n = (vm.eval_expr(&p[1]).ok()? as usize).max(1);
            let chunks: Vec<Value> = s.chars().collect::<Vec<_>>().chunks(n).map(|c| Value::Text(c.iter().collect())).collect();
            let r = Value::List(chunks);
            vm.echo(r.display());
            return Some(Ok(r));
        }
    }

    if code.starts_with("~wordwrap(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let w = (vm.eval_expr(&p[1]).ok()? as usize).max(1);
            let mut r = String::new();
            let mut line = String::new();
            for word in s.split_whitespace() {
                if line.len() + word.len() + (if line.is_empty() { 0 } else { 1 }) > w {
                    r.push_str(&line); r.push('\n'); line.clear();
                }
                if !line.is_empty() { line.push(' '); }
                line.push_str(word);
            }
            r.push_str(&line);
            vm.echo(&r);
            return Some(Ok(Value::Text(r)));
        }
    }

    if code.starts_with("~charat(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let i = vm.eval_expr(&p[1]).ok()? as usize;
            let c = s.chars().nth(i).map(|c| c.to_string()).unwrap_or_default();
            vm.echo(&c);
            return Some(Ok(Value::Text(c)));
        }
    }

    if code.starts_with("~codeat(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let i = vm.eval_expr(&p[1]).ok()? as usize;
            let n = s.chars().nth(i).map(|c| c as u32 as f64).unwrap_or(-1.0);
        vm.echo(Value::fmt_num(n));
        return Some(Ok(Value::Number(n)));
        }
    }

    if code.starts_with("~fromcode(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[10..code.len() - 1]).ok()? as u32;
        let c = char::from_u32(n).map(|c| c.to_string()).unwrap_or_default();
        vm.echo(&c);
        return Some(Ok(Value::Text(c)));
    }

    if code.starts_with("~sortchars(") && code.ends_with(')') {
        let s = vm.eval_value(&code[11..code.len() - 1]).ok()?.as_text()?;
        let mut chars: Vec<char> = s.chars().collect();
        chars.sort();
        let r: String = chars.into_iter().collect();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~levenshtein(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let a = vm.eval_value(&p[0]).ok()?.as_text()?;
            let b = vm.eval_value(&p[1]).ok()?.as_text()?;
            let (m, n) = (a.len(), b.len());
            let mut dp = vec![vec![0u16; n + 1]; m + 1];
            for i in 0..=m { dp[i][0] = i as u16; }
            for j in 0..=n { dp[0][j] = j as u16; }
            for (i, ca) in a.chars().enumerate() {
                for (j, cb) in b.chars().enumerate() {
                    dp[i+1][j+1] = if ca == cb { dp[i][j] } else {
                        (dp[i][j] + 1).min(dp[i+1][j] + 1).min(dp[i][j+1] + 1)
                    };
                }
            }
            let r = dp[m][n] as f64;
            vm.echo(Value::fmt_num(r));
            return Some(Ok(Value::Number(r)));
        }
    }

    if code.starts_with("~rot13(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len() - 1]).ok()?.as_text()?;
        let r: String = s.chars().map(|c| match c {
            'a'..='m' | 'A'..='M' => ((c as u8) + 13) as char,
            'n'..='z' | 'N'..='Z' => ((c as u8) - 13) as char,
            _ => c,
        }).collect();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~caesar(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?;
            let shift = vm.eval_expr(&p[1]).ok()? as i32;
            let r: String = s.chars().map(|c| {
                if c.is_ascii_alphabetic() {
                    let first = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                    ((((c as u8).wrapping_sub(first)).wrapping_add(shift as u8)) % 26 + first) as char
                } else { c }
            }).collect();
            vm.echo(&r);
            return Some(Ok(Value::Text(r)));
        }
    }

    if code.starts_with("~atbash(") && code.ends_with(')') {
        let s = vm.eval_value(&code[8..code.len() - 1]).ok()?.as_text()?;
        let r: String = s.chars().map(|c| {
            if c.is_ascii_lowercase() { (b'a' + b'z' - c as u8) as char }
            else if c.is_ascii_uppercase() { (b'A' + b'Z' - c as u8) as char }
            else { c }
        }).collect();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~pig_latin(") && code.ends_with(')') {
        let s = vm.eval_value(&code[11..code.len() - 1]).ok()?.as_text()?;
        let r = s.split_whitespace().map(|w| {
            if w.is_empty() { return w.to_string(); }
            let mut ch = w.chars();
            let first = ch.next().unwrap();
            if "aeiouAEIOU".contains(first) { format!("{}way", w) }
            else { format!("{}{}ay", ch.as_str(), first) }
        }).collect::<Vec<_>>().join(" ");
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~rle_encode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len() - 1]).ok()?.as_text()?;
        let mut r = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            let mut count = 1u32;
            while let Some(&n) = chars.peek() { if n == c { count += 1; chars.next(); } else { break; } }
            r.push_str(&format!("{}{}", c, count));
        }
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~rle_decode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len() - 1]).ok()?.as_text()?;
        let mut r = String::new();
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            let n: String = chars.by_ref().take_while(|c| c.is_ascii_digit()).collect();
            let count: usize = n.parse().unwrap_or(1);
            r.push_str(&c.to_string().repeat(count));
        }
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~url_encode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len() - 1]).ok()?.as_text()?;
        let r: String = s.chars().map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        }).collect();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~url_decode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len() - 1]).ok()?.as_text()?;
        let bytes = s.as_bytes();
        let mut r = String::new();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                // Читаем кусок по байтам, поскольку индексы здесь байтовые
                // Строку срезать нельзя, так как срез упадёт в середине русской буквы
                if let Ok(pair) = std::str::from_utf8(&bytes[i+1..i+3]) {
                    if let Ok(n) = u8::from_str_radix(pair, 16) { r.push(n as char); i += 3; continue; }
                }
            } else if bytes[i] == b'+' { r.push(' '); } else { r.push(bytes[i] as char); }
            i += 1;
        }
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~html_escape(") && code.ends_with(')') {
        let s = vm.eval_value(&code[13..code.len() - 1]).ok()?.as_text()?;
        let r = s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;");
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~html_unescape(") && code.ends_with(')') {
        let s = vm.eval_value(&code[15..code.len() - 1]).ok()?.as_text()?;
        let r = s.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'");
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~base64_encode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[15..code.len() - 1]).ok()?.as_text()?;
        use base64::Engine;
        let r = base64::engine::general_purpose::STANDARD.encode(&s);
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~base64_decode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[15..code.len() - 1]).ok()?.as_text()?;
        use base64::Engine;
        match base64::engine::general_purpose::STANDARD.decode(&s) {
            Ok(bytes) => { let r = String::from_utf8_lossy(&bytes).to_string(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
            Err(e) => return Some(Err(e.to_string())),
        }
    }

    if code.starts_with("~hex(") && code.ends_with(')') {
        let s = vm.eval_value(&code[5..code.len() - 1]).ok()?.as_text()?;
        let r: String = s.bytes().map(|b| format!("{:02X}", b)).collect();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~unhex(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len() - 1]).ok()?.as_text()?;
        let r: Result<String, _> = (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..(i+2).min(s.len())], 16).map(|b| b as char)).collect();
        match r { Ok(v) => { vm.echo(&v); return Some(Ok(Value::Text(v))); } Err(e) => return Some(Err(e.to_string())), }
    }

    if code.starts_with("~uuid(") && code.ends_with(')') || code == "~uuid" {
        use uuid::Uuid;
        let r = Uuid::new_v4().to_string();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~hash(") && code.ends_with(')') {
        let s = vm.eval_value(&code[6..code.len() - 1]).ok()?.as_text()?;
        let mut h: u64 = 5381;
        for b in s.bytes() { h = h.wrapping_mul(33).wrapping_add(b as u64); }
        let r = format!("{:016X}", h);
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~crc32(") && code.ends_with(')') {
        let s = vm.eval_value(&code[7..code.len() - 1]).ok()?.as_text()?;
        let r = format!("{:08X}", s.bytes().fold(0xFFFFFFFFu32, |mut c, b| {
            c ^= b as u32;
            for _ in 0..8 { c = if c & 1 != 0 { c >> 1 ^ 0xEDB88320 } else { c >> 1 }; }
            c
        }) ^ 0xFFFFFFFF);
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~morse_encode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[14..code.len() - 1]).ok()?.as_text()?.to_uppercase();
        let morse: std::collections::HashMap<char, &str> = [
            ('A', ".-"), ('B', "-..."), ('C', "-.-."), ('D', "-.."), ('E', "."), ('F', "..-."),
            ('G', "--."), ('H', "...."), ('I', ".."), ('J', ".---"), ('K', "-.-"), ('L', ".-.."),
            ('M', "--"), ('N', "-."), ('O', "---"), ('P', ".--."), ('Q', "--.-"), ('R', ".-."),
            ('S', "..."), ('T', "-"), ('U', "..-"), ('V', "...-"), ('W', ".--"), ('X', "-..-"),
            ('Y', "-.--"), ('Z', "--.."), ('0', "-----"), ('1', ".----"), ('2', "..---"),
            ('3', "...--"), ('4', "....-"), ('5', "....."), ('6', "-...."), ('7', "--..."),
            ('8', "---.."), ('9', "----."),
        ].iter().cloned().collect();
        let r = s.chars().filter_map(|c| morse.get(&c).copied()).collect::<Vec<_>>().join(" ");
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~morse_decode(") && code.ends_with(')') {
        let s = vm.eval_value(&code[14..code.len() - 1]).ok()?.as_text()?;
        let morse: std::collections::HashMap<&str, char> = [
            (".-", 'A'), ("-...", 'B'), ("-.-.", 'C'), ("-..", 'D'), (".", 'E'), ("..-.", 'F'),
            ("--.", 'G'), ("....", 'H'), ("..", 'I'), (".---", 'J'), ("-.-", 'K'), (".-..", 'L'),
            ("--", 'M'), ("-.", 'N'), ("---", 'O'), (".--.", 'P'), ("--.-", 'Q'), (".-.", 'R'),
            ("...", 'S'), ("-", 'T'), ("..-", 'U'), ("...-", 'V'), (".--", 'W'), ("-..-", 'X'),
            ("-.--", 'Y'), ("--..", 'Z'), ("-----", '0'), (".----", '1'), ("..---", '2'),
            ("...--", '3'), ("....-", '4'), (".....", '5'), ("-....", '6'), ("--...", '7'),
            ("---..", '8'), ("----.", '9'),
        ].iter().cloned().collect();
        let r: String = s.split(' ').filter_map(|code| morse.get(code)).collect();
        vm.echo(&r);
        return Some(Ok(Value::Text(r)));
    }

    if code.starts_with("~strip_punct(") && code.ends_with(')') { let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().filter(|c| c.is_alphanumeric()||c.is_whitespace()).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~strip_digits(") && code.ends_with(')') { let s = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().filter(|c| !c.is_ascii_digit()).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~strip_whitespace(") && code.ends_with(')') { let s = vm.eval_value(&code[19..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().filter(|c| !c.is_whitespace()).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~keep_digits(") && code.ends_with(')') { let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().filter(|c| c.is_ascii_digit()).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~keep_alpha(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().filter(|c| c.is_alphabetic()).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~censor(") && code.ends_with(')') { let s = vm.eval_value(&code[8..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().map(|c| if c.is_alphanumeric() {'*'}else{c}).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~mask_email(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let at = s.find('@').unwrap_or(s.len()); let r = if at>0 { format!("{}***{}", &s[..1], &s[at..]) } else { s }; vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~mask_phone(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let r = if s.len()>6 { format!("{}****{}", &s[..3], &s[s.len()-4..]) } else { s }; vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~initials(") && code.ends_with(')') { let s = vm.eval_value(&code[10..code.len()-1]).ok()?.as_text()?; let r: String = s.split_whitespace().filter_map(|w| w.chars().next()).map(|c| c.to_uppercase().next().unwrap_or(c)).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~acronym(") && code.ends_with(')') { let s = vm.eval_value(&code[9..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().filter(|c| c.is_uppercase()).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~normalize_spaces(") && code.ends_with(')') { let s = vm.eval_value(&code[19..code.len()-1]).ok()?.as_text()?; let r = s.split_whitespace().collect::<Vec<_>>().join(" "); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~remove_duplicates(") && code.ends_with(')') { let s = vm.eval_value(&code[19..code.len()-1]).ok()?.as_text()?; let mut r = String::new(); for c in s.chars() { if !r.contains(c) { r.push(c); } } vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~syllables(") && code.ends_with(')') { let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; let vowels = "aeiouаеёиоуыэюя"; let mut count = 0; let mut prev_vowel = false; for c in s.to_lowercase().chars() { let is_v = vowels.contains(c); if is_v && !prev_vowel { count += 1; } prev_vowel = is_v; } vm.echo(count.max(1)); return Some(Ok(Value::Number(count.max(1) as f64))); }
    if code.starts_with("~word_count(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let count = s.split_whitespace().count(); vm.echo(count); return Some(Ok(Value::Number(count as f64))); }
    if code.starts_with("~char_count(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; vm.echo(s.chars().count()); return Some(Ok(Value::Number(s.chars().count() as f64))); }
    if code.starts_with("~line_count(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let count = s.lines().count(); vm.echo(count); return Some(Ok(Value::Number(count as f64))); }
    if code.starts_with("~has_upper(") && code.ends_with(')') { let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; vm.echo(s.chars().any(|c|c.is_uppercase())); return Some(Ok(Value::Bool(s.chars().any(|c|c.is_uppercase())))); }
    if code.starts_with("~has_lower(") && code.ends_with(')') { let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; vm.echo(s.chars().any(|c|c.is_lowercase())); return Some(Ok(Value::Bool(s.chars().any(|c|c.is_lowercase())))); }
    if code.starts_with("~has_digit(") && code.ends_with(')') { let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; vm.echo(s.chars().any(|c|c.is_ascii_digit())); return Some(Ok(Value::Bool(s.chars().any(|c|c.is_ascii_digit())))); }
    if code.starts_with("~has_space(") && code.ends_with(')') { let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; vm.echo(s.chars().any(|c|c.is_whitespace())); return Some(Ok(Value::Bool(s.chars().any(|c|c.is_whitespace())))); }
    if code.starts_with("~has_punct(") && code.ends_with(')') { let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; vm.echo(s.chars().any(|c|c.is_ascii_punctuation())); return Some(Ok(Value::Bool(s.chars().any(|c|c.is_ascii_punctuation())))); }
    if code.starts_with("~is_title(") && code.ends_with(')') { let s = vm.eval_value(&code[10..code.len()-1]).ok()?.as_text()?; let mut prev = ' '; let r = s.chars().all(|c|{let r=if prev.is_whitespace(){c.is_uppercase()}else{c.is_lowercase()};prev=c;r}); vm.echo(r); return Some(Ok(Value::Bool(r))); }
    if code.starts_with("~pad_left(") && code.ends_with(')') { let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = vm.eval_expr(&p[1]).ok()? as usize; let c = if p.len()>=3{vm.eval_value(&p[2]).ok()?.as_text()?.chars().next().unwrap_or(' ')}else{' '}; let r = format!("{}{}", c.to_string().repeat(n.saturating_sub(s.len())), s); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~pad_right(") && code.ends_with(')') { let p = VM::split_top_level(&code[11..code.len()-1], ','); if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = vm.eval_expr(&p[1]).ok()? as usize; let c = if p.len()>=3{vm.eval_value(&p[2]).ok()?.as_text()?.chars().next().unwrap_or(' ')}else{' '}; let r = format!("{}{}", s, c.to_string().repeat(n.saturating_sub(s.len()))); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~pad_both(") && code.ends_with(')') { let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = vm.eval_expr(&p[1]).ok()? as usize; let c = if p.len()>=3{vm.eval_value(&p[2]).ok()?.as_text()?.chars().next().unwrap_or(' ')}else{' '}; let pad = n.saturating_sub(s.len()); let l = pad/2; let r = format!("{}{}{}", c.to_string().repeat(l), s, c.to_string().repeat(pad-l)); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~wrap_angle(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let r = format!("<{}>", s); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~wrap_square(") && code.ends_with(')') { let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?; let r = format!("[{}]", s); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~wrap_brace(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let r = format!("{{{}}}", s); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~wrap_paren(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let r = format!("({})", s); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~wrap_quote(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let r = format!("\"{}\"", s); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~markdown_bold(") && code.ends_with(')') { let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text()?; let r = format!("**{}**", s); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~markdown_italic(") && code.ends_with(')') { let s = vm.eval_value(&code[17..code.len()-1]).ok()?.as_text()?; let r = format!("*{}*", s); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~markdown_code(") && code.ends_with(')') { let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text()?; let r = format!("`{}`", s); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~markdown_link(") && code.ends_with(')') { let p = VM::split_top_level(&code[15..code.len()-1], ','); if p.len() >= 2 { let t = vm.eval_value(&p[0]).ok()?.as_text()?; let u = vm.eval_value(&p[1]).ok()?.as_text()?; let r = format!("[{}]({})", t, u); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~json_format(") && code.ends_with(')') { let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?; let r = s.replace(",", ", ").replace(":", ": "); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~json_minify(") && code.ends_with(')') { let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().filter(|c|!c.is_whitespace()).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~tab_to_space(") && code.ends_with(')') { let p = VM::split_top_level(&code[14..code.len()-1], ','); if p.len() >= 1 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = if p.len()>=2{vm.eval_expr(&p[1]).ok()? as usize}else{4}; let r = s.replace("\t", &" ".repeat(n)); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~space_to_tab(") && code.ends_with(')') { let p = VM::split_top_level(&code[14..code.len()-1], ','); if p.len() >= 1 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = if p.len()>=2{vm.eval_expr(&p[1]).ok()? as usize}else{4}; let r = s.replace(&" ".repeat(n), "\t"); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~swap_words(") && code.ends_with(')') { let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_value(&p[0]).ok()?.as_text()?; let b = vm.eval_value(&p[1]).ok()?.as_text()?; let r = format!("{} {}", b, a); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~shuffle_chars(") && code.ends_with(')') { let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text()?; let mut c: Vec<char> = s.chars().collect(); use rand::seq::SliceRandom; c.shuffle(&mut rand::thread_rng()); let r: String = c.into_iter().collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~random_case(") && code.ends_with(')') { let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().map(|c| if rand::random::<bool>(){c.to_uppercase().next().unwrap_or(c)}else{c.to_lowercase().next().unwrap_or(c)}).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~alternating_case(") && code.ends_with(')') { let s = vm.eval_value(&code[18..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().enumerate().map(|(i,c)| if i%2==0{c.to_uppercase().next().unwrap_or(c)}else{c.to_lowercase().next().unwrap_or(c)}).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~shorten(") && code.ends_with(')') { let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = vm.eval_expr(&p[1]).ok()? as usize; let r: String = s.chars().take(n).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~first_word(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let r = s.split_whitespace().next().unwrap_or("").to_string(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~last_word(") && code.ends_with(')') { let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; let r = s.split_whitespace().last().unwrap_or("").to_string(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~nth_word(") && code.ends_with(')') { let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = vm.eval_expr(&p[1]).ok()? as usize; let r = s.split_whitespace().nth(n).unwrap_or("").to_string(); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~longest_word(") && code.ends_with(')') { let s = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text()?; let r = s.split_whitespace().max_by_key(|w| w.len()).unwrap_or("").to_string(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~shortest_word(") && code.ends_with(')') { let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text()?; let r = s.split_whitespace().min_by_key(|w| w.len()).unwrap_or("").to_string(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~unique_words(") && code.ends_with(')') { let s = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text()?; let mut seen = std::collections::HashSet::new(); let r: Vec<&str> = s.split_whitespace().filter(|w| seen.insert(*w)).collect(); vm.echo(r.join(" ")); return Some(Ok(Value::Text(r.join(" ")))); }
    if code.starts_with("~freq_words(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let mut freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new(); for w in s.split_whitespace() { *freq.entry(w.to_string()).or_insert(0) += 1; } let r: Vec<Value> = freq.into_iter().map(|(k, v)| Value::List(vec![Value::Text(k), Value::Number(v as f64)])).collect(); let rv = Value::List(r); vm.echo(rv.display()); return Some(Ok(rv)); }
    if code.starts_with("~random_word(") && code.ends_with(')') { let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?; let words: Vec<&str> = s.split_whitespace().collect(); if words.is_empty() { return Some(Ok(Value::None)); } use rand::Rng; let r = words[rand::thread_rng().gen_range(0..words.len())].to_string(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~encode_rot13(") && code.ends_with(')') { let s = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().map(|c| match c { 'a'..='z' => (((c as u8 - b'a' + 13) % 26) + b'a') as char, 'A'..='Z' => (((c as u8 - b'A' + 13) % 26) + b'A') as char, _ => c }).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~encode_base64(") && code.ends_with(')') { let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text()?; use base64::Engine; let r = base64::engine::general_purpose::STANDARD.encode(s.as_bytes()); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~decode_base64(") && code.ends_with(')') { let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text()?; use base64::Engine; let b = base64::engine::general_purpose::STANDARD.decode(&s).ok()?; let r = String::from_utf8_lossy(&b).to_string(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~escape_json(") && code.ends_with(')') { let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?; let r = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t"); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~unescape_json(") && code.ends_with(')') { let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text()?; let r = s.replace("\\\"", "\"").replace("\\n", "\n").replace("\\r", "\r").replace("\\t", "\t").replace("\\\\", "\\"); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~escape_xml(") && code.ends_with(')') { let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?; let r = s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("'", "&apos;").replace("\"", "&quot;"); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~escape_regex(") && code.ends_with(')') { let s = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text()?; let r: String = s.chars().map(|c| if ".*+?^$()[]{}|\\".contains(c) { format!("\\{}", c) } else { c.to_string() }).collect(); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~pad_number(") && code.ends_with(')') { let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 2 { let n = vm.eval_expr(&p[0]).ok()? as i64; let w = vm.eval_expr(&p[1]).ok()? as usize; let r = format!("{:0>width$}", n, width = w); vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }
    if code.starts_with("~format_money(") && code.ends_with(')') { let n = vm.eval_expr(&code[14..code.len()-1]).ok()?; let r = format!("${:.2}", n); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~format_percent(") && code.ends_with(')') { let n = vm.eval_expr(&code[16..code.len()-1]).ok()?; let r = format!("{:.1}%", n); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~commas(") && code.ends_with(')') { let n = vm.eval_expr(&code[8..code.len()-1]).ok()? as i64; let r = n.to_string().as_bytes().rchunks(3).map(std::str::from_utf8).rev().collect::<Result<Vec<_>,_>>().ok()?.join(","); vm.echo(&r); return Some(Ok(Value::Text(r))); }
    if code.starts_with("~plural(") && code.ends_with(')') { let p = VM::split_top_level(&code[7..code.len()-1], ','); if p.len() >= 2 { let n = vm.eval_expr(&p[0]).ok()? as i64; let s = vm.eval_value(&p[1]).ok()?.as_text()?; let r = if n == 1 { format!("{} {}", n, s) } else { format!("{} {}s", n, s) }; vm.echo(&r); return Some(Ok(Value::Text(r))); } else { return None; } }

    st1!("~casefold(", 10, |s: &str| s.to_lowercase());
    st1!("~capitalize_first(", 18, |s: &str| { let mut c = s.chars(); match c.next() { Some(x) => x.to_uppercase().collect::<String>() + c.as_str(), None => String::new() } });
    st1!("~trim_start(", 12, |s: &str| s.trim_start().to_string());
    st1!("~trim_end(", 10, |s: &str| s.trim_end().to_string());
    st1!("~swap_case_words(", 17, |s: &str| s.split_whitespace().map(|w| w.chars().map(|c| if c.is_uppercase() { c.to_lowercase().next().unwrap_or(c) } else { c.to_uppercase().next().unwrap_or(c) }).collect::<String>()).collect::<Vec<_>>().join(" "));
    st1!("~scramble_words(", 16, |s: &str| { let mut v: Vec<&str> = s.split_whitespace().collect(); use rand::seq::SliceRandom; v.shuffle(&mut rand::thread_rng()); v.join(" ") });
    st1!("~rotate_str(", 12, |s: &str| { if s.is_empty() { String::new() } else { let mut c = s.chars(); let last = c.next_back().unwrap(); let mut r = last.to_string(); r.push_str(c.as_str()); r } });
    st1!("~rot18(", 7, |s: &str| s.chars().map(|c| { if c.is_ascii_alphabetic() { let base = if c.is_ascii_uppercase() { b'A' } else { b'a' }; (((c as u8 - base + 13) % 26) + base) as char } else if c.is_ascii_digit() { let d = (c as u8 - b'0' + 5) % 10; (d + b'0') as char } else { c } }).collect());
    st1!("~rot47_str(", 11, |s: &str| s.chars().map(|c| { let v = c as u32; if (33..=126).contains(&v) { let n = v - 33 + 47; if n > 93 { (n - 94 + 33) as u8 as char } else { (n + 33) as u8 as char } } else { c } }).collect());
    st1!("~pascal_case_str(", 17, |s: &str| { let mut out = String::new(); let mut cap = true; for c in s.chars() { if c == '_' || c == '-' || c == ' ' { cap = true; continue } if cap { out.extend(c.to_uppercase()); cap = false } else { out.push(c) } } out });
    st1!("~constant_case_str(", 19, |s: &str| { let mut out = String::new(); for c in s.chars() { if c.is_ascii_whitespace() || c == '_' || c == '-' || c == ' ' { out.push('_') } else { out.extend(c.to_uppercase()) } } out });
    st1!("~train_case_str(", 16, |s: &str| s.split(|c: char| c == ' ' || c == '_').filter(|w| !w.is_empty()).collect::<Vec<_>>().join("-").to_lowercase());
    st1!("~bytes_to_hex(", 14, |s: &str| s.bytes().map(|b| format!("{:02x} ", b)).collect::<String>().trim_end().to_string());
    st1!("~hex_to_bytes(", 14, |s: &str| { let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect(); let b = clean.as_bytes(); let mut r = String::new(); let mut i = 0; while i + 1 < b.len() { if let Ok(pair) = std::str::from_utf8(&b[i..i+2]) { if let Ok(v) = u8::from_str_radix(pair, 16) { r.push(v as char) } } i += 2 } r });
    st1!("~ascii_only(", 12, |s: &str| s.chars().filter(|c| c.is_ascii()).collect());
    st1!("~printable_only(", 16, |s: &str| s.chars().filter(|c| c.is_ascii_graphic() || *c == ' ' || *c == '\n' || *c == '\t').collect());
    st1!("~split_words_camel(", 19, |s: &str| { let mut out = String::new(); for c in s.chars() { if c.is_ascii_uppercase() && !out.is_empty() { out.push(' ') } out.push(c) } out });
    st1!("~merge_spaces(", 14, |s: &str| { let mut out = String::new(); let mut sp = false; for c in s.chars() { if c.is_whitespace() { if !sp { out.push(' '); sp = true } } else { out.push(c); sp = false } } out.trim().to_string() });
    st1!("~quote_lines(", 13, |s: &str| s.lines().map(|l| format!("> {}", l)).collect::<Vec<_>>().join("\n"));
    st1!("~unquote_lines(", 15, |s: &str| s.lines().map(|l| l.strip_prefix("> ").unwrap_or(l).to_string()).collect::<Vec<_>>().join("\n"));
    st1!("~strip_bom(", 11, |s: &str| s.trim_start_matches('\u{feff}').to_string());
    st1!("~add_bom(", 9, |s: &str| format!("\u{feff}{}", s));
    st1!("~strip_comments(", 16, |s: &str| s.lines().filter(|l| !l.trim_start().starts_with('#')).collect::<Vec<_>>().join("\n"));
    st1!("~censor_words(", 14, |s: &str| s.split_whitespace().map(|w| if w.chars().count() <= 3 { "***".to_string() } else { format!("{}***", w.chars().next().unwrap()) }).collect::<Vec<_>>().join(" "));
    st1!("~expand_tabs(", 13, |s: &str| s.replace('\t', "    "));
    st1!("~shrink_tabs(", 13, |s: &str| { let mut out = String::new(); let mut sp = 0; for c in s.chars() { if c == ' ' { sp += 1; if sp == 4 { out.push('\t'); sp = 0 } } else { for _ in 0..sp { out.push(' ') } sp = 0; out.push(c) } } out });
    st1!("~signed_str(", 12, |s: &str| if s.starts_with('-') { s.to_string() } else { format!("+{}", s) });
    st1!("~fill_paragraph(", 16, |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" "));
    st1!("~left_align_words(", 18, |s: &str| s.split_whitespace().map(|w| format!("{:<20}", w)).collect::<Vec<_>>().join(" "));
    st1!("~right_align_words(", 19, |s: &str| s.split_whitespace().map(|w| format!("{:>20}", w)).collect::<Vec<_>>().join(" "));
    st1!("~center_words(", 14, |s: &str| s.split_whitespace().map(|w| { let n = w.chars().count(); let p = (20usize.saturating_sub(n)) / 2; format!("{}{}{}", " ".repeat(p), w, " ".repeat(20 - n - p)) }).collect::<Vec<_>>().join(" "));
    st1!("~strip_ansi(", 12, |s: &str| { let mut out = String::new(); let mut esc = false; for c in s.chars() { if c == '\u{1b}' { esc = true } else if esc { if c == 'm' { esc = false } } else { out.push(c) } } out });
    st1!("~double_str(", 12, |s: &str| s.chars().flat_map(|c| [c, c]).collect());
    st1!("~words_shuffle(", 15, |s: &str| { let mut v: Vec<char> = s.chars().collect(); use rand::seq::SliceRandom; v.shuffle(&mut rand::thread_rng()); v.into_iter().collect() });
    st1!("~letters_sort(", 14, |s: &str| { let mut v: Vec<char> = s.chars().filter(|c| c.is_alphabetic()).collect(); v.sort(); v.into_iter().collect() });
    st1!("~markdown_escape(", 17, |s: &str| s.replace('*', "\\*").replace('_', "\\_").replace('`', "\\`").replace('[', "\\["));
    st1!("~wrap_each_word(", 16, |s: &str| s.split_whitespace().map(|w| format!("({})", w)).collect::<Vec<_>>().join(" "));
    st1!("~first_letters(", 15, |s: &str| s.split_whitespace().filter_map(|w| w.chars().next()).collect());
    st1!("~word_shuffle_keep(", 19, |s: &str| s.split_whitespace().map(|w| { let n = w.chars().count(); if n <= 3 { w.to_string() } else { let f = w.chars().next().unwrap(); let l = w.chars().last().unwrap(); let mut m: Vec<char> = w.chars().skip(1).take(n - 2).collect(); use rand::seq::SliceRandom; m.shuffle(&mut rand::thread_rng()); format!("{}{}{}", f, m.into_iter().collect::<String>(), l) } }).collect::<Vec<_>>().join(" "));
    st1!("~mask_card(", 11, |s: &str| { let n = s.chars().count(); if n <= 4 { s.to_string() } else { let tail: String = s.chars().skip(n - 4).collect(); format!("{}****{}", "".repeat(0), tail).trim().to_string() } });
    st1!("~ord_suffix(", 12, |s: &str| { let n = s.parse::<i64>().unwrap_or(0); let suffix = match n % 100 { 11 | 12 | 13 => "th", _ => match n % 10 { 1 => "st", 2 => "nd", 3 => "rd", _ => "th" } }; format!("{}{}", n, suffix) });
    st1!("~email_local_part(", 18, |s: &str| s.split('@').next().unwrap_or("").to_string());
    st1!("~email_domain(", 14, |s: &str| s.split('@').nth(1).unwrap_or("").to_string());
    st1!("~url_scheme(", 12, |s: &str| s.split("://").next().unwrap_or("").to_string());
    st1!("~url_hostname(", 14, |s: &str| { let after = s.split("://").nth(1).unwrap_or(s); after.split(['/', '?', '#']).next().unwrap_or("").to_string() });
    st1!("~url_pathname(", 14, |s: &str| { let after = s.split("://").nth(1).unwrap_or(s); let noq = after.split(['?', '#']).next().unwrap_or(after); noq.find('/').map(|i| noq[i..].to_string()).unwrap_or_default() });
    st1!("~int_with_sep(", 14, |s: &str| { let c: Vec<char> = s.chars().collect(); let mut r = String::new(); for (i, ch) in c.iter().enumerate() { if i > 0 && (c.len() - i) % 3 == 0 { r.push(',') } r.push(*ch) } r });
    st1!("~strip_emoji(", 13, |s: &str| s.chars().filter(|c| { let u = *c as u32; !(0x1F300..=0x1F64F).contains(&u) && !(0x1F680..=0x1F6FF).contains(&u) }).collect());

    st1n!("~count_upper(", 13, |s: &str| s.chars().filter(|c| c.is_uppercase()).count());
    st1n!("~count_lower(", 13, |s: &str| s.chars().filter(|c| c.is_lowercase()).count());
    st1n!("~count_vowels_str(", 18, |s: &str| s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count());
    st1n!("~count_consonants_str(", 22, |s: &str| s.chars().filter(|c| c.is_alphabetic() && !"aeiouAEIOU".contains(*c)).count());
    st1n!("~count_spaces(", 14, |s: &str| s.chars().filter(|c| c.is_whitespace()).count());
    st1n!("~count_sentences_str(", 21, |s: &str| s.chars().filter(|c| *c == '.' || *c == '!' || *c == '?').count());
    st1n!("~count_newlines(", 16, |s: &str| s.chars().filter(|c| *c == '\n').count());
    st1n!("~count_tabs(", 12, |s: &str| s.chars().filter(|c| *c == '\t').count());
    st1n!("~count_punct(", 13, |s: &str| s.chars().filter(|c| c.is_ascii_punctuation()).count());
    st1n!("~total_chars(", 13, |s: &str| s.chars().count());
    st1n!("~total_bytes(", 13, |s: &str| s.len());
    st1n!("~longest_word_len(", 18, |s: &str| s.split_whitespace().map(|w| w.chars().count()).max().unwrap_or(0));
    st1n!("~shortest_word_len(", 19, |s: &str| s.split_whitespace().map(|w| w.chars().count()).min().unwrap_or(0));
    st1n!("~avg_word_len(", 14, |s: &str| { let words: Vec<&str> = s.split_whitespace().collect(); if words.is_empty() { 0.0 } else { let total: usize = words.iter().map(|w| w.chars().count()).sum(); total as f64 / words.len() as f64 } });
    st1n!("~ansi_length(", 13, |s: &str| { let mut n = 0usize; let mut esc = false; for c in s.chars() { if c == '\u{1b}' { esc = true } else if esc { if c == 'm' { esc = false } } else { n += 1 } } n });
    st1n!("~count_tokens(", 14, |s: &str| s.split_whitespace().count());

    st1b!("~is_title_case(", 15, |s: &str| s.split_whitespace().all(|w| w.chars().next().map_or(false, |c| c.is_uppercase())));
    st1b!("~is_sentence_case(", 18, |s: &str| s.chars().next().map_or(false, |c| c.is_uppercase()));
    st1b!("~is_palindrome_str(", 19, |s: &str| { let clean: String = s.chars().filter(|c| c.is_alphanumeric()).map(|c| c.to_lowercase().next().unwrap_or(c)).collect(); clean.chars().eq(clean.chars().rev()) });
    st1b!("~is_blank_str(", 14, |s: &str| s.trim().is_empty());
    st1b!("~utf8_validate(", 15, |s: &str| std::str::from_utf8(s.as_bytes()).is_ok());
    st1b!("~is_absolute_path(", 18, |s: &str| s.starts_with('/') || s.starts_with("\\") || s.len() >= 3 && s.as_bytes()[1] == b':' && (s.as_bytes()[2] == b'\\' || s.as_bytes()[2] == b'/'));
    st1b!("~char_is_emoji(", 15, |s: &str| { let u = s.chars().next().map(|c| c as u32).unwrap_or(0); (0x1F300..=0x1F64F).contains(&u) || (0x1F680..=0x1F6FF).contains(&u) });

    st1l!("~split_whitespace2(", 19, |s: &str| -> Vec<String> { s.split_whitespace().map(|w| w.to_string()).collect() });
    st1l!("~split_lines2(", 14, |s: &str| -> Vec<String> { s.lines().map(|l| l.to_string()).collect() });
    st1l!("~split_sentences(", 17, |s: &str| { let mut out = Vec::new(); let mut cur = String::new(); for c in s.chars() { cur.push(c); if c == '.' || c == '!' || c == '?' { out.push(cur.trim().to_string()); cur.clear(); } } if !cur.trim().is_empty() { out.push(cur.trim().to_string()); } out });
    st1l!("~split_chars(", 13, |s: &str| -> Vec<String> { s.chars().map(|c| c.to_string()).collect() });
    st1l!("~tokenize(", 10, |s: &str| -> Vec<String> { s.split(|c: char| !c.is_alphanumeric()).filter(|t| !t.is_empty()).map(|t| t.to_string()).collect() });
    st1l!("~extract_email(", 15, |s: &str| -> Vec<String> { s.split_whitespace().filter(|t| t.contains('@') && t.contains('.')).map(|t| t.trim_matches(|c: char| !c.is_alphanumeric() && c != '@' && c != '.' && c != '_' && c != '-').to_string()).collect() });
    st1l!("~extract_urls(", 14, |s: &str| -> Vec<String> { s.split_whitespace().filter(|t| t.contains("://") || t.starts_with("www.")).map(|t| t.trim_matches(|c: char| !c.is_alphanumeric() && c != ':' && c != '/' && c != '.' && c != '?' && c != '=' && c != '&' && c != '-' && c != '_').to_string()).collect() });
    st1l!("~extract_phone(", 15, |s: &str| -> Vec<String> { s.split_whitespace().filter(|t| t.chars().filter(|c| c.is_ascii_digit()).count() >= 7).map(|t| t.to_string()).collect() });
    st1l!("~extract_ip_str(", 16, |s: &str| -> Vec<String> { s.split_whitespace().filter(|t| { let parts: Vec<&str> = t.split('.').collect(); parts.len() == 4 && parts.iter().all(|p| p.parse::<u16>().map(|v| v <= 255).unwrap_or(false)) }).map(|t| t.to_string()).collect() });
    st1l!("~word_positions(", 16, |s: &str| { let mut out = Vec::new(); let mut in_w = false; let mut start = 0usize; for (i, c) in s.char_indices() { if c.is_alphanumeric() { if !in_w { start = i; in_w = true } } else if in_w { out.push(start.to_string()); in_w = false } } if in_w { out.push(start.to_string()); } out });
    st1l!("~word_lengths(", 14, |s: &str| -> Vec<String> { s.split_whitespace().map(|w| w.chars().count().to_string()).collect() });
    st1l!("~csv_parse_row(", 15, |s: &str| -> Vec<String> { s.split(',').map(|f| f.trim().trim_matches('"').to_string()).collect() });

    st1!("~remove_vowels(", 15, |s: &str| s.chars().filter(|c| !"aeiouAEIOU".contains(*c)).collect());
    st1!("~remove_consonants(", 19, |s: &str| s.chars().filter(|c| !c.is_alphabetic() || "aeiouAEIOU".contains(*c)).collect());

    st2!("~replace_first(", 15, |s: &str, a: &str| if a.is_empty() { s.to_string() } else { s.replacen(a, "", 1) });
    st2!("~replace_last(", 14, |s: &str, a: &str| { if a.is_empty() { s.to_string() } else if let Some(pos) = s.rfind(a) { format!("{}{}{}", &s[..pos], "", &s[pos + a.len()..]) } else { s.to_string() } });
    st2!("~remove_substring(", 18, |s: &str, a: &str| s.replace(a, ""));
    st2!("~remove_prefix_str(", 19, |s: &str, a: &str| s.strip_prefix(a).unwrap_or(s).to_string());
    st2!("~remove_suffix_str(", 19, |s: &str, a: &str| s.strip_suffix(a).unwrap_or(s).to_string());
    st2!("~vigenere_enc(", 14, |s: &str, k: &str| { let key: Vec<char> = k.chars().collect(); if key.is_empty() { s.to_string() } else { s.chars().enumerate().map(|(i, c)| { if c.is_ascii_alphabetic() { let kb = key[i % key.len()].to_ascii_lowercase() as u8 - b'a'; let base = if c.is_ascii_uppercase() { b'A' } else { b'a' }; (((c as u8 - base + kb) % 26) + base) as char } else { c } }).collect() } });
    st2!("~vigenere_dec(", 14, |s: &str, k: &str| { let key: Vec<char> = k.chars().collect(); if key.is_empty() { s.to_string() } else { s.chars().enumerate().map(|(i, c)| { if c.is_ascii_alphabetic() { let kb = key[i % key.len()].to_ascii_lowercase() as u8 - b'a'; let base = if c.is_ascii_uppercase() { b'A' } else { b'a' }; (((c as u8 - base + 26 - kb) % 26) + base) as char } else { c } }).collect() } });
    st2!("~xor_str(", 9, |s: &str, k: &str| { if k.is_empty() { s.to_string() } else { s.bytes().zip(k.bytes().cycle()).map(|(b, kb)| (b ^ kb) as char).collect() } });
    st2!("~trim_chars(", 12, |s: &str, ch: &str| s.trim_matches(|c| ch.contains(c)).to_string());
    st2!("~trim_start_chars(", 18, |s: &str, ch: &str| s.trim_start_matches(|c| ch.contains(c)).to_string());
    st2!("~trim_end_chars(", 16, |s: &str, ch: &str| s.trim_end_matches(|c| ch.contains(c)).to_string());
    st2!("~csv_field_quote(", 17, |s: &str, d: &str| { if s.contains(d) || s.contains('"') { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_string() } });
    st2!("~sql_quote_str(", 15, |s: &str, _: &str| s.replace('\'', "''"));
    st2!("~shell_quote_str(", 17, |s: &str, _: &str| format!("'{}'", s.replace('\'', "'\\''")));
    st2!("~glob_to_regex(", 15, |s: &str, _: &str| glob_to_regex(s));
    st2!("~path_join_str(", 15, |a: &str, b: &str| if a.ends_with('/') || a.ends_with('\\') { format!("{}{}", a, b) } else { format!("{}/{}", a, b) });
    st2!("~key_value_str(", 15, |k: &str, v: &str| format!("{}={}", k, v));
    st2!("~longest_common_prefix(", 23, |a: &str, b: &str| { let mut out = String::new(); for (x, y) in a.chars().zip(b.chars()) { if x == y { out.push(x) } else { break } } out });
    st2!("~longest_common_suffix(", 23, |a: &str, b: &str| { let mut out = String::new(); for (x, y) in a.chars().rev().zip(b.chars().rev()) { if x == y { out.insert(0, x) } else { break } } out });
    st2!("~longest_common_substring(", 26, |a: &str, b: &str| longest_common_substring(a, b));
    st2!("~similarity_pct(", 16, |a: &str, b: &str| { let m = a.chars().count().max(b.chars().count()); if m == 0 { "100%".to_string() } else { let lev = lev_dist(a, b); format!("{:.1}%", (1.0 - lev as f64 / m as f64) * 100.0) } });
    st2!("~mask_since(", 12, |s: &str, ch: &str| { if let Some(i) = s.find(ch) { format!("{}***", &s[..i]) } else { s.to_string() } });
    st2!("~before_str(", 12, |s: &str, sep: &str| s.split(sep).next().unwrap_or(s).to_string());
    st2!("~after_str(", 11, |s: &str, sep: &str| s.split(sep).nth(1).unwrap_or("").to_string());

    st2b!("~is_anagram(", 12, |a: &str, b: &str| { let mut x: Vec<char> = a.chars().filter(|c| !c.is_whitespace()).map(|c| c.to_lowercase().next().unwrap_or(c)).collect(); let mut y: Vec<char> = b.chars().filter(|c| !c.is_whitespace()).map(|c| c.to_lowercase().next().unwrap_or(c)).collect(); x.sort(); y.sort(); x == y });
    st2b!("~wildcard_match(", 16, |s: &str, p: &str| wildcard_match(s, p));
    st2b!("~contains_all(", 14, |s: &str, list: &str| list.split(',').all(|sub| !sub.trim().is_empty() && s.contains(sub.trim())));
    st2b!("~starts_with_any(", 17, |s: &str, list: &str| list.split(',').any(|sub| !sub.trim().is_empty() && s.starts_with(sub.trim())));
    st2b!("~ends_with_any(", 15, |s: &str, list: &str| list.split(',').any(|sub| !sub.trim().is_empty() && s.ends_with(sub.trim())));
    st2b!("~equals_ignore_case(", 20, |a: &str, b: &str| a.to_lowercase() == b.to_lowercase());

    st2n!("~pad_start(", 11, |s: &str, n: usize| format!("{:>w$}", s, w = n));
    st2n!("~pad_end(", 9, |s: &str, n: usize| format!("{:<w$}", s, w = n));
    st2n!("~pad_start_char(", 16, |s: &str, n: usize| format!("{}{}", "*".repeat(n.saturating_sub(s.chars().count())), s));
    st2n!("~pad_end_char(", 14, |s: &str, n: usize| format!("{}{}", s, "*".repeat(n.saturating_sub(s.chars().count()))));
    st2n!("~truncate_chars(", 16, |s: &str, n: usize| { let chars: Vec<char> = s.chars().collect(); if chars.len() > n { format!("{}…", chars[..n].iter().collect::<String>()) } else { s.to_string() } });
    st2n!("~insert_str(", 12, |s: &str, pos: usize| { let chars: Vec<char> = s.chars().collect(); let i = pos.min(chars.len()); let mut out: String = chars[..i].iter().collect(); out.push('*'); out.extend(chars[i..].iter()); out });
    st2n!("~delete_chars(", 14, |s: &str, pos: usize| { let chars: Vec<char> = s.chars().collect(); if pos >= chars.len() { s.to_string() } else { chars.iter().enumerate().filter(|(i, _)| *i != pos).map(|(_, c)| c).collect() } });
    st2n!("~take_str(", 10, |s: &str, n: usize| s.chars().take(n).collect());
    st2n!("~drop_str(", 10, |s: &str, n: usize| s.chars().skip(n).collect());
    st2n!("~add_line_numbers(", 18, |s: &str, start: usize| s.lines().enumerate().map(|(i, l)| format!("{:>3} {}", start + i, l)).collect::<Vec<_>>().join("\n"));
    st2n!("~prefix_lines(", 14, |s: &str, n: usize| s.lines().map(|l| format!("{}{}", " ".repeat(n), l)).collect::<Vec<_>>().join("\n"));
    st2n!("~suffix_lines(", 14, |s: &str, n: usize| s.lines().map(|l| format!("{}{}", l, " ".repeat(n))).collect::<Vec<_>>().join("\n"));
    st2n!("~repeat_each_str(", 17, |s: &str, n: usize| s.chars().flat_map(|c| std::iter::repeat(c).take(n)).collect());
    st2n!("~chunk_each(", 12, |s: &str, n: usize| s.chars().collect::<Vec<_>>().chunks(n.max(1)).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>().join(" "));
    st2n!("~wrap_every(", 12, |s: &str, n: usize| s.chars().collect::<Vec<_>>().chunks(n.max(1)).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>().join("\n"));
    st2n!("~tabify_lines(", 14, |s: &str, _: usize| { let mut out = Vec::new(); for l in s.lines() { let lead = l.chars().take_while(|c| *c == ' ').count(); let tabs = lead / 4; let rem = lead % 4; out.push(format!("{}{}{}", "\t".repeat(tabs), " ".repeat(rem), l.trim_start())); } out.join("\n") });
    st2n!("~insert_every(", 14, |s: &str, n: usize| { s.chars().enumerate().map(|(i, c)| if i > 0 && i % n == 0 { format!(" {}", c) } else { c.to_string() }).collect() });
    st2n!("~delete_nth_word(", 17, |s: &str, n: usize| s.split_whitespace().enumerate().filter(|(i, _)| *i != n).map(|(_, w)| w).collect::<Vec<_>>().join(" "));
    st2n!("~keep_nth_word(", 15, |s: &str, n: usize| s.split_whitespace().nth(n).unwrap_or("").to_string());
    st2n!("~take_words(", 12, |s: &str, n: usize| s.split_whitespace().take(n).collect::<Vec<_>>().join(" "));
    st2n!("~drop_words(", 12, |s: &str, n: usize| s.split_whitespace().skip(n).collect::<Vec<_>>().join(" "));
    st2n!("~last_n_chars(", 14, |s: &str, n: usize| { let chars: Vec<char> = s.chars().collect(); let skip = chars.len().saturating_sub(n); chars[skip..].iter().collect() });
    st2n!("~mid_chars(", 11, |s: &str, n: usize| s.chars().skip(n).collect());
    st2n!("~truncate_mid(", 14, |s: &str, n: usize| { let chars: Vec<char> = s.chars().collect(); if chars.len() <= n * 2 + 1 { s.to_string() } else { format!("{}…{}", chars[..n].iter().collect::<String>(), chars[chars.len() - n..].iter().collect::<String>()) } });
    st2n!("~number_prefix(", 15, |s: &str, start: usize| s.lines().enumerate().map(|(i, l)| format!("{}{}", start + i, l)).collect::<Vec<_>>().join("\n"));
    st2n!("~pad_number_str(", 16, |s: &str, n: usize| format!("{:>w$}", s, w = n));
    st2n!("~max_len_str(", 13, |s: &str, n: usize| format!("{:<w$}", s, w = n));

    None
}

fn glob_to_regex(pat: &str) -> String {
    let mut out = String::from("^");
    for c in pat.chars() {
        match c {
            '*' => out.push_str(".*"),
            '?' => out.push('.'),
            '.' | '+' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '$' | '|' | '\\' => { out.push('\\'); out.push(c); }
            _ => out.push(c),
        }
    }
    out.push('$');
    out
}

fn wildcard_match(s: &str, pat: &str) -> bool {
    let re = match regex::Regex::new(&glob_to_regex(pat)) { Ok(r) => r, Err(_) => return false };
    re.is_match(s)
}

fn lev_dist(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

fn longest_common_substring(a: &str, b: &str) -> String {
    let av: Vec<char> = a.chars().collect();
    let bv: Vec<char> = b.chars().collect();
    let mut best = 0usize;
    let mut end = 0usize;
    let mut dp = vec![vec![0usize; bv.len() + 1]; av.len() + 1];
    for i in 1..=av.len() {
        for j in 1..=bv.len() {
            if av[i - 1] == bv[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
                if dp[i][j] > best { best = dp[i][j]; end = i; }
            }
        }
    }
    av[end - best..end].iter().collect()
}
