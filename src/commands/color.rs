use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~clr_rgb(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ',');
        if p.len() == 3 {
            let r = vm.eval_value(&p[0]).ok()?.as_number()? as u8;
            let g = vm.eval_value(&p[1]).ok()?.as_number()? as u8;
            let b = vm.eval_value(&p[2]).ok()?.as_number()? as u8;
            let s = format!("{},{},{}", r, g, b); vm.echo(&s); return Some(Ok(Value::Text(s)));
        }
        return None;
    }
    if code.starts_with("~clr_hex(") && code.ends_with(')') {
        let hex = VM::tq(&code[9..code.len()-1]).trim_start_matches('#').to_string();
        if hex.len() == 6 {
            if let Ok(n) = u32::from_str_radix(&hex, 16) {
                let s = format!("{},{},{}", (n>>16)&0xFF, (n>>8)&0xFF, n&0xFF);
                vm.echo(&s); return Some(Ok(Value::Text(s)));
            }
        }
        return Some(Err("Bad hex color".to_string()));
    }
    if code.starts_with("~clr_to_hex(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() == 3 {
            let r = vm.eval_value(&p[0]).ok()?.as_number()? as u8;
            let g = vm.eval_value(&p[1]).ok()?.as_number()? as u8;
            let b = vm.eval_value(&p[2]).ok()?.as_number()? as u8;
            let s = format!("#{:02X}{:02X}{:02X}", r, g, b); vm.echo(&s); return Some(Ok(Value::Text(s)));
        }
        return None;
    }
    if code.starts_with("~clr_red(") && code.ends_with(')') {
        let s = vm.eval_value(&code[9..code.len()-1]).ok()?.as_text()?;
        if let Ok(n) = parse_rgb(&s) { vm.echo(n.0); return Some(Ok(Value::Number(n.0 as f64))); }
        return Some(Err("Bad color".to_string()));
    }
    if code.starts_with("~clr_green(") && code.ends_with(')') {
        let s = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?;
        if let Ok(n) = parse_rgb(&s) { vm.echo(n.1); return Some(Ok(Value::Number(n.1 as f64))); }
        return Some(Err("Bad color".to_string()));
    }
    if code.starts_with("~clr_blue(") && code.ends_with(')') {
        let s = vm.eval_value(&code[10..code.len()-1]).ok()?.as_text()?;
        if let Ok(n) = parse_rgb(&s) { vm.echo(n.2); return Some(Ok(Value::Number(n.2 as f64))); }
        return Some(Err("Bad color".to_string()));
    }
    if code.starts_with("~clr_brightness(") && code.ends_with(')') {
        let s = vm.eval_value(&code[16..code.len()-1]).ok()?.as_text()?;
        if let Ok(n) = parse_rgb(&s) {
            let b = (n.0 as f64 * 0.299 + n.1 as f64 * 0.587 + n.2 as f64 * 0.114) / 255.0;
            vm.echo(b); return Some(Ok(Value::Number(b)));
        }
        return Some(Err("Bad color".to_string()));
    }
    if code.starts_with("~clr_grayscale(") && code.ends_with(')') {
        let s = vm.eval_value(&code[15..code.len()-1]).ok()?.as_text()?;
        if let Ok(n) = parse_rgb(&s) {
            let g = (n.0 as f64 * 0.299 + n.1 as f64 * 0.587 + n.2 as f64 * 0.114) as u8;
            let r = format!("{},{},{}", g, g, g); vm.echo(&r); return Some(Ok(Value::Text(r)));
        }
        return Some(Err("Bad color".to_string()));
    }
    if code.starts_with("~clr_invert(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?;
        if let Ok(n) = parse_rgb(&s) {
            let r = format!("{},{},{}", 255-n.0, 255-n.1, 255-n.2);
            vm.echo(&r); return Some(Ok(Value::Text(r)));
        }
        return Some(Err("Bad color".to_string()));
    }
    if code.starts_with("~clr_blend(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() == 3 {
            let a = parse_rgb(&vm.eval_value(&p[0]).ok()?.as_text()?).ok()?;
            let b_text = vm.eval_value(&p[1]).ok()?.as_text()?;
            let b = parse_rgb(&b_text).ok()?;
            let t = vm.eval_value(&p[2]).ok()?.as_number()?.clamp(0.0, 1.0);
            let r = format!("{},{},{}",
                (a.0 as f64 + (b.0 as f64 - a.0 as f64) * t) as u8,
                (a.1 as f64 + (b.1 as f64 - a.1 as f64) * t) as u8,
                (a.2 as f64 + (b.2 as f64 - a.2 as f64) * t) as u8);
            vm.echo(&r); return Some(Ok(Value::Text(r)));
        }
        return None;
    }
    None
}

fn parse_rgb(s: &str) -> Result<(u8, u8, u8), ()> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix('#') {
        if h.len() == 6 { return u32::from_str_radix(h, 16).map(|n| ((n>>16) as u8, (n>>8) as u8, n as u8)).map_err(|_| ()); }
    }
    if s.contains(',') {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<u8>().map_err(|_| ())?;
            let g = parts[1].trim().parse::<u8>().map_err(|_| ())?;
            let b = parts[2].trim().parse::<u8>().map_err(|_| ())?;
            return Ok((r, g, b));
        }
    }
    Err(())
}
