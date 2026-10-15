use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! s1 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let n=match vm.eval_expr(&code[$p.len()..code.len()-1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(n);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));} }; }
    macro_rules! s2 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let p=VM::split_top_level(&code[$p.len()..code.len()-1],',');if p.len()>=2{let a=match vm.eval_expr(&p[0]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let b=match vm.eval_expr(&p[1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(a,b);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));}}}; }

    s1!("~pace_speed(", 11, |pace: f64| 60.0 / pace);
    s1!("~speed_pace(", 11, |speed: f64| 60.0 / speed);
    s2!("~race_time(", 10, |dist: f64, pace: f64| dist * pace);
    s2!("~race_distance(", 14, |time: f64, pace: f64| time / pace);
    s2!("~calories_run(", 12, |w: f64, d: f64| 0.9 * w * d);
    s2!("~calories_walk(", 13, |w: f64, d: f64| 0.5 * w * d);
    s2!("~calories_cycle(", 14, |w: f64, d: f64| 0.6 * w * d);
    s2!("~calories_swim(", 13, |w: f64, d: f64| 1.2 * w * d);
    s2!("~bmi_from_wh(", 12, |w: f64, h: f64| w / (h/100.0).powi(2));
    s1!("~body_fat_wh(", 13, |bmi: f64| 1.2 * bmi + 0.23 * 30.0 - 16.2);
    s2!("~waist_hip(", 10, |w: f64, h: f64| w / h);
    s2!("~ffmi(", 5, |w: f64, bfp: f64| w * (1.0 - bfp/100.0) / (1.7_f64).powi(2));
    s2!("~one_rep_max(", 12, |w: f64, reps: f64| w * (1.0 + reps / 30.0));
    s2!("~epley(", 6, |w: f64, reps: f64| w * (1.0 + reps / 30.0));
    s2!("~brzycki(", 8, |w: f64, reps: f64| w * 36.0 / (37.0 - reps));
    s2!("~wilks(", 6, |w: f64, total: f64| total * 500.0 / (-0.000001093*w.powi(4) + 0.000739*w.powi(3) - 0.191* w.powi(2) + 24.4*w + 473.0));
    s2!("~dips_score(", 11, |w: f64, reps: f64| (w * 0.8) * (1.0 + reps / 30.0));
    s2!("~pullup_score(", 13, |w: f64, reps: f64| w * (1.0 + reps / 30.0));

    None
}
