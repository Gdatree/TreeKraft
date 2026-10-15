use crate::main::vm::{VM, Value};
use std::thread;
use std::time::Duration;

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~audio_beep(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 2 {
            let freq = vm.eval_expr(&p[0]).ok()? as u32; let dur = vm.eval_expr(&p[1]).ok()? as u32;
            crate::main::platform::win_sound::beep(freq, dur);
            Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~audio_play(") && code.ends_with(')') {
        let s = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?;
        match crate::main::platform::win_sound::play_wav(&s) {
            Ok(()) => { vm.echo(&format!("playing {}", s)); Some(Ok(Value::None)) }
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~audio_volume(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[14..code.len()-1]).ok()? as u8;
        vm.sound_volume = n; vm.echo(n); Some(Ok(Value::Number(n as f64)))
    } else if code.starts_with("~audio_stop") {
        crate::main::platform::win_sound::stop_wav();
        vm.echo("stopped"); Some(Ok(Value::None))
    } else {
        None
    }
}
