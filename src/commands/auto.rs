use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if VM::matches_cmd_prefix(code, "send(") {
        let a = VM::extract_cmd_args(code, "send(")?; let s = vm.eval_value(&a).ok()?.as_text()?;
        println!("[Auto] Send: {}", s); Some(Ok(Value::None))
    } else if VM::matches_cmd_prefix(code, "click(") {
        let a = VM::extract_cmd_args(code, "click(")?; let p = VM::split_top_level(&a, ',');
        if p.len() >= 2 { let x = vm.eval_value(&p[0]).ok()?.as_number()?; let y = vm.eval_value(&p[1]).ok()?.as_number()?; println!("[Auto] Click: ({},{})", x, y); Some(Ok(Value::None)) } else { None }
    } else if VM::matches_cmd_prefix(code, "hotkey(") {
        let a = VM::extract_cmd_args(code, "hotkey(")?; let p = VM::split_top_level(&a, ',');
        if p.len() >= 2 { let key = vm.eval_value(&p[0]).ok()?.as_text()?; let cmd = vm.eval_value(&p[1]).ok()?.as_text()?; println!("[Auto] Hotkey '{}' -> {}", key, cmd); Some(Ok(Value::None)) } else { None }
    } else if code.starts_with("~key_press(") && code.ends_with(')') {
        let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; println!("[Auto] Key press: {}", s); Some(Ok(Value::None))
    } else if code.starts_with("~key_hold(") && code.ends_with(')') {
        let s = vm.eval_value(&code[10..code.len()-1]).ok()?.as_text()?; println!("[Auto] Key hold: {}", s); Some(Ok(Value::None))
    } else if code.starts_with("~key_release(") && code.ends_with(')') {
        let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?; println!("[Auto] Key release: {}", s); Some(Ok(Value::None))
    } else if code.starts_with("~mouse_move(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 2 { let x = vm.eval_expr(&p[0]).ok()?; let y = vm.eval_expr(&p[1]).ok()?; println!("[Auto] Mouse move: ({},{})", x, y); Some(Ok(Value::None)) } else { None }
    } else if code.starts_with("~type_text(") && code.ends_with(')') {
        let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; println!("[Auto] Type: {}", s); Some(Ok(Value::None))
    } else if code.starts_with("~scroll(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[8..code.len()-1]).ok()?; println!("[Auto] Scroll: {}", n); Some(Ok(Value::None))
    } else {
        None
    }
}
