use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! v1 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let n=match vm.eval_expr(&code[$p.len()..code.len()-1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(n);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));} }; }
    v1!("~dog_age(", 8, |h: f64| 16.0 * h.ln() + 31.0);
    v1!("~cat_age(", 8, |h: f64| 15.0 + 9.0*(h-1.0)/4.0);
    v1!("~rabbit_age(", 11, |h: f64| 21.0 + 4.0*h/12.0);
    v1!("~horse_age(", 10, |h: f64| 3.0 + 3.0*h+3.0);
    v1!("~bird_age(", 9, |h: f64| 10.0 + 4.0*h/12.0);
    v1!("~pet_calories(", 13, |w: f64| 70.0 * w.powf(0.75));
    v1!("~pet_water(", 10, |w: f64| w * 0.05);
    v1!("~pet_food(", 9, |w: f64| w * 0.025);
    v1!("~pregnancy_dog(", 14, |_: f64| 63.0);
    v1!("~pregnancy_cat(", 14, |_: f64| 65.0);
    v1!("~pregnancy_rabbit(", 17, |_: f64| 31.0);
    v1!("~pregnancy_horse(", 17, |_: f64| 340.0);
    v1!("~pregnancy_cow(", 15, |_: f64| 283.0);
    v1!("~pregnancy_pig(", 14, |_: f64| 114.0);
    v1!("~pregnancy_sheep(", 16, |_: f64| 147.0);
    v1!("~heart_rate_dog(", 15, |_: f64| 100.0);
    v1!("~heart_rate_cat(", 15, |_: f64| 150.0);
    v1!("~heart_rate_horse(", 17, |_: f64| 35.0);
    None
}
