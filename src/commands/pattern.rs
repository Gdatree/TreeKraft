use crate::main::vm::{VM, Value};

fn text(vm: &mut VM, s: &str) -> Option<String> {
    vm.eval_value(s).ok().and_then(|v| v.as_text())
}

fn number(vm: &mut VM, s: &str) -> Option<f64> {
    vm.eval_expr(s).ok()
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~pluralize(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 4 {
            let n = number(vm, &p[0])? as u64; let one = text(vm, &p[1])?; let few = text(vm, &p[2])?; let many = text(vm, &p[3])?;
            let r = if n % 10 == 1 && n % 100 != 11 { one } else if n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 10 || n % 100 >= 20) { few } else { many };
            let result = format!("{} {}", n, r);
            vm.echo(&format!("{}", result)); Some(Ok(Value::Text(result)))
        } else { None }
    } else if code.starts_with("~slugify(") && code.ends_with(')') {
        let s = text(vm, &code[9..code.len()-1])?;
        let r: String = s.to_lowercase().chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' }).collect::<String>().trim_matches('-').to_string();
        vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~truncate(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 2 { let s = text(vm, &p[0])?; let n = number(vm, &p[1])? as usize; let r: String = s.chars().take(n).collect(); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r))) } else { None }
    } else if code.starts_with("~ellipsis(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 2 { let s = text(vm, &p[0])?; let n = number(vm, &p[1])? as usize; let r = if s.chars().count() > n { s.chars().take(n-3).collect::<String>() + "..." } else { s }; vm.echo(format!("{}", r)); Some(Ok(Value::Text(r))) } else { None }
    } else if code.starts_with("~escape_shell(") && code.ends_with(')') {
        let s = text(vm, &code[14..code.len()-1])?; let r = format!("\"{}\"", s.replace('"', "\\\"").replace('$', "\\$").replace('`', "\\`")); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~wrap_text(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ','); if p.len() >= 2 {
            let s = text(vm, &p[0])?; let w = number(vm, &p[1])? as usize; let mut r = String::new(); let mut line = String::new();
            for word in s.split_whitespace() { if line.len() + word.len() + 1 > w { r.push_str(&line); r.push('\n'); line.clear(); } if !line.is_empty() { line.push(' '); } line.push_str(word); }
            r.push_str(&line); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
        } else { None }
    } else if code.starts_with("~indent(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ','); if p.len() >= 2 {
            let s = text(vm, &p[0])?; let n = number(vm, &p[1])? as usize; let r: String = s.lines().map(|l| format!("{}{}", " ".repeat(n), l)).collect::<Vec<_>>().join("\n");
            vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
        } else { None }
    } else if code.starts_with("~dedent(") && code.ends_with(')') {
        let s = text(vm, &code[8..code.len()-1])?;
        let min_indent = s.lines().filter(|l| !l.trim().is_empty()).map(|l| l.len() - l.trim_start().len()).min().unwrap_or(0);
        let r: String = s.lines().map(|l| if l.len() > min_indent { &l[min_indent..] } else { l.trim_start() }).collect::<Vec<_>>().join("\n");
        vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
    } else {
        None
    }
}
