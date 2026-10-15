use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! m1 {
        ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let n = match vm.eval_expr(&code[$p.len()..code.len()-1]) { Ok(v) => v, Err(e) => return Some(Err(e)) }; let r = ($f)(n); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } };
    }

    // Ноты переводим в герцы по номеру чтобы не считать формулу вручную
    m1!("~note_freq(", 10, |n: f64| 440.0 * (2.0f64).powf((n - 69.0) / 12.0));
    m1!("~midi_note(", 9, |f: f64| 69.0 + 12.0 * (f / 440.0).log2());
    m1!("~cents(", 6, |r: f64| 1200.0 * r.log2());

    // Интервалы считаем в полутонах и коэффициентах чтобы строить аккорды точнее
    m1!("~interval_semitones(", 19, |r: f64| 12.0 * r.log2());
    m1!("~interval_ratio(", 15, |s: f64| (2.0f64).powf(s / 12.0));

    // Темп переводим в миллисекунды и герцы чтобы звук и свет шли синхронно
    m1!("~bpm_to_ms(", 10, |bpm: f64| 60000.0 / bpm);
    m1!("~ms_to_bpm(", 10, |ms: f64| 60000.0 / ms);
    m1!("~bpm_to_hz(", 10, |bpm: f64| bpm / 60.0);
    m1!("~hz_to_bpm(", 10, |hz: f64| hz * 60.0);

    // Задержку эха и реверба считаем в долях бита чтобы эффект дышал вместе с треком
    if code.starts_with("~delay_time(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        let bpm = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
        if bpm == 0.0 { return Some(Err("delay_time: bpm is 0".to_string())); }
        let beats = if p.len() >= 2 { match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 1.0 };
        let r = 60000.0 / bpm * beats;
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }
    if code.starts_with("~reverb_time(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        let v = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
        let a = if p.len() >= 2 { match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 0.2 };
        if a <= 0.0 { return Some(Err("reverb_time: absorption must be > 0".to_string())); }
        let r = 0.161 * v / a;
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }
    m1!("~decay_time(", 11, |t: f64| t * 0.001);

    // Мягкий коэффициент фильтра считаем тут чтобы срезать резкие скачки звука
    if code.starts_with("~lowpass(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ',');
        let fc = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
        let sr = if p.len() >= 2 { match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 44100.0 };
        if fc <= 0.0 || sr <= 0.0 { return Some(Err("lowpass: fc and sr must be > 0".to_string())); }
        let rc = 1.0 / (2.0 * std::f64::consts::PI * fc);
        let r = (1.0 / sr) / (rc + 1.0 / sr);
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }
    if code.starts_with("~highpass(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        let fc = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
        let sr = if p.len() >= 2 { match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 44100.0 };
        if fc <= 0.0 || sr <= 0.0 { return Some(Err("highpass: fc and sr must be > 0".to_string())); }
        let rc = 1.0 / (2.0 * std::f64::consts::PI * fc);
        let r = rc / (rc + 1.0 / sr);
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }

    // Пила и шум нужны как кирпичики чтобы собирать синтез звука из простого
    m1!("~sawtooth(", 9, |t: f64| t % 1.0);
    m1!("~square_wave(", 12, |t: f64| if t % 1.0 < 0.5 { 1.0 } else { -1.0 });
    m1!("~triangle_wave(", 14, |t: f64| 2.0 * (2.0 * (t % 1.0) - 1.0).abs() - 1.0);
    m1!("~noise(", 6, |_: f64| rand::random::<f64>() * 2.0 - 1.0);

    None
}
