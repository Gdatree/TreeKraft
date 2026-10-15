use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! k1 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let n=match vm.eval_expr(&code[$p.len()..code.len()-1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(n);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));} }; }
    macro_rules! k2 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let p=VM::split_top_level(&code[$p.len()..code.len()-1],',');if p.len()>=2{let a=match vm.eval_expr(&p[0]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let b=match vm.eval_expr(&p[1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(a,b);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));}}}; }

    k2!("~baking_temp(", 12, |c: f64, _: f64| c * 9.0/5.0 + 32.0);
    k1!("~cooking_time(", 13, |w: f64| w * 45.0 / 1000.0);
    k1!("~roast_time(", 11, |w: f64| w * 30.0 / 1000.0 + 20.0);
    k2!("~turkey_time(", 12, |w: f64, stuffed: f64| if stuffed > 0.0 { w * 13.0 + 15.0 } else { w * 15.0 });
    k2!("~servings_scale(", 15, |orig: f64, new: f64| new / orig);
    k2!("~alcohol_volume(", 14, |v: f64, pct: f64| v * pct / 100.0);
    k2!("~proof_to_pct(", 13, |p: f64, _: f64| p / 2.0);
    k1!("~brew_gravity(", 13, |sg: f64| (sg - 1.0) * 1000.0);
    k1!("~abv(", 4, |og: f64| (og - 1.0) * 131.25);
    k2!("~ibu(", 4, |h: f64, a: f64| h * a * 10.0 / 20.0);
    k2!("~srm_to_ebc(", 11, |srm: f64, _: f64| srm * 1.97);
    k2!("~ebc_to_srm(", 11, |ebc: f64, _: f64| ebc / 1.97);
    k2!("~dilution(", 9, |c1: f64, c2: f64| c1 * 1.0 / c2);
    k2!("~concentration(", 14, |c: f64, d: f64| c / d);
    k1!("~yeast_activation(", 17, |t: f64| (t - 30.0).abs());
    // Гидратацию считаем как долю воды в муке в процентах поэтому просим оба веса и отклоняем нулевую муку чтобы не делить на ноль
    if code.starts_with("~dough_hydration(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len()-1], ',');
        if p.len() >= 2 {
            let w = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let f = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if f == 0.0 { return Some(Err("dough_hydration: flour is 0".to_string())); }
            let r = w / f * 100.0;
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return Some(Err("dough_hydration needs water, flour".to_string()));
    }
    k2!("~pressure_cooker(", 16, |t: f64, p: f64| t * 0.75);

    None
}
