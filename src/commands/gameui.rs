use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if VM::matches_cmd_prefix(code, "score_add(") {
        let a = VM::extract_cmd_args(code, "score_add(")?; let d = vm.eval_value(&a).ok()?.as_number()?;
        let s = vm.game_score.unwrap_or(0.0) + d; vm.game_score = Some(s); vm.echo(format!("{}", Value::fmt_num(s))); Some(Ok(Value::Number(s)))
    } else if VM::matches_cmd(code, "score") {
        let s = vm.game_score.unwrap_or(0.0); vm.echo(format!("{}", Value::fmt_num(s))); Some(Ok(Value::Number(s)))
    } else if code.starts_with("~menu(") && code.ends_with(')') {
        let items_str = vm.eval_value(&code[6..code.len()-1]).ok()?.as_text()?;
        vm.menu_items = items_str.split(',').map(|s| s.trim().to_string()).collect();
        vm.menu_selected = 0; vm.game_paused = true;
        println!("Menu: {}", items_str); Some(Ok(Value::None))
    } else if code.starts_with("~menu_select(") && code.ends_with(')') || code == "~menu_select" {
        vm.echo(&format!("{}", vm.menu_selected)); Some(Ok(Value::Number(vm.menu_selected as f64)))
    } else if code.starts_with("~hud(") && code.ends_with(')') {
        println!("[HUD] {}", &code[5..code.len()-1]); Some(Ok(Value::None))
    } else if code.starts_with("~pause") && (code.len() == 6 || (code.starts_with("~pause(") && code.ends_with(')'))) {
        vm.game_paused = true; println!("Game paused"); Some(Ok(Value::None))
    } else if code.starts_with("~resume") && (code.len() == 7 || (code.starts_with("~resume(") && code.ends_with(')'))) {
        vm.game_paused = false; println!("Game resumed"); Some(Ok(Value::None))
    } else if code.starts_with("~game_over(") && code.ends_with(')') {
        let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?; println!("{}", s); Some(Ok(Value::None))
    } else if code.starts_with("~show_score(") && code.ends_with(')') || code == "~show_score" {
        let s = vm.game_score.unwrap_or(0.0); println!("Score: {}", Value::fmt_num(s)); Some(Ok(Value::None))
    } else if code.starts_with("~lives(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[7..code.len()-1]).ok()? as i32; vm.lives = n as i64; vm.echo(format!("{}", n)); Some(Ok(Value::Number(n as f64)))
    } else if code.starts_with("~hp(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[4..code.len()-1]).ok()? as i64; vm.hp = n; vm.echo(format!("{}", n)); Some(Ok(Value::Number(n as f64)))
    } else if code.starts_with("~xp(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[4..code.len()-1]).ok()? as i64; vm.xp = n; vm.echo(format!("{}", n)); Some(Ok(Value::Number(n as f64)))
    } else if code.starts_with("~dialog(") && code.ends_with(')') {
        let s = vm.eval_value(&code[8..code.len()-1]).ok()?.as_text()?; println!("[Dialog] {}", s); Some(Ok(Value::None))
    } else if code.starts_with("~notify(") && code.ends_with(')') {
        let s = vm.eval_value(&code[8..code.len()-1]).ok()?.as_text()?;
        println!("\x1B[1;96m[!] {}\x1B[0m", s); Some(Ok(Value::None))
    } else if code.starts_with("~healthbar(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ','); if p.len() >= 2 {
            let cur = vm.eval_expr(&p[0]).ok()? as i64; let max = vm.eval_expr(&p[1]).ok()? as i64;
            let w = if p.len() >= 3 { vm.eval_expr(&p[2]).ok()? as usize } else { 20 };
            let pct = if max > 0 { cur * w as i64 / max } else { 0 };
            let bar = format!("{}{}", "█".repeat(pct as usize), "░".repeat(w.saturating_sub(pct as usize)));
            println!("{} {:>3}%", bar, cur * 100 / max.max(1));
            Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~counter(") && code.ends_with(')') {
        let name = code[9..code.len()-1].trim();
        let n = vm.get_var(name).and_then(|v| v.as_number()).unwrap_or(0.0) + 1.0;
        vm.set_var(name, Value::Number(n));
        vm.echo(format!("{}", Value::fmt_num(n))); Some(Ok(Value::Number(n)))
    } else {
        None
    }
}
