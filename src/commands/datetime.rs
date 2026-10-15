use std::time::{SystemTime, UNIX_EPOCH};
use crate::main::vm::{VM, Value};

/// День недели по дате ISO "YYYY-MM-DD" (пустая → сегодня).
/// Возвращает английское имя; при непарсируемой дате — None.
fn weekday_of(arg: &str) -> Option<String> {
    const NAMES: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
    let days: i64 = if arg.trim().is_empty() {
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        secs.div_euclid(86400)
    } else {
        let p: Vec<&str> = arg.trim().split('-').collect();
        if p.len() != 3 { return None; }
        let (y, m, d): (i64, i64, i64) = (p[0].parse().ok()?, p[1].parse().ok()?, p[2].parse().ok()?);
        days_from_civil(y, m, d)?
    };
    // 1970-01-01 — четверг; индекс 0=Sunday.
    let w = (days.rem_euclid(7) + 4).rem_euclid(7) as usize;
    Some(NAMES[w].to_string())
}

/// Текущий год по системным часам (Hinnant civil_from_days).
fn current_year() -> i64 {
    let days = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64 / 86400;
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 }.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = mp + if mp < 10 { 3 } else { -9 };
    if m <= 2 { y += 1; }
    y
}

/// Дней от 1970-01-01 (алгоритм Hinnant); None при невалидном месяце/дне.
fn days_from_civil(y: i64, m: i64, d: i64) -> Option<i64> {    if !(1..=12).contains(&m) || !(1..=31).contains(&d) { return None; }
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 }.div_euclid(400);
    let yoe = (y - era * 400) as u64;
    let mp = (m + if m > 2 { -3 } else { 9 }) as u64;
    let doy = (153 * mp + 2) / 5 + d as u64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe as i64 - 719468)
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code == "~now()" || code == "~now" {
        let s = format!("{}", chrono_now()); vm.echo(&s); return Some(Ok(Value::Text(s)));
    }
    if code == "~today()" || code == "~today" {
        let s = today_iso(); vm.echo(&s); return Some(Ok(Value::Text(s)));
    }
    if code == "~time()" || code == "~time" {
        let s = time_iso(); vm.echo(&s); return Some(Ok(Value::Text(s)));
    }
    if code == "~datetime()" || code == "~datetime" {
        let s = format!("{}T{}", today_iso(), time_iso()); vm.echo(&s); return Some(Ok(Value::Text(s)));
    }
    if code == "~nowms()" || code == "~nowms" {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as f64;
        vm.echo(ts); return Some(Ok(Value::Number(ts)));
    }
    if code.starts_with("~format_date(") && code.ends_with(')') {
        let fmt = VM::tq(&code[13..code.len()-1]);
        let s = format_date(&fmt); vm.echo(&s); return Some(Ok(Value::Text(s)));
    }
    if code.starts_with("~from_timestamp(") && code.ends_with(')') {
        let ts = vm.eval_value(&code[16..code.len()-1]).ok()?.as_number()? as u64;
        let secs = ts.min(4102444800);
        let dur = std::time::Duration::from_secs(secs);
        let dt = UNIX_EPOCH + dur;
        let s = format!("{:?}", dt); vm.echo(&s); return Some(Ok(Value::Text(s)));
    }
    if code.starts_with("~sleep(") && code.ends_with(')') {
        let ms = vm.eval_value(&code[7..code.len()-1]).ok()?.as_number().unwrap_or(1000.0) as u64;
        std::thread::sleep(std::time::Duration::from_millis(ms));
        return Some(Ok(Value::None));
    }
    if code.starts_with("~wait(") && code.ends_with(')') {
        let ms = vm.eval_value(&code[6..code.len()-1]).ok()?.as_number().unwrap_or(1000.0) as u64;
        std::thread::sleep(std::time::Duration::from_millis(ms));
        return Some(Ok(Value::None));
    }
    if code.starts_with("~timer_start(") && code.ends_with(')') {
        let name = VM::tq(&code[13..code.len()-1]); vm.timers.insert(name, std::time::Instant::now()); return Some(Ok(Value::None));
    }
    if code.starts_with("~timer_elapsed(") && code.ends_with(')') {
        let name = VM::tq(&code[15..code.len()-1]);
        if let Some(start) = vm.timers.get(&name) {
            // НЕ печатаем здесь (иначе ~print(timer_elapsed(a)) печатал бы дважды)
            let ms = start.elapsed().as_secs_f64() * 1000.0; return Some(Ok(Value::Number(ms)));
        } return Some(Err("Timer not found".to_string()));
    }
    if code.starts_with("~timer_reset(") && code.ends_with(')') {
        let name = VM::tq(&code[13..code.len()-1]); vm.timers.insert(name, std::time::Instant::now()); return Some(Ok(Value::None));
    }
    if code.starts_with("~day_of_week(") && code.ends_with(')') {
        // day_of_week("2026-09-26") → "Saturday"; пустой аргумент → сегодня.
        let arg = VM::tq(&code[13..code.len()-1]);
        let name = weekday_of(&arg).unwrap_or_else(|| "Monday".to_string());
        vm.echo(&name); return Some(Ok(Value::Text(name)));
    }
    if code.starts_with("~is_leap_year(") && code.ends_with(')') {
        // Пустой/нечисловой аргумент → текущий год (не захардкоженный 2024).
        let y = vm.eval_value(&code[14..code.len()-1]).ok()?.as_number().unwrap_or(current_year() as f64) as i64;
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        vm.echo(leap); return Some(Ok(Value::Bool(leap)));
    }
    if code.starts_with("~days_in_month(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ',');
        if p.len() == 2 {
            let y = vm.eval_value(&p[0]).ok()?.as_number()? as i64;
            let m = vm.eval_value(&p[1]).ok()?.as_number()? as usize;
            let days = match m { 1|3|5|7|8|10|12 => 31, 4|6|9|11 => 30, 2 => if (y%4==0&&y%100!=0)||y%400==0 {29} else {28}, _ => 0 };
            vm.echo(days); return Some(Ok(Value::Number(days as f64)));
        } return None;
    }
    None
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    let days = secs / 86400 + 719468;
    let era = days;
    let e = era / 146097;
    let d2 = era - e * 146097;
    let y = (d2 - d2/1460 + d2/36524 - d2/146096) / 365;
    let y2 = y + e * 400;
    let d3 = d2 - (y*365 + y/4 - y/100);
    let m = (d3*5 + 2) / 153;
    let d4 = d3 - (m*153 + 2)/5 + 1;
    let (y3, m2) = if m < 10 { (y2 + 1, m + 3) } else { (y2 + 2, m - 9) };
    let h = (secs % 86400) / 3600;
    let mi = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y3, m2, d4, h, mi, s)
}

fn today_iso() -> String {
    chrono_now()[..10].to_string()
}

fn time_iso() -> String {
    chrono_now()[11..].to_string()
}

fn format_date(fmt: &str) -> String {
    let now = chrono_now();
    let parts: Vec<&str> = now.split(&[' ', ':'][..]).collect();
    let date_parts: Vec<&str> = if parts.len() >= 1 { parts[0].split('-').collect() } else { vec![] };
    let time_parts: Vec<&str> = if parts.len() >= 2 { parts[1].split(':').collect() } else { vec![] };
    let y = date_parts.get(0).unwrap_or(&"2024");
    let m = date_parts.get(1).unwrap_or(&"01");
    let d = date_parts.get(2).unwrap_or(&"01");
    let h = time_parts.get(0).unwrap_or(&"00");
    let mi = time_parts.get(1).unwrap_or(&"00");
    let s = time_parts.get(2).unwrap_or(&"00");
    fmt.replace("%Y", y).replace("%m", m).replace("%d", d)
        .replace("%H", h).replace("%M", mi).replace("%S", s)
        .replace("%h", &m[..]).replace("%i", &mi[..])
}
