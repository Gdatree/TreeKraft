use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! a1 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let n=match vm.eval_expr(&code[$p.len()..code.len()-1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(n);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));} }; }
    a1!("~acre_to_sqm(", 12, |ac: f64| ac * 4046.86);
    a1!("~sqm_to_acre(", 12, |sqm: f64| sqm / 4046.86);
    a1!("~hectare_to_acre(", 16, |ha: f64| ha * 2.47105);
    a1!("~acre_to_hectare(", 16, |ac: f64| ac / 2.47105);
    // Урожайность считают как количество урожая поделенное на площадь, поэтому ниже просим оба числа и отклоняем нулевую площадь чтобы не делить на ноль
    if code.starts_with("~yield_per_acre(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[16..code.len()-1], ',');
        if p.len() >= 2 {
            let a = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let b = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if b == 0.0 { return Some(Err("yield_per_acre: area is 0".to_string())); }
            let r = a / b;
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return Some(Err("yield_per_acre needs production, area".to_string()));
    }
    if code.starts_with("~yield_per_ha(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len()-1], ',');
        if p.len() >= 2 {
            let a = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let b = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if b == 0.0 { return Some(Err("yield_per_ha: area is 0".to_string())); }
            let r = a / b;
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return Some(Err("yield_per_ha needs production, area_ha".to_string()));
    }
    if code.starts_with("~seed_rate(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 2 {
            let a = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let b = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if b == 0.0 { return Some(Err("seed_rate: area is 0".to_string())); }
            let r = a / b;
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return Some(Err("seed_rate needs seed_amount, area".to_string()));
    }
    if code.starts_with("~fertilizer_rate(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len()-1], ',');
        if p.len() >= 2 {
            let a = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let b = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if b == 0.0 { return Some(Err("fertilizer_rate: area is 0".to_string())); }
            let r = a / b;
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return Some(Err("fertilizer_rate needs amount, area".to_string()));
    }
    if code.starts_with("~population(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() >= 2 {
            let a = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let b = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if b == 0.0 { return Some(Err("population: area is 0".to_string())); }
            let r = a / b;
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return Some(Err("population needs plant_count, area".to_string()));
    }
    if code.starts_with("~compost_ratio(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ',');
        if p.len() >= 2 {
            let a = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let b = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if b == 0.0 { return Some(Err("compost_ratio: browns is 0".to_string())); }
            let r = a / b;
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return Some(Err("compost_ratio needs greens, browns".to_string()));
    }
    a1!("~gdd(", 4, |t: f64| (t + (t-10.0))/2.0);
    a1!("~frost_days(", 11, |t: f64| if t < 0.0 { 1.0 } else { 0.0 });
    None
}
