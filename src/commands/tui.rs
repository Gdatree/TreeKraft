use crate::main::vm::{VM, Value};
use crate::main::screen::Screen;

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~tui_init(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 2 {
            let w = vm.eval_value(&p[0]).ok()?.as_number()? as usize;
            let h = vm.eval_value(&p[1]).ok()?.as_number()? as usize;
            let border = if p.len() > 2 { VM::tq(&p[2]).chars().next().unwrap_or('#') } else { '#' };
            let fill = if p.len() > 3 { VM::tq(&p[3]).chars().next().unwrap_or(' ') } else { ' ' };
            vm.screen = Some(Screen::new(w, h, border, fill));
            vm.screen_char = '#';
            println!("TUI initialized: {}x{}", w, h); Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~tui_win(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ',');
        if p.len() >= 5 {
            let x = vm.eval_expr(&p[0]).ok()? as usize;
            let y = vm.eval_expr(&p[1]).ok()? as usize;
            let w = vm.eval_expr(&p[2]).ok()? as usize;
            let h = vm.eval_expr(&p[3]).ok()? as usize;
            let title = vm.eval_value(&p[4]).ok()?.as_text()?;
            if vm.screen.is_some() {
                let scr = vm.screen.as_mut()?;
                scr.draw_box(x, y, w, h, scr.border);
                if !title.is_empty() && title.len() + 4 < w { scr.draw_text(x+2, y, &format!(" {} ", title)); }
            }
            Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~tui_text(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 3 {
            let x = vm.eval_expr(&p[0]).ok()? as usize;
            let y = vm.eval_expr(&p[1]).ok()? as usize;
            let s = vm.eval_value(&p[2]).ok()?.as_text()?;
            let interp = vm.interp(&s);
            if vm.screen.is_some() { vm.screen.as_mut()?.draw_text(x, y, &interp); }
            Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~tui_btn(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ',');
        if p.len() >= 3 && vm.screen.is_some() {
            let x = vm.eval_expr(&p[0]).ok()? as usize;
            let y = vm.eval_expr(&p[1]).ok()? as usize;
            let label = vm.eval_value(&p[2]).ok()?.as_text()?;
            let scr = vm.screen.as_mut()?;
            let text = format!("[ {} ]", label);
            scr.draw_text(x, y, &text);
            Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~tui_input(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 3 && vm.screen.is_some() {
            let x = vm.eval_expr(&p[0]).ok()? as usize;
            let y = vm.eval_expr(&p[1]).ok()? as usize;
            let label = vm.eval_value(&p[2]).ok()?.as_text()?;
            let scr = vm.screen.as_mut()?;
            let text = format!("{}: _", label);
            scr.draw_text(x, y, &text);
            Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~tui_bar(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ',');
        if p.len() >= 4 && vm.screen.is_some() {
            let x = vm.eval_expr(&p[0]).ok()? as usize;
            let y = vm.eval_expr(&p[1]).ok()? as usize;
            let w = vm.eval_expr(&p[2]).ok()? as usize;
            let pct = vm.eval_expr(&p[3]).ok()?;
            let filled = ((pct / 100.0) * w as f64).round() as usize;
            let bar: String = format!("[{}{}]", "█".repeat(filled), "░".repeat(w.saturating_sub(filled)));
            vm.screen.as_mut()?.draw_text(x, y, &bar);
            Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~tui_show(") && code.ends_with(')') || code == "~tui_show" {
        if let Some(ref scr) = vm.screen { scr.render(); }
        Some(Ok(Value::None))
    } else if code.starts_with("~tui_clear(") && code.ends_with(')') || code == "~tui_clear" {
        if let Some(ref mut scr) = vm.screen { scr.clear(); }
        Some(Ok(Value::None))
    } else {
        None
    }
}
