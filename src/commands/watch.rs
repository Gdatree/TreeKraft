use crate::main::vm::{VM, Value};
use std::collections::HashMap;
use std::time::SystemTime;

thread_local! {
    static WATCHED: std::cell::RefCell<HashMap<String, u64>> = std::cell::RefCell::new(HashMap::new());
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~watch_file(") && code.ends_with(')') {
        let path = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?;
        match std::fs::metadata(&path) {
            Ok(m) => {
                if let Ok(t) = m.modified() {
                    let secs = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs();
                    WATCHED.with(|w| w.borrow_mut().insert(path.clone(), secs));
                    println!("Watching: {}", path); Some(Ok(Value::None))
                } else { Some(Err("No modified time".to_string())) }
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~watch_check(") && code.ends_with(')') || code == "~watch_check" {
        let changed = WATCHED.with(|w| {
            let watched = w.borrow();
            let mut any_changed = false;
            for (path, old_time) in watched.iter() {
                if let Ok(m) = std::fs::metadata(path) {
                    if let Ok(t) = m.modified() {
                        let secs = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs();
                        if secs != *old_time {
                            println!("Changed: {}", path);
                            any_changed = true;
                        }
                    }
                }
            }
            any_changed
        });
        vm.echo(format!("{}", changed)); Some(Ok(Value::Bool(changed)))
    } else if code.starts_with("~watch_stop(") && code.ends_with(')') || code == "~watch_stop" {
        WATCHED.with(|w| w.borrow_mut().clear());
        println!("Watch stopped"); Some(Ok(Value::None))
    } else if code.starts_with("~watch_list") {
        WATCHED.with(|w| { for (path, _) in w.borrow().iter() { println!("  {}", path); } });
        Some(Ok(Value::None))
    } else {
        None
    }
}
