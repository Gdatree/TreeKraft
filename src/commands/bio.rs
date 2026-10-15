use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! b1 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let n=match vm.eval_expr(&code[$p.len()..code.len()-1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(n);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));} }; }

    // Рост по умолчанию считаем 170 см чтобы формулу площади тела можно было вызвать только по весу
    if code.starts_with("~bsa_mosteller(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ',');
        let w = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
        let h = if p.len() >= 2 { match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 170.0 };
        let r = (h * w / 3600.0).sqrt();
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }
    if code.starts_with("~bsa_dubois(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        let w = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
        let h = if p.len() >= 2 { match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 170.0 };
        let r = 0.007184 * w.powf(0.425) * h.powf(0.725);
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }
    if code.starts_with("~bmi(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len()-1], ',');
        let w = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
        let h = if p.len() >= 2 { match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 1.7 };
        let r = w / h.powi(2);
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }
    b1!("~ibm_devine(", 11, |h: f64| 50.0 + 0.9 * (h - 152.0));
    b1!("~ibm_robinson(", 13, |h: f64| 52.0 + 1.9 * (h - 152.0));
    b1!("~ibm_miller(", 12, |h: f64| 56.2 + 1.41 * (h - 152.0));
    b1!("~ibm_hamwi(", 10, |h: f64| 48.0 + 2.7 * (h - 152.0));
    b1!("~water_intake(", 13, |w: f64| w * 0.033);
    // Рост и возраст по умолчанию считаем как 170 см и 30 лет чтобы расход калорий считался даже если введен только вес и чтобы не требовать лишние числа
    if code.starts_with("~calorie_resting(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len()-1], ',');
        let w = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
        let h = if p.len() >= 2 { match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 170.0 };
        let age = if p.len() >= 3 { match vm.eval_expr(&p[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 30.0 };
        let r = 10.0 * w + 6.25 * h - 5.0 * age + 5.0;
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }
    b1!("~max_hr(", 7, |age: f64| 220.0 - age);
    b1!("~target_hr(", 10, |age: f64| (220.0 - age) * 0.7);
    b1!("~vo2max(", 7, |age: f64| 56.5 - 0.4 * age);
    if code.starts_with("~body_fat(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        let bmi = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
        let age = if p.len() >= 2 { match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 30.0 };
        let r = 1.2 * bmi + 0.23 * age - 16.2;
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }
    b1!("~lean_mass(", 10, |w: f64| w * 0.8);
    b1!("~blood_volume(", 13, |w: f64| w * 0.07);
    b1!("~gfr(", 4, |age: f64| 175.0 * (0.8_f64).powf(-1.154) * age.powf(-0.203) * 0.742);
    if code.starts_with("~bmi_category(") && code.ends_with(')') { let bmi = match vm.eval_expr(&code[14..code.len()-1]) { Ok(v) => v, Err(e) => return Some(Err(e)) }; let r = if bmi < 18.5 { "underweight" } else if bmi < 25.0 { "normal" } else if bmi < 30.0 { "overweight" } else { "obese" }; vm.echo(r); return Some(Ok(Value::Text(r.to_string()))); }
    b1!("~pregnancy_week(", 15, |d: f64| (d / 7.0).floor());
    b1!("~due_date(", 9, |d: f64| d + 280.0);
    // Каждый из пяти признаков Апгар оцениваем от нуля до двух поэтому требуем ровно пять чисел и проверяем границы чтобы сумма отражала состояние а не случайный набор
    if code.starts_with("~apgar_score(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() != 5 {
            return Some(Err("apgar_score needs 5 components 0-2 (appearance, pulse, grimace, activity, respiration)".to_string()));
        }
        let mut sum = 0.0;
        for part in &p {
            let v = match vm.eval_expr(part) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if !(0.0..=2.0).contains(&v) {
                return Some(Err(format!("apgar component out of range 0-2: {}", v)));
            }
            sum += v;
        }
        vm.echo(Value::fmt_num(sum)); return Some(Ok(Value::Number(sum)));
    }

    None
}
