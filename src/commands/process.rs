use crate::main::vm::{VM, Value};
use std::process::Command;

fn shell_cmd(cmd: &str) -> Command {
    #[cfg(windows)]
    { let mut c = Command::new("cmd"); c.args(["/C", cmd]); c }
    #[cfg(not(windows))]
    { let mut c = Command::new("sh"); c.arg("-c").arg(cmd); c }
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~exec_output(") && code.ends_with(')') {
        let cmd = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?;
        match shell_cmd(&cmd).output() {
            Ok(o) => { let r = String::from_utf8_lossy(&o.stdout).trim().to_string(); vm.echo(format!("{}", r)); Some(Ok(Value::Text(r))) }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~exec_status(") && code.ends_with(')') {
        let cmd = vm.eval_value(&code[13..code.len()-1]).ok()?.as_text()?;
        match shell_cmd(&cmd).status() {
            Ok(s) => { let r = s.code().unwrap_or(-1) as f64; vm.echo(format!("{}", Value::fmt_num(r))); Some(Ok(Value::Number(r))) }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~exec_bg(") && code.ends_with(')') {
        let cmd = vm.eval_value(&code[9..code.len()-1]).ok()?.as_text()?;
        match shell_cmd(&cmd).spawn() {
            Ok(mut child) => { let pid = child.id() as f64; vm.echo(format!("{}", Value::fmt_num(pid))); Some(Ok(Value::Number(pid))) }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~process_kill(") && code.ends_with(')') {
        let pid = vm.eval_expr(&code[14..code.len()-1]).ok()? as u32;
        #[cfg(windows)]
        let r = Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).status();
        #[cfg(not(windows))]
        let r = Command::new("kill").args(["-9", &pid.to_string()]).status();
        match r {
            Ok(_) => Some(Ok(Value::Bool(true))),
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~process_list") && (code.len() == 13 || (code.starts_with("~process_list(") && code.ends_with(')'))) {
        #[cfg(windows)]
        let out = Command::new("tasklist").args(["/FO", "CSV", "/NH"]).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string());
        #[cfg(not(windows))]
        let out = Command::new("ps").args(["-eo", "pid,comm"]).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string());
        match out {
            Ok(out) => {
                let procs: Vec<Value> = out.lines().filter_map(|l| {
                    #[cfg(windows)]
                    { let parts: Vec<&str> = l.split(',').collect(); if parts.len() >= 2 { Some(Value::Text(format!("{}: {}", parts[0].trim_matches('"'), parts[1].trim_matches('"')))) } else { None } }
                    #[cfg(not(windows))]
                    { let mut it = l.split_whitespace(); match (it.next(), it.next()) { (Some(p), Some(c)) => Some(Value::Text(format!("{}: {}", p, c))), _ => None } }
                }).collect();
                let r = Value::List(procs); vm.echo(format!("{}", r.display())); Some(Ok(r))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~process_exists(") && code.ends_with(')') {
        let pid = vm.eval_expr(&code[16..code.len()-1]).ok()? as u32;
        #[cfg(windows)]
        let r = Command::new("tasklist").args(["/FI", &format!("PID eq {}", pid), "/NH"]).output().map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()));
        #[cfg(not(windows))]
        let r = Command::new("ps").arg("-p").arg(pid.to_string()).args(["-o", "pid="]).output().map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty());
        match r {
            Ok(exists) => { vm.echo(format!("{}", exists)); Some(Ok(Value::Bool(exists))) }
            Err(_) => { vm.echo("false"); Some(Ok(Value::Bool(false))) }
        }
    } else if code.starts_with("~shell(") && code.ends_with(')') {
        let cmd = vm.eval_value(&code[7..code.len()-1]).ok()?.as_text()?;
        match shell_cmd(&cmd).status() {
            Ok(_) => Some(Ok(Value::None)),
            Err(e) => Some(Err(e.to_string())),
        }
    } else {
        None
    }
}
