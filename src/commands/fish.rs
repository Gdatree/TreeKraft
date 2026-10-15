use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! f1 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let n=match vm.eval_expr(&code[$p.len()..code.len()-1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(n);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));} }; }
    f1!("~fish_age(", 9, |l: f64| l * 3.5);
    f1!("~fish_weight(", 12, |l: f64| 0.00001 * l.powi(3));
    f1!("~fish_condition(", 15, |k: f64| k);
    f1!("~catch_release(", 14, |t: f64| t);
    f1!("~tide_height(", 12, |h: f64| h);
    f1!("~tide_range(", 11, |r: f64| r);
    f1!("~moon_phase(", 11, |d: f64| (d % 29.53) / 29.53);
    f1!("~solunar_rating(", 15, |r: f64| r);
    f1!("~water_temp(", 11, |t: f64| t);
    f1!("~dissolved_oxygen(", 17, |t: f64| 14.6 - 0.4*t + 0.008*t*t);
    f1!("~water_clarity(", 14, |c: f64| c);
    f1!("~casting_distance(", 17, |p: f64| p * 5.0);
    f1!("~line_strength(", 14, |d: f64| d);
    f1!("~hook_size(", 9, |s: f64| s);
    f1!("~bait_depth(", 11, |d: f64| d - 0.5);
    f1!("~trolling_speed(", 15, |s: f64| s);
    f1!("~depth_finder(", 13, |d: f64| d);
    f1!("~sonar_reading(", 14, |r: f64| r);
    f1!("~water_current(", 14, |c: f64| c);
    f1!("~wave_height(", 12, |h: f64| h);
    None
}
