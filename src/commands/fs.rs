use crate::main::vm::{VM, Value};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~read_file(") && code.ends_with(')') {
        let path = VM::tq(&code[11..code.len()-1]);
        let content = vm.interp(&path);
        match fs::read_to_string(&content) { Ok(s) => { vm.echo(&s); Some(Ok(Value::Text(s))) } Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~write_file(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() == 2 {
            let path = vm.interp(&VM::tq(&p[0]));
            let content = vm.eval_value(&p[1]).ok()?.display();
            match fs::write(&path, &content) { Ok(_) => Some(Ok(Value::None)), Err(e) => Some(Err(e.to_string())) }
        } else { None }
    } else if code.starts_with("~append_file(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() == 2 {
            let path = vm.interp(&VM::tq(&p[0]));
            let content = vm.eval_value(&p[1]).ok()?.display();
            match fs::OpenOptions::new().append(true).create(true).open(&path) {
                Ok(mut f) => { let _ = writeln!(f, "{}", content); Some(Ok(Value::None)) }
                Err(e) => Some(Err(e.to_string()))
            }
        } else { None }
    } else if code.starts_with("~file_exists(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[13..code.len()-1]));
        let ok = Path::new(&path).exists(); vm.echo(ok); Some(Ok(Value::Bool(ok)))
    } else if code.starts_with("~is_file(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[9..code.len()-1]));
        let ok = Path::new(&path).is_file(); vm.echo(ok); Some(Ok(Value::Bool(ok)))
    } else if code.starts_with("~is_dir(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[8..code.len()-1]));
        let ok = Path::new(&path).is_dir(); vm.echo(ok); Some(Ok(Value::Bool(ok)))
    } else if code.starts_with("~file_size(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[11..code.len()-1]));
        match fs::metadata(&path) { Ok(m) => { let s = m.len() as f64; vm.echo(s); Some(Ok(Value::Number(s))) } Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~file_modified(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[15..code.len()-1]));
        match fs::metadata(&path) {
            Ok(m) => match m.modified() {
                Ok(t) => { let s = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64(); vm.echo(s); Some(Ok(Value::Number(s))) }
                Err(_) => Some(Ok(Value::Number(0.0)))
            },
            Err(e) => Some(Err(e.to_string()))
        }
    } else if code.starts_with("~delete_file(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[13..code.len()-1]));
        match fs::remove_file(&path) { Ok(_) => Some(Ok(Value::Bool(true))), Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~create_dir(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[12..code.len()-1]));
        match fs::create_dir_all(&path) { Ok(_) => Some(Ok(Value::None)), Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~delete_dir(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[12..code.len()-1]));
        match fs::remove_dir_all(&path) { Ok(_) => Some(Ok(Value::Bool(true))), Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~list_dir(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[10..code.len()-1]));
        match fs::read_dir(&path) {
            Ok(entries) => {
                let mut files = Vec::new();
                for e in entries.flatten() {
                    files.push(Value::Text(e.file_name().to_string_lossy().to_string()));
                }
                Some(Ok(Value::List(files)))
            }
            Err(e) => Some(Err(e.to_string()))
        }
    } else if code.starts_with("~copy_file(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() == 2 {
            let src = vm.interp(&VM::tq(&p[0]));
            let dst = vm.interp(&VM::tq(&p[1]));
            match fs::copy(&src, &dst) { Ok(_) => Some(Ok(Value::None)), Err(e) => Some(Err(e.to_string())) }
        } else { None }
    } else if code.starts_with("~move_file(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() == 2 {
            let src = vm.interp(&VM::tq(&p[0]));
            let dst = vm.interp(&VM::tq(&p[1]));
            match fs::rename(&src, &dst) { Ok(_) => Some(Ok(Value::None)), Err(e) => Some(Err(e.to_string())) }
        } else { None }
    } else if code.starts_with("~file_ext(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[10..code.len()-1]));
        let ext = Path::new(&path).extension().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        vm.echo(&ext); Some(Ok(Value::Text(ext)))
    } else if code.starts_with("~file_stem(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[11..code.len()-1]));
        let stem = Path::new(&path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        vm.echo(&stem); Some(Ok(Value::Text(stem)))
    } else if code.starts_with("~file_name(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[11..code.len()-1]));
        let name = Path::new(&path).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        vm.echo(&name); Some(Ok(Value::Text(name)))
    } else if code.starts_with("~expand_user(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[13..code.len()-1]));
        let expanded = if path.starts_with('~') {
            dirs_fallback() + &path[1..]
        } else { path };
        vm.echo(&expanded); Some(Ok(Value::Text(expanded)))
    } else if code.starts_with("~abs_path(") && code.ends_with(')') {
        let path = vm.interp(&VM::tq(&code[10..code.len()-1]));
        if let Ok(p) = std::env::current_dir().map(|d| d.join(&path)) {
            let s = p.to_string_lossy().to_string(); vm.echo(&s); Some(Ok(Value::Text(s)))
        } else { Some(Ok(Value::Text(path))) }
    } else {
        None
    }
}

fn dirs_fallback() -> String {
    std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_default()
}
