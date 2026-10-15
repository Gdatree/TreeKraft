use crate::main::vm::{VM, Value};

/// Базовые частоты нот четвёртой октавы, чтобы ноты и мелодии звучали одинаково
fn note_base(name: &str) -> Option<u32> {
    match name {
        "C" => Some(262), "C#" => Some(277), "D" => Some(294), "D#" => Some(311),
        "E" => Some(330), "F" => Some(349), "F#" => Some(370), "G" => Some(392),
        "G#" => Some(415), "A" => Some(440), "A#" => Some(466), "B" => Some(494),
        _ => None,
    }
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if VM::matches_cmd(code, "beep") || VM::matches_cmd(code, "beep()") {
        crate::main::platform::win_sound::beep(800, 200);
        vm.echo("*beep*"); Some(Ok(Value::None))
    } else if VM::matches_cmd_prefix(code, "snd_beep(") {
        let a = VM::extract_cmd_args(code, "snd_beep(")?; let p = VM::split_top_level(&a, ',');
        if p.len() >= 2 { let f = vm.eval_value(&p[0]).ok()?.as_number()? as u32; let d = vm.eval_value(&p[1]).ok()?.as_number()? as u32;
            crate::main::platform::win_sound::beep(f, d);
            Some(Ok(Value::None)) } else { None }
    } else if VM::matches_cmd_prefix(code, "snd_volume(") {
        let a = VM::extract_cmd_args(code, "snd_volume(")?; let n = vm.eval_value(&a).ok()?.as_number()? as u8; vm.sound_volume = n; vm.echo(n); Some(Ok(Value::Number(n as f64)))
    } else if code.starts_with("~play_tone(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ','); if p.len() >= 2 { let f = vm.eval_expr(&p[0]).ok()? as u32; let d = vm.eval_expr(&p[1]).ok()? as u32;
            crate::main::platform::win_sound::beep(f, d);
            Some(Ok(Value::None)) } else { None }
    } else if code.starts_with("~play_note(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ','); if p.len() >= 2 {
            let note = vm.eval_value(&p[0]).ok()?.as_text()?;
            let octave = vm.eval_expr(&p[1]).ok()? as u32;
            let dur = if p.len() >= 3 { vm.eval_expr(&p[2]).ok()? as u32 } else { 200 };
            let base = note.trim().trim_end_matches(|c: char| c.is_ascii_digit());
            let freq = note_base(base).unwrap_or(440) * 2u32.pow(octave.saturating_sub(4));
            crate::main::platform::win_sound::beep(freq, dur);
            Some(Ok(Value::None)) } else { None }
    } else if code.starts_with("~play_melody(") && code.ends_with(')') {
        let s = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?;
        for part in s.split_whitespace() {
            let (note, dur) = if let Some(idx) = part.find(|c: char| !c.is_ascii_alphabetic()) { (&part[..idx], part[idx..].parse::<u32>().unwrap_or(200)) } else { (part, 200) };
            if let Some(freq) = note_base(note) { crate::main::platform::win_sound::beep(freq, dur); std::thread::sleep(std::time::Duration::from_millis(10)); }
        }
        Some(Ok(Value::None))
    } else if code.starts_with("~sound_play(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?;
        match crate::main::platform::win_sound::play_wav(&s) {
            Ok(()) => { vm.echo(&format!("playing {}", s)); Some(Ok(Value::None)) }
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~sound_stop(") && code.ends_with(')') || code == "~sound_stop" {
        crate::main::platform::win_sound::stop_wav();
        vm.echo("stopped"); Some(Ok(Value::None))
    } else if code.starts_with("~sound_volume(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[14..code.len()-1]).ok()? as u8; vm.sound_volume = n; vm.echo(n); Some(Ok(Value::Number(n as f64)))
    } else {
        None
    }
}
