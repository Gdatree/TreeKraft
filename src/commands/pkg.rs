use crate::main::vm::{VM, Value};
use std::path::PathBuf;

fn pkg_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".treekraft").join("pkgs")
}

fn registry_path() -> PathBuf {
    pkg_dir().join("registry.json")
}

fn valid_pkg_name(s: &str) -> bool {
    !s.is_empty() && !s.contains('/') && !s.contains('\\') && !s.contains(':')
        && !s.contains("..")
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // ~trek_pkg install("name") / install(name, path)
    if code.starts_with("~trek_pkg install(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[18..code.len()-1], ',');
        if p.len() >= 1 {
            let name = VM::tq(&p[0]);
            if !valid_pkg_name(&name) {
                return Some(Err(format!("Invalid package name: '{}'", name)));
            }
            if std::fs::create_dir_all(pkg_dir()).is_err() {
                return Some(Err("Cannot create package dir".to_string()));
            }
            // ищем файл в локальном реестре или по пути
            let candidates = [
                PathBuf::from(format!("{}.tk", name)),
                pkg_dir().join(format!("{}.tk", name)),
                PathBuf::from(&name),
            ];
            let mut src = None;
            for c in candidates.iter() {
                if c.exists() { src = Some(c.clone()); break; }
            }
            let src = match src {
                Some(s) => s,
                None => return Some(Err(format!("Package '{}' not found (no file {}.tk)", name, name))),
            };
            let content = match std::fs::read_to_string(&src) { Ok(c) => c, Err(e) => return Some(Err(e.to_string())) };
            let dest = pkg_dir().join(format!("{}.tk", name));
            if std::fs::write(&dest, &content).is_err() {
                return Some(Err("Failed to write package".to_string()));
            }
            // обновляем registry.json
            let mut reg = read_registry();
            reg.insert(name.clone(), dest.to_string_lossy().to_string());
            write_registry(&reg);
            vm.echo(format!("Package '{}' installed", name));
            return Some(Ok(Value::Bool(true)));
        }
        return None;
    }
    if code == "~trek_pkg list" || code == "~trek_pkg list()" || code.starts_with("~trek_pkg list(") {
        let reg = read_registry();
        if reg.is_empty() { vm.echo("  (no packages installed)"); }
        for (k, v) in &reg { vm.echo(format!("  {} -> {}", k, v)); }
        let list: Vec<Value> = reg.keys().map(|k| Value::Text(k.clone())).collect();
        return Some(Ok(Value::List(list)));
    }
    if code.starts_with("~trek_pkg remove(") && code.ends_with(')') {
        let name = VM::tq(&code[17..code.len()-1]);
        let mut reg = read_registry();
        if let Some(path) = reg.remove(&name) {
            let _ = std::fs::remove_file(&path);
            write_registry(&reg);
            vm.echo(format!("Package '{}' removed", name));
            return Some(Ok(Value::Bool(true)));
        }
        return Some(Err(format!("Package '{}' not installed", name)));
    }
    if code.starts_with("~trek_pkg search(") && code.ends_with(')') {
        let kw = VM::tq(&code[17..code.len()-1]).to_lowercase();
        let reg = read_registry();
        let mut found = 0;
        for (k, _) in &reg {
            if k.to_lowercase().contains(&kw) { vm.echo(format!("  {}", k)); found += 1; }
        }
        if found == 0 { vm.echo("  (nothing found)"); }
        return Some(Ok(Value::None));
    }
    if code.starts_with("~trek_pkg path(") && code.ends_with(')') {
        let name = VM::tq(&code[15..code.len()-1]);
        let reg = read_registry();
        if let Some(path) = reg.get(&name) {
            vm.echo(&format!("{}", path));
            return Some(Ok(Value::Text(path.clone())));
        }
        return Some(Err(format!("Package '{}' not installed", name)));
    }
    if code.starts_with("~import_pkg(") && code.ends_with(')') {
        let name = VM::tq(&code[12..code.len()-1]);
        let reg = read_registry();
        if let Some(path) = reg.get(&name) {
            // читаем напрямую (как ~import): путь с бэкслэшами НЕ переспариваем
            // через строковый литерал; ошибки импорта не глотаем (иначе
            // dispatcher решит «не совпадение» → Unknown command).
            match std::fs::read_to_string(&path) {
                Ok(c) => return Some(vm.run_script(&c)),
                Err(e) => return Some(Err(format!("import '{}': {}", path, e))),
            }
        }
        return Some(Err(format!("Package '{}' not installed", name)));
    }
    None
}

fn read_registry() -> std::collections::HashMap<String, String> {
    let path = registry_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(crate::main::vm::Value::Dict(d)) = crate::main::vm::json_parse(&data) {
            let mut out = std::collections::HashMap::new();
            for (k, v) in d {
                // новый формат (tkp): {"name": {"nick":..,"path":..,"tag":..}}; старый — строка-путь
                let p = match v {
                    crate::main::vm::Value::Text(t) => t,
                    crate::main::vm::Value::Dict(o) => o.get("path").map(|x| x.display()).unwrap_or_default(),
                    _ => continue,
                };
                if !p.is_empty() { out.insert(k, p); }
            }
            return out;
        }
    }
    std::collections::HashMap::new()
}

fn write_registry(reg: &std::collections::HashMap<String, String>) {
    let pairs: std::collections::HashMap<String, crate::main::vm::Value> = reg.iter().map(|(k, v)| (k.clone(), crate::main::vm::Value::Text(v.clone()))).collect();
    let json = crate::main::vm::simple_json_stringify(&crate::main::vm::Value::Dict(pairs));
    let _ = std::fs::write(registry_path(), json);
}
