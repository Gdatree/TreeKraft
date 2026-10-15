use crate::main::vm::{VM, Value};
use regex::Regex;

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~regex_match(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let text = vm.eval_value(&p[0]).ok()?.as_text()?;
            let pat = vm.eval_value(&p[1]).ok()?.as_text()?;
            match Regex::new(&pat) {
                Ok(re) => { let r = re.is_match(&text); vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r))) }
                Err(e) => Some(Err(e.to_string())),
            }
        } else { None }
    } else if code.starts_with("~regex_find(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let text = vm.eval_value(&p[0]).ok()?.as_text()?;
            let pat = vm.eval_value(&p[1]).ok()?.as_text()?;
            match Regex::new(&pat) {
                Ok(re) => {
                    let r = re.find(&text).map(|m| m.as_str().to_string()).unwrap_or_default();
                    vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
                }
                Err(e) => Some(Err(e.to_string())),
            }
        } else { None }
    } else if code.starts_with("~regex_findall(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len() - 1], ',');
        if p.len() == 2 {
            let text = vm.eval_value(&p[0]).ok()?.as_text()?;
            let pat = vm.eval_value(&p[1]).ok()?.as_text()?;
            match Regex::new(&pat) {
                Ok(re) => {
                    let list: Vec<Value> = re.find_iter(&text).map(|m| Value::Text(m.as_str().to_string())).collect();
                    let r = Value::List(list); vm.echo(format!("{}", r.display())); Some(Ok(r))
                }
                Err(e) => Some(Err(e.to_string())),
            }
        } else { None }
    } else if code.starts_with("~regex_replace(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len() - 1], ',');
        if p.len() == 3 {
            let text = vm.eval_value(&p[0]).ok()?.as_text()?;
            let pat = vm.eval_value(&p[1]).ok()?.as_text()?;
            let rep = vm.eval_value(&p[2]).ok()?.as_text()?;
            match Regex::new(&pat) {
                Ok(re) => { let r = re.replace_all(&text, &rep).to_string(); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r))) }
                Err(e) => Some(Err(e.to_string())),
            }
        } else { None }
    } else if code.starts_with("~regex_split(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let text = vm.eval_value(&p[0]).ok()?.as_text()?;
            let pat = vm.eval_value(&p[1]).ok()?.as_text()?;
            match Regex::new(&pat) {
                Ok(re) => {
                    let list: Vec<Value> = re.split(&text).map(|s| Value::Text(s.to_string())).collect();
                    let r = Value::List(list); vm.echo(format!("{}", r.display())); Some(Ok(r))
                }
                Err(e) => Some(Err(e.to_string())),
            }
        } else { None }
    } else if code.starts_with("~regex_escape(") && code.ends_with(')') {
        let text = vm.eval_value(&code[14..code.len() - 1]).ok()?.as_text()?;
        let r = regex::escape(&text);
        vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~regex_count(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let text = vm.eval_value(&p[0]).ok()?.as_text()?;
            let pat = vm.eval_value(&p[1]).ok()?.as_text()?;
            match Regex::new(&pat) {
                Ok(re) => { let n = re.find_iter(&text).count() as f64; vm.echo(format!("{}", Value::fmt_num(n))); Some(Ok(Value::Number(n))) }
                Err(e) => Some(Err(e.to_string())),
            }
        } else { None }
    } else {
        None
    }
}
