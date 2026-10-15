use crate::main::vm::{VM, Value};
use crate::commands::{self, Resolver};
use std::collections::HashMap;

/// Таблица псевдонимов, новое читаемое имя (без ~) > старое имя команды (без ~).
/// Позволяет писать `print(...)`, `list_push(...)` вместо `~print(...)`, `~add(...)`.
/// Заполняется поэтапно (сначала ядро, вывод/списки/словари/матем/строки/файлы/время).
/// Резолверы продолжают матчить старые имена, обратной совместимости нет, но ~ исчезает.
static COMMAND_ALIASES: &[(&str, &str)] = &[
    // ВЫВОД (output.rs)
    ("print", "print"), ("println", "println"), ("echo", "echo"), ("dump", "dump"),
    ("format", "format"),
    ("fg", "fg"), ("bg", "bg"), ("color", "color"), ("reset_color", "resetcolor"),
    ("bold", "bold"), ("italic", "italic"), ("underline", "underline"),
    ("strikethrough", "strikethrough"), ("blink", "blink"), ("hidden", "hidden"),
    ("rgb", "rgb"), ("rgb_bg", "rgb_bg"), ("gradient", "gradient"), ("rainbow", "rainbow"),
    ("cursor", "cur"), ("cls", "cls"),
    ("red", "red"), ("green", "green"), ("blue", "blue"), ("yellow", "yellow"),
    ("cyan", "cyan"), ("magenta", "magenta"),
    ("alert", "al"), ("typewriter", "cm"), ("printf", "printf"), ("pad", "pad"),
    ("progress_bar", "progressbar"), ("spinner", "spinner"), ("box", "box"),
    ("text", "tx"), ("text_red", "tx.red"), ("text_green", "tx.green"),
    ("text_blue", "tx.blue"), ("text_yellow", "tx.yellow"),
    // СПИСКИ (lists.rs)
    ("list", "list"), ("list_new", "list"),
    ("list_push", "add"), ("push", "add"), ("list_get", "get"), ("list_set", "set"),
    ("list_remove", "remove"), ("list_insert", "insert"), ("list_pop", "pop"),
    ("list_len", "len"), ("list_sort", "sort"), ("list_sort_desc", "sort_desc"),
    ("list_sort_by_len", "sort_by_len"), ("list_reverse", "rev"), ("list_contains", "contains"),
    ("list_index", "index"), ("list_slice", "slice"), ("list_join", "join"),
    ("list_map", "list_map"), ("list_filter", "list_filter"), ("list_reduce", "list_reduce"),
    ("list_foreach", "list_foreach"), ("list_sum", "list_sum"), ("list_min", "list_min"),
    ("list_max", "list_max"), ("list_avg", "list_avg"), ("list_flatten", "list_flatten"),
    ("list_unique", "list_unique"), ("list_count", "list_count"), ("list_zip", "list_zip"),
    ("list_rotate", "list_rotate"), ("list_chunk", "list_chunk"),
    ("list_shift", "list_shift"), ("list_unshift", "list_unshift"),
    ("list_remove_val", "list_removeval"), ("list_shuffle", "list_shuffle"),
    ("list_repeat", "list_repeat"), ("list_fill", "list_fill"), ("list_clear", "list_clear"),
    ("list_copy", "list_copy"), ("list_swap", "list_swap"), ("list_find", "list_find"),
    ("list_take", "list_take"), ("list_drop", "list_drop"), ("list_partition", "list_partition"),
    ("list_group", "list_group"), ("list_top", "list_top"), ("list_tail", "list_tail"),
    ("list_last", "list_last"), ("list_head", "list_head"),
    ("list_sort_by", "list_sort_by"),
    // СЛОВАРИ (lists.rs)
    ("dict", "dict"), ("dict_new", "dict"), ("dict_set", "dset"), ("dict_get", "dget"),
    ("dict_del", "ddel"), ("dict_has", "dhas"), ("dict_keys", "dkeys"), ("dict_values", "dvals"),
    ("dict_len", "dlen"), ("dict_items", "ditems"), ("dict_merge", "dmerge"),
    ("dict_clear", "dclear"), ("dict_copy", "dcopy"), ("dict_update", "dupdate"),
    ("dict_from_list", "dfromlist"), ("dict_get_or", "dgetor"), ("dict_pop", "dpop"),
    ("dict_default", "ddefault"),
    // МАТЕМ (math.rs)
    ("abs", "abs"), ("sqrt", "sqrt"), ("sin", "sin"), ("cos", "cos"), ("tan", "tan"),
    ("asin", "asin"), ("acos", "acos"), ("atan", "atan"), ("atan2", "atan2"),
    ("floor", "floor"), ("ceil", "ceil"), ("round", "round"), ("exp", "exp"), ("log", "log"),
    ("log10", "log10"), ("log2", "log2"), ("pow", "pow"), ("min", "min"), ("max", "max"),
    ("clamp", "clamp"), ("sign", "sign"), ("trunc", "trunc"), ("fract", "fract"),
    ("hypot", "hypot"), ("deg", "deg"), ("rad", "rad"), ("pi", "pi"), ("e", "e"),
    ("is_prime", "is_prime"), ("gcd", "gcd"), ("lcm", "lcm"), ("factorial", "factorial"),
    ("fib", "fibonacci"), ("sum", "sum"), ("mean", "mean"), ("median", "median"),
    ("mode", "mode"), ("randf", "randf"), ("randi", "rnd"), ("rand_range", "rnd"),
    ("sigma", "sigma"), ("divisors", "divisors"), ("is_even", "is_even"), ("is_odd", "is_odd"),
    ("gamma", "gamma_l"), ("erf", "erf_approx"), ("erf_inv", "erf_inv_approx"),
    ("bessel_j0", "bessel_j0"), ("bessel_j1", "bessel_j1"), ("bessel_y0", "bessel_y0"),
    ("bessel_y1", "bessel_y1"),
    // СТРОКИ (strings.rs)
    ("upper", "upper"), ("lower", "lower"), ("trim", "trim"), ("reverse", "rev"),
    ("repeat", "rep"), ("strlen", "len"), ("split", "split"), ("concat", "concat"),
    ("replace", "replace"), ("substr", "sub"), ("find", "find"), ("starts_with", "startswith"),
    ("ends_with", "endswith"), ("rnd_str", "rnds"), ("password", "passwd"),
    ("is_space", "isspace"), ("swap_case", "swapcase"), ("capitalize", "capitalize"),
    ("title_case", "titlecase"), ("snake_case", "snake"), ("camel_case", "camel"),
    ("kebab_case", "kebab"), ("word_count", "wordcount"), ("char_at", "charat"),
    ("ascii_code", "ascii"), ("from_ascii", "from_ascii"), ("ord", "ord"), ("chr", "chr"),
    ("levenshtein", "levenshtein"), ("hamming", "hamming"), ("jaccard", "jaccard"),
    ("wildcard_match", "wildcard_match"), ("glob_match", "glob_match"),
    ("longest_common_substr", "longest_common_substring"), ("deaccent", "deaccent"),
    ("escape_html", "escape_html"), ("unescape_html", "unescape_html"),
    ("url_encode", "url_encode"), ("url_decode", "url_decode"),
    // ВРЕМЯ/СИСТЕМА (time.rs, datetime.rs)
    ("now", "now"), ("now_ms", "nowms"), ("time", "time"), ("date", "date"),
    ("year", "year"), ("month", "month"), ("day", "day"), ("hour", "hour"),
    ("minute", "minute"), ("second", "second"), ("sleep_ms", "sleepms"), ("sleep", "wait"),
    ("timer_start", "timer_start"), ("timer_stop", "timer_stop"),
    ("timer_elapsed", "timer_elapsed"), ("timer_reset", "timer_reset"),
    ("format_date", "format_date"), ("from_timestamp", "from_timestamp"),
    ("day_of_week", "day_of_week"), ("is_leap_year", "is_leap_year"),
    ("days_in_month", "days_in_month"),
    ("os", "os"), ("arch", "arch"), ("hostname", "hostname"), ("username", "username"),
    ("pid", "pid"), ("cpu_count", "cpu_count"), ("cwd", "cwd"), ("homedir", "homedir"),
    ("tempdir", "tempdir"), ("mem", "mem"), ("mem_clear", "memclear"), ("sysinfo", "sysinfo"),
    // ФАЙЛЫ (time.rs fs.rs)
    ("file_read", "read"), ("file_write", "write"), ("file_append", "append"),
    ("file_read_lines", "readlines"), ("file_exists", "exs"), ("file_size", "fsize"),
    ("dir", "dir"), ("mkdir", "mkdir"), ("rmdir", "rmdir"), ("rmfile", "delfile"),
    ("cp", "cp"), ("mv", "mv"),
    ("save_state", "save"), ("load_state", "load"),
    ("env", "env"), ("shell", "shell"), ("exec", "exec"), ("exec_output", "execoutput"),
    ("beep", "beep"),
    // Удаленные bulk-дубли (u1, октябрь): старое имя с суффиксом 2 -> канон.
    // Формулы были побайтово одинаковы, оставлен один обработчик.
    ("hypotenuse2", "hypotenuse"), ("kite_area2", "kite_area"), ("rhombus_area2", "rhombus_area"),
    ("fl_oz_to_ml2", "fl_oz_to_ml"), ("ft3_to_l2", "ft3_to_l"), ("fts_to_ms2", "fts_to_ms"),
    ("gal_to_l2", "gal_to_l"), ("kg_to_t2", "kg_to_t"), ("kmh_to_kn2", "kmh_to_kn"),
    ("kn_to_kmh2", "kn_to_kmh"), ("l_to_gal2", "l_to_gal"), ("m3_to_l2", "m3_to_l"),
    ("m_to_ft2", "m_to_ft"), ("m_to_km2", "m_to_km"), ("m_to_mm2", "m_to_mm"),
    ("m_to_yd2", "m_to_yd"), ("mm_to_cm2", "mm_to_cm"), ("t_to_kg2", "t_to_kg"),
];

/// Переводит входящий код с новым именем в код со старым именем (для резолверов).
/// Если псевдонима нет, возвращает код как есть (резолверы сами решат).
pub fn translate_alias(code: &str) -> String {
    let trimmed = code.trim();
    // алиас применяем только к вызовам `name(...)`, чтобы не задеть
    // переменные/параметры, совпадающие с именами алиасов (например `deg`)
    // в выражении `deg * PI / 180` это переменная, а не команда ~deg.
    let name_end = trimmed.find('(').unwrap_or(trimmed.len());
    let name = &trimmed[..name_end];
    let rest = &trimmed[name_end..];
    for (new, old) in COMMAND_ALIASES {
        if *new == name {
            return format!("~{}{}", old, rest);
        }
    }
    code.to_string()
}

impl VM {
    // Пре-компиляция тел блоков, сразу в Vec<String> (без join/split на каждой итерации)
    pub fn collect_simple_lines(&self, lines: &[String], mut idx: usize) -> Result<(usize, Vec<String>), String> {
        let mut body = Vec::new();
        let mut depth = 0i32;
        while idx < lines.len() {
            let line = lines[idx].trim();
            if depth == 0 && line == "end" { idx += 1; break; }
            if Self::block_opener(line) { depth += 1; }
            if line == "end" { depth -= 1; }
            body.push(line.to_string());
            idx += 1;
        }
        Ok((idx, body))
    }

    /// Собирает ветки if/elif/else. Каждая ветка = (Option<cond>, lines)
    /// где None-cond = ветка else. head_cond, условие самой первой ветки if.
    pub fn collect_if_arms(&self, lines: &[String], mut idx: usize, head_cond: Option<String>) -> Result<(usize, Vec<(Option<String>, Vec<String>)>), String> {
        let mut arms: Vec<(Option<String>, Vec<String>)> = Vec::new();
        let mut cur_cond: Option<String> = head_cond;
        let mut body: Vec<String> = Vec::new();
        let mut depth = 0i32;
        let mut flush = |arms: &mut Vec<(Option<String>, Vec<String>)>, cond: &mut Option<String>, body: &mut Vec<String>| {
            arms.push((cond.take(), std::mem::take(body)));
        };
        while idx < lines.len() {
            let line = lines[idx].trim();
            if depth == 0 && line == "end" { idx += 1; break; }
            if depth == 0 {
                if let Some(c) = Self::elif_cond(line) {
                    flush(&mut arms, &mut cur_cond, &mut body);
                    cur_cond = Some(c);
                    idx += 1;
                    continue;
                }
                if line == "else" {
                    flush(&mut arms, &mut cur_cond, &mut body);
                    cur_cond = None;
                    idx += 1;
                    continue;
                }
            }
            if Self::block_opener(line) { depth += 1; }
            if line == "end" { depth -= 1; }
            body.push(line.to_string());
            idx += 1;
        }
        flush(&mut arms, &mut cur_cond, &mut body);
        Ok((idx, arms))
    }

    /// Читает условие из строки elif/else-if. Строка вида, `elif c then` | `else if c then`
    pub fn elif_cond(line: &str) -> Option<String> {
        let l = line.trim();
        let rest = if l.starts_with("elif ") { &l[5..] }
            else if l.starts_with("else if ") { &l[8..] }
            else if l.starts_with("~elif ") { &l[6..] }
            else { return None; };
        let c = rest.trim();
        let c = c.strip_suffix(" then").or_else(|| c.strip_suffix(" do")).unwrap_or(c).trim();
        if c.is_empty() { None } else { Some(c.to_string()) }
    }

    // Старая сигнатура, для совместимости (on_start)
    pub fn collect_if_lines(&self, lines: &[String], mut idx: usize) -> Result<(usize, Vec<String>, Vec<String>), String> {
        let (ni, arms) = self.collect_if_arms(lines, idx, None)?;
        idx = ni;
        let mut tb = Vec::new();
        let mut eb = Vec::new();
        if let Some((_, a)) = arms.first() { tb = a.clone(); }
        if let Some((None, a)) = arms.last() { eb = a.clone(); }
        Ok((idx, tb, eb))
    }

    pub fn collect_try_lines(&self, lines: &[String], mut idx: usize) -> Result<(usize, Vec<String>, Vec<String>, String), String> {
        let mut tb = Vec::new();
        let mut cb = Vec::new();
        let mut in_c = false;
        let mut depth = 0i32;
        let mut err_var = String::new();
        while idx < lines.len() {
            let line = lines[idx].trim();
            if depth == 0 && line == "end" { idx += 1; break; }
            if depth == 0 && (line == "~catch" || line.starts_with("~catch ") || line == "catch" || line.starts_with("catch ")) {
                let bare = line.trim_start_matches('~');
                if let Some(after) = bare.strip_prefix("catch ") {
                    if let Some(as_var) = after.strip_prefix("as ") {
                        err_var = as_var.trim().to_string();
                    }
                }
                in_c = true; idx += 1; continue;
            }
            if Self::block_opener(line) { depth += 1; }
            if line == "end" { depth -= 1; }
            if in_c { cb.push(line.to_string()); } else { tb.push(line.to_string()); }
            idx += 1;
        }
        Ok((idx, tb, cb, err_var))
    }

    // Старые методы (String), для совместимости (on_start)
    pub fn collect_simple_block(&self, lines: &[String], mut idx: usize) -> Result<(usize, String), String> {
        let (idx, body) = self.collect_simple_lines(lines, idx)?;
        Ok((idx, body.join("\n")))
    }

    pub fn collect_if_block(&self, lines: &[String], mut idx: usize) -> Result<(usize, String, String), String> {
        let (idx, tb, eb) = self.collect_if_lines(lines, idx)?;
        Ok((idx, tb.join("\n"), eb.join("\n")))
    }

    pub fn collect_try_block(&self, lines: &[String], mut idx: usize) -> Result<(usize, String, String), String> {
        let (idx, tb, cb, _) = self.collect_try_lines(lines, idx)?;
        Ok((idx, tb.join("\n"), cb.join("\n")))
    }

    pub fn collect_case_block(&self, lines: &[String], mut idx: usize) -> Result<(usize, Vec<(String, String)>), String> {
        let mut arms = Vec::new();
        while idx < lines.len() {
            let line = lines[idx].trim();
            if line == "end" { idx += 1; break; }
            if let Some((p, b)) = line.split_once(" -> ") { arms.push((p.trim().to_string(), b.trim().to_string())); }
            idx += 1;
        }
        Ok((idx, arms))
    }

    // Собирает ветки switch, Option паттерн (None = else) + тело.
    // Форма, `case <pat>, ` / `case <pat> > inline` / `else, ` / `else > inline`.
    pub fn collect_switch_arms(&self, lines: &[String], mut idx: usize, brace_form: bool) -> Result<(usize, Vec<(Option<String>, Vec<String>)>), String> {
        let mut arms: Vec<(Option<String>, Vec<String>)> = Vec::new();
        let mut cur: Option<String> = None;
        let mut body: Vec<String> = Vec::new();
        let mut depth = 0i32;
        let term = if brace_form { "}" } else { "end" };
        let mut flush = |arms: &mut Vec<(Option<String>, Vec<String>)>, cur: &mut Option<String>, body: &mut Vec<String>| {
            if cur.is_some() || !body.is_empty() {
                arms.push((cur.take(), std::mem::take(body)));
            } else {
                cur.take();
            }
        };
        loop {
            if idx >= lines.len() { break; }
            let line = lines[idx].trim();
            if depth == 0 {
                if line == term { idx += 1; break; }
                if let Some(rest) = line.strip_prefix("case ") {
                    flush(&mut arms, &mut cur, &mut body);
                    if let Some((p, b)) = rest.split_once(" -> ") {
                        cur = None;
                        arms.push((Some(p.trim().to_string()), vec![b.trim().to_string()]));
                        idx += 1;
                        continue;
                    }
                    cur = Some(rest.trim_end_matches(':').trim().to_string());
                    idx += 1;
                    continue;
                }
                if line == "else" || line.starts_with("else") {
                    flush(&mut arms, &mut cur, &mut body);
                    if let Some(b) = line.strip_prefix("else ") {
                        if let Some((_, b2)) = b.split_once(" -> ") {
                            cur = None;
                            arms.push((None, vec![b2.trim().to_string()]));
                            idx += 1;
                            continue;
                        }
                    }
                    cur = None;
                    idx += 1;
                    continue;
                }
            }
            if Self::block_opener(line) { depth += 1; }
            if line == "end" { depth -= 1; }
            body.push(line.to_string());
            idx += 1;
        }
        flush(&mut arms, &mut cur, &mut body);
        Ok((idx, arms))
    }
}

/// Находит первый тернарный `?` на верхнем уровне и соответствующий ему `, `.
/// Возвращает (позиция '?', позиция ', ') в байтах.
fn find_ternary(expr: &str) -> Option<(usize, usize)> {
    let b = expr.as_bytes();
    let mut i = 0usize;
    let mut depth = 0i32;
    let mut q: Option<u8> = None;
    let mut qpos: Option<usize> = None;
    let mut tern = 0i32;
    while i < b.len() {
        let c = b[i];
        if let Some(qc) = q {
            if c == qc && (i == 0 || b[i - 1] != b'\\') { q = None; }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => { q = Some(c); }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'?' if depth == 0 => {
                if qpos.is_none() { qpos = Some(i); }
                tern += 1;
            }
            b':' if depth == 0 && tern > 0 => {
                tern -= 1;
                if tern == 0 { return qpos.map(|qp| (qp, i)); }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn sandbox_blocked(cmd: &str) -> bool {
    const BLOCKED: &[&str] = &[
        "exit","quit","shell","exec","execoutput","shell_cmd","exec_bg","exec_status",
        "prockill","procexists","process_kill","process_list","process_exists",
        "setenv","set_env","sandbox_off",
        "db_open","db_restore","db_exec","db_query","db_insert","db_create","db_columns",
        "read_file","write_file","append_file","delete_file","delete_dir",
        "copy_file","move_file","create_dir","list_dir","download",
        "tcp_listen","tcp_connect","ws_server","ws_connect",
        "clipboard_set","import","import_pkg",
    ];
    BLOCKED.contains(&cmd)
}

/// Вычисляет ветку тернарника, допуская вложенные тернарники в then/else.
fn eval_ternary_expr(vm: &mut VM, part: &str) -> Result<Value, String> {
    match find_ternary(part) {
        Some((qpos, cpos)) => {
            let cond = part[..qpos].trim();
            let then_part = part[qpos+1..cpos].trim();
            let else_part = part[cpos+1..].trim();
            if !cond.is_empty() && !then_part.is_empty() && !else_part.is_empty() {
                if vm.eval_cond(cond).unwrap_or(false) {
                    eval_ternary_expr(vm, then_part)
                } else {
                    eval_ternary_expr(vm, else_part)
                }
            } else {
                vm.eval_value(part)
            }
        }
        None => vm.eval_value(part),
    }
}

/// Главный evaluate
pub fn evaluate(vm: &mut VM, code: &str) -> Result<Value, String> {
    let stripped = VM::sic(code.trim());
    let code = stripped.trim();
    if code.is_empty() { return Ok(Value::None); }

    // ОДНОСТРОЧНЫЙ if-СТЕЙТМЕНТ
    // `if c then stmt [else stmt] end`, одной строкой, тело = statement(-ы).
    // (Многострочный if пишется как `if c then` + body + `end` и обрабатывается
    // блок-коллектором ДО evaluate; if-ВЫРАЖЕНИЕ в RHS идёт через eval_value.)
    // Держим ДО обработки ';'-цепочек, тело if может содержать ';'.
    if (code.starts_with("if ") || code.starts_with("~if ")) && code.ends_with(" end") {
        let c: &str = if code.starts_with("~if ") { code.strip_prefix("~if ").unwrap() } else { code };
        if let Some((cond, rest)) = VM::parse_if(c) {
            return vm.eval_inline_if_stmt(&cond, &rest);
        }
    }

    // алиасы
    if !vm.aliases.is_empty() && !code.starts_with("~alias") && !code.starts_with("~unalias") {
        if let Some(cmd) = vm.aliases.get(code).cloned() { return evaluate(vm, &cmd); }
        for (alias, cmd) in vm.aliases.clone() {
            if code.starts_with(&alias) && code.len() > alias.len() {
                let rest = &code[alias.len()..];
                return evaluate(vm, &format!("{}{}", cmd, rest));
            }
        }
    }

    // цепочки через ;
    if code.contains(';') {
        let cmds = VM::split_top_level(code, ';');
        if cmds.len() > 1 {
            let mut last = Value::None;
            for cmd in &cmds {
                let cmd = cmd.trim();
                if !cmd.is_empty() { last = evaluate(vm, cmd)?; }
            }
            return Ok(last);
        }
    }

    // МОДУЛЬНАЯ СИСТЕМА
    if code.starts_with("import ") {
        // sandbox, import исполнит произвольный .tk-файл = код-выполнение.
        if vm.sandbox_on {
            let cmd = "import".to_string();
            if sandbox_blocked(&cmd) && !vm.sandbox_allow.contains(&cmd) {
                return Err("Sandbox: command 'import' is blocked".to_string());
            }
        }
        let rest = code[7..].trim();
        if let Some((path, as_ns)) = rest.split_once(" as ") {
            let path = VM::tq(path.trim());
            let ns = as_ns.trim().to_string();
            let content = match std::fs::read_to_string(&path) { Ok(c) => c, Err(e) => return Err(format!("import '{}': {}", path, e)) };
            let prev_exports = vm.exports.clone();
            vm.exports.clear();
            vm.push_scope();
            let result = vm.run_script(&content);
            let exported = std::mem::take(&mut vm.exports);
            // Переменные живут во FLAT-мапе (scopes-стек пуст, туда писать нечего!)
            // поэтому состав модуля собираем по ключам кадра depth_keys, (имя, старое).
            // Старый код брал vm.scopes.pop() (всегда пусто > модуль пуст) и не делал
            // pop_scope (переменные текли в глобал + рассинхрон depth_keys).
            let frame_keys: Vec<String> = vm.depth_keys.last().cloned().unwrap_or_default()
                .into_iter().map(|(k, _)| k).collect();
            let mut module_vars = HashMap::new();
            if exported.is_empty() {
                // Без явных export, всё, что создал/тронул кадр.
                for k in &frame_keys {
                    if let Some(v) = vm.variables.get(k) { module_vars.insert(k.clone(), v.clone()); }
                }
            } else {
                for var_name in &exported {
                    if let Some(val) = vm.variables.get(var_name) { module_vars.insert(var_name.clone(), val.clone()); }
                }
            }
            vm.modules.insert(ns, module_vars);
            vm.pop_scope();
            vm.exports = prev_exports;
            // Ошибку библиотеки не глотаем (иначе битый lib импортируется молча).
            match result {
                Err(e) if e.starts_with("__return__") => {}
                Err(e) if e.starts_with("__break__") => return Err("break outside loop".to_string()),
                Err(e) if e.starts_with("__continue__") => return Err("continue outside loop".to_string()),
                Err(e) => return Err(e),
                Ok(_) => {}
            }
            return Ok(Value::None);
        } else {
            // Simple import, just run the file
            return crate::main::eval::evaluate(vm, &format!("~import(\"{}\")", VM::tq(rest)));
        }
    }
    if code.starts_with("export ") || code.starts_with("~export ") {
        let rest = if code.starts_with("~export ") { &code[8..] } else { &code[7..] };
        let rest = rest.trim();
        if let Some((name, expr)) = VM::sft(rest, '=') {
            let name = name.trim().to_string();
            let value = vm.eval_value(expr.trim())?;
            vm.try_set_var(&name, value.clone())?;
            vm.exports.insert(name.clone());
            return Ok(value);
        } else if !rest.is_empty() {
            vm.exports.insert(rest.to_string());
            return Ok(Value::None);
        }
    }

    // базовые команды
    if code == "break" { return Err("__break__".to_string()); }
    if code == "continue" { return Err("__continue__".to_string()); }
    // return x (без тильды)
    if code == "return" || code.starts_with("return ") || code.starts_with("return(") {
        let mut val_str = code[6..].trim().to_string();
        // return(x) / return (x), снимаем внешние скобки, чтобы return(5) вернул 5
        if val_str.len() >= 2 && val_str.starts_with('(') && val_str.ends_with(')') {
            val_str = val_str[1..val_str.len()-1].trim().to_string();
        }
        let val = if val_str.is_empty() || val_str == "(" { Value::None } else { vm.eval_value(&val_str)? };
        return Err(format!("__return__:{}", crate::main::dtam::encode_value(&val)));
    }

    // help, license, exit
    if code == "help" || code == "help()" {
        if vm.help_shown == 0 { crate::main::help::show_welcome(); vm.help_shown = 1; }
        else { crate::main::help::show_full_ru(); }
        return Ok(Value::None);
    }
    if code == "help en" || code == "help en()" {
        crate::main::help::show_full_en();
        return Ok(Value::None);
    }
    if code.starts_with("help ") && !code.starts_with("help en") {
        crate::main::help::show_topic(&code[5..].trim(), "ru");
        return Ok(Value::None);
    }
    if code.starts_with("help en ") {
        crate::main::help::show_topic(&code[8..].trim(), "en");
        return Ok(Value::None);
    }
    if code == "license" || code == "license()" {
        crate::main::help::show_license();
        return Ok(Value::None);
    }
    if code == "exit()" || code == "quit()" || code == "exit" { std::process::exit(0); }

    // вывод текста
    if code.starts_with("text(") && code.ends_with(')') {
        let val = vm.eval_value(&code[5..code.len()-1])?.as_text().unwrap_or_default();
        let t = vm.interp(&val);
        println!("{}", t);
        return Ok(Value::Text(t));
    }
    if code.starts_with("tx(\"") || code.starts_with("tx('") || code.starts_with("tx(`") {
        // скобочная форма, tx("..."), кавычка идёт ПОСЛЕ скобки (индекс 3)
        if let Some(q) = code.chars().nth(3) {
            if let Some(end) = code[4..].find(q) {
                let text = vm.interp(&code[4..4+end]);
                println!("{}", text);
                return Ok(Value::Text(text));
            }
        }
    }
    if (code.starts_with("tx\"") || code.starts_with("tx'") || code.starts_with("tx`")) && code.len() >= 4 {
        // короткая форма БЕЗ скобок, tx"привет" / tx'hi' / tx`ok`
        let q = code.as_bytes()[2] as char;
        if let Some(end) = code[3..].find(q) {
            let text = vm.interp(&code[3..3+end]);
            println!("{}", text);
            return Ok(Value::Text(text));
        }
    }

    // x++, x, (инкремент/декремент)
    if code.ends_with("++") {
        let name = code[..code.len()-2].trim();
        if VM::is_assignable(name) {
            let val = vm.get_var(name).and_then(|v| v.as_number()).unwrap_or(0.0) + 1.0;
            vm.try_set_var(name, Value::Number(val))?;
            return Ok(Value::Number(val));
        }
    }
    if code.ends_with("--") {
        let name = code[..code.len()-2].trim();
        if VM::is_assignable(name) {
            let val = vm.get_var(name).and_then(|v| v.as_number()).unwrap_or(0.0) - 1.0;
            vm.try_set_var(name, Value::Number(val))?;
            return Ok(Value::Number(val));
        }
    }

    // индексация списка/строки var[idx] и срезы var[start, end]
    if let Some(open) = code.find('[') {
        if code.ends_with(']') {
            let name = code[..open].trim();
            let idx_str = code[open+1..code.len()-1].trim();
            if VM::is_assignable(name) && !idx_str.is_empty() {
                if let Some((start_s, end_s)) = idx_str.split_once(':') {
                    // срез list[a, b]
                    let a = vm.eval_expr(start_s.trim()).unwrap_or(0.0) as isize;
                    let b = if end_s.trim().is_empty() { isize::MAX } else { vm.eval_expr(end_s.trim()).unwrap_or(isize::MAX as f64) as isize };
                    if let Some(val) = vm.get_var(name) {
                        return match val {
                            Value::List(l) => {
                                let len = l.len() as isize;
                                let a = if a < 0 { len + a } else { a };
                                let b = if b < 0 { len + b } else { b };
                                // Перевёрнутый/вылетевший диапазон, пустой срез, а не паника
                                // (l[3, 1] как в Python; a также ограничена сверху len).
                                let a = a.max(0).min(len) as usize; let b = b.max(0).min(len) as usize;
                                if a >= b { Ok(Value::List(vec![])) }
                                else { Ok(Value::List(l[a..b].to_vec())) }
                            }
                            Value::Text(t) => {
                                let chars: Vec<char> = t.chars().collect();
                                let len = chars.len() as isize;
                                let a = if a < 0 { len + a } else { a };
                                let b = if b < 0 { len + b } else { b };
                                let a = a.max(0).min(len) as usize; let b = b.max(0).min(len) as usize;
                                if a >= b { Ok(Value::Text(String::new())) }
                                else { Ok(Value::Text(chars[a..b].iter().collect())) }
                            }
                            _ => Err("Cannot index non-list/text".to_string())
                        };
                    }
                } else if let Ok(idx) = vm.eval_expr(idx_str) {
                    let idx = idx as isize;
                    if let Some(val) = vm.get_var(name) {
                        return match val {
                            Value::List(l) => {
                                let i = if idx < 0 { l.len() as isize + idx } else { idx };
                                if i >= 0 && (i as usize) < l.len() { Ok(l[i as usize].clone()) }
                                else { Err("Index out of range".to_string()) }
                            }
                            Value::Text(t) => {
                                let chars: Vec<char> = t.chars().collect();
                                let i = if idx < 0 { chars.len() as isize + idx } else { idx };
                                if i >= 0 && (i as usize) < chars.len() { Ok(Value::Text(chars[i as usize].to_string())) }
                                else { Err("Index out of range".to_string()) }
                            }
                            _ => Err("Cannot index non-list/text".to_string())
                        };
                    }
                }
            }
        }
    }

    // x += 5, x -= 5, x *= 5, x /= 5 (составные присваивания)
    for &(op, transform) in &[("+=", "+"), ("-=", "-"), ("*=", "*"), ("/=", "/")] {
        if let Some((name, expr)) = code.split_once(op) {
            let name = name.trim();
            let expr = expr.trim();
            if VM::is_assignable(name) && !expr.is_empty() {
                let new_expr = format!("{} = {} {} {}", name, name, transform, expr);
                return evaluate(vm, &new_expr);
            }
        }
    }

    // ДЕСТРУКТУРИЗАЦИЯ
    // [a, b] = list | {a, b} = dict | a, b = 1, 2
    if !code.starts_with('~') {
    if let Some((lhs, rhs)) = VM::sft(code, '=') {
        let lhs = lhs.trim().to_string();
        if !lhs.is_empty() {
            let rhs = rhs.trim().to_string();
            if lhs.starts_with('[') && lhs.ends_with(']') {
                let vars = VM::split_top_level(&lhs[1..lhs.len()-1], ',');
                let val = vm.eval_value(&rhs)?;
                let ty = val.type_name().to_string();
                match &val {
                    Value::List(items) => {
                        if vars.len() > items.len() { return Err(format!("Destructure: list has {} items, need {} vars", items.len(), vars.len())); }
                        for (i, v) in vars.iter().enumerate() {
                            let vn = v.trim();
                            if vn == "_" { continue; }
                            if !VM::is_assignable(vn) { return Err(format!("Destructure: bad var '{}'", vn)); }
                            vm.try_set_var(vn, items.get(i).cloned().unwrap_or(Value::None))?;
                        }
                        return Ok(val);
                    }
                    Value::Text(t) => {
                        let chars: Vec<char> = t.chars().collect();
                        if vars.len() > chars.len() { return Err(format!("Destructure: string has {} chars, need {} vars", chars.len(), vars.len())); }
                        for (i, v) in vars.iter().enumerate() {
                            let vn = v.trim();
                            if vn == "_" { continue; }
                            if !VM::is_assignable(vn) { return Err(format!("Destructure: bad var '{}'", vn)); }
                            vm.try_set_var(vn, Value::Text(chars.get(i).map(|c| c.to_string()).unwrap_or_default()))?;
                        }
                        return Ok(val);
                    }
                    _ => return Err(format!("Destructure: cannot unpack {}", ty)),
                }
            }
            if lhs.starts_with('{') && lhs.ends_with('}') {
                let vars = VM::split_top_level(&lhs[1..lhs.len()-1], ',');
                let val = vm.eval_value(&rhs)?;
                let ty = val.type_name().to_string();
                match &val {
                    Value::Dict(d) => {
                        for v in &vars {
                            let vn = v.trim();
                            if vn == "_" { continue; }
                            if !VM::is_assignable(vn) { return Err(format!("Destructure: bad var '{}'", vn)); }
                            let found = d.get(vn).cloned().unwrap_or(Value::None);
                            vm.try_set_var(vn, found)?;
                        }
                        return Ok(val);
                    }
                    _ => return Err(format!("Destructure: cannot unpack {}", ty)),
                }
            }
            if lhs.contains(',') {
                let lvars = VM::split_top_level(&lhs, ',');
                let rvals = VM::split_top_level(&rhs, ',');
                if lvars.len() != rvals.len() { return Err(format!("Destructure: {} vars but {} values", lvars.len(), rvals.len())); }
                let vals: Result<Vec<Value>, String> = rvals.iter().map(|r| vm.eval_value(r.trim())).collect();
                let vals = vals?;
                for (v, val) in lvars.iter().zip(vals.iter()) {
                    let vn = v.trim();
                    if vn == "_" { continue; }
                    if !VM::is_assignable(vn) { return Err(format!("Destructure: bad var '{}'", vn)); }
                    vm.try_set_var(vn, val.clone())?;
                }
                return Ok(vals.last().cloned().unwrap_or(Value::None));
            }
        }
    }
    } // !code.starts_with('~')

    // простое присваивание x = expr (с поддержкой тернарного оператора)
    if let Some((name, expr)) = VM::sft(code, '=') {
        if VM::is_assignable(&name) {
            let value = match find_ternary(&expr) {
                Some((qpos, cpos)) => {
                    let cond = expr[..qpos].trim();
                    let then_part = expr[qpos+1..cpos].trim();
                    let else_part = expr[cpos+1..].trim();
                    if !cond.is_empty() && !then_part.is_empty() && !else_part.is_empty() {
                        if vm.eval_cond(cond).unwrap_or(false) {
                            eval_ternary_expr(vm, then_part)?
                        } else {
                            eval_ternary_expr(vm, else_part)?
                        }
                    } else { vm.eval_value(&expr)? }
                }
                None => vm.eval_value(&expr)?,
            };
            vm.try_set_var(name.trim(), value.clone())?;
            return Ok(value);
        }
    }

    // вызов функции верхнего уровня, myfunc(x)
    if code.contains('(') && code.ends_with(')') {
        if let Some(open) = code.find('(') {
            let name = code[..open].trim();
            if !name.contains(' ') && !name.contains('~') && !name.contains('(') {
                if vm.functions.contains_key(name) || matches!(vm.get_var(name), Some(Value::Lambda(..))) {
                    return vm.eval_value(code);
                }
                // метод-синтаксис, list.push(x) > ~add(list, x)
                if let Some(rewritten) = VM::method_rewrite(name, &code[open+1..code.len()-1].trim()) {
                    return evaluate(vm, &rewritten);
                }
            }
        }
    }

    // ~def name(params), body, определение функции
    if code.starts_with("~def ") || code.starts_with("def ") {
        let code_str = if code.starts_with("~def ") { &code[5..] } else { &code[4..] };
        let rest = code_str.trim();
        if let Some((head, body)) = VM::sft(rest, ',') {
            let head = head.trim();
            let body = body.trim();
            let (name, params) = if let Some(open) = head.find('(') {
                if let Some(close) = head.rfind(')') {
                    let nm = head[..open].trim().to_string();
                    let params_str = &head[open+1..close];
                    let params: Vec<String> = if params_str.trim().is_empty() { Vec::new() } else {
                        VM::split_top_level(params_str, ',').into_iter().map(|p| p.trim().to_string()).collect()
                    };
                    (nm, params)
                } else { return Err("Bad def header: missing )".to_string()); }
            } else { (head.to_string(), Vec::new()) };
            let body_cache = crate::main::bytecode::compile_body(&body, &params);
            vm.functions.insert(name.clone(), crate::main::vm::FunctionDef {
                params, body: body.to_string(),
                code: body_cache,
            });
            vm.echo(format!("Function '{}' created", name));
            return Ok(Value::None);
        }
    }

    // ~call name(args), вызов функции
    if code.starts_with("~call ") || code.starts_with("call ") {
        let rest = if code.starts_with("~call ") { &code[6..] } else { &code[5..] };
        let rest = rest.trim();
        if let Some(open) = rest.find('(') {
            // сбалансированная закрывающая скобка ПЕРВОГО вызова (не последняя в строке
            // `call fib(n-1) + call fib(n-2)` > args = "n-1", не "n-1) + call fib(n-2")
            let mut depth = 0i32;
            let mut close = None;
            for (bi, bc) in rest.bytes().enumerate().skip(open + 1) {
                match bc {
                    b'(' => depth += 1,
                    b')' if depth == 0 => { close = Some(bi); break; }
                    b')' => depth -= 1,
                    _ => {}
                }
            }
            if let Some(close) = close {
                // Составное выражение (`call f(x) + ...`), разбирает целиком
                // eval_func_expr. Иначе хвост после первого вызова молча терялся
                // (slow `call fib(15)` давал 1 вместо 610).
                if !rest[close + 1..].trim().is_empty() {
                    return vm.eval_func_expr(rest);
                }
                let name = rest[..open].trim().to_string();
                let args_str = &rest[open+1..close];
                let args = vm.parse_arg_specs(args_str)?;
                if let Some(func) = vm.functions.get(&name).cloned() {
                    if vm.call_stack.len() >= 1000 {
                        return Err("recursion depth limit exceeded".to_string());
                    }
                    vm.call_stack.push(name.clone());
                    vm.push_scope();
                    vm.bind_named_params(&func.params, &args)?;
                    let result = if func.body.starts_with("return") || func.body.contains('\n') || func.body.contains("if ") || func.body.contains("while ") {
                        // Тело с return, через скрипт
                        match vm.run_script(&func.body) {
                            Ok(v) => v,
                            Err(e) if e.starts_with("__return__:") => { crate::main::dtam::decode_value(&e[11..]).unwrap_or(Value::None) }
                            Err(e) => { vm.pop_scope(); vm.call_stack.pop(); return Err(e); }
                        }
                    } else {
                        // Простое тело a * 2, как выражение. Жёсткие ошибки
                        // (деление на ноль и т.п.) пробрасываем, иначе slow
                        // молча печатает сырой текст вызова вместо ошибки.
                        // Откат на run_script, только для мягких ("не выражение").
                        match vm.eval_value(&func.body) {
                            Ok(v) => v,
                            Err(e) if !crate::main::vm::VM::is_soft_eval_error(&e) => {
                                vm.pop_scope(); vm.call_stack.pop(); return Err(e);
                            }
                            Err(_) => {
                                match vm.run_script(&func.body) {
                                    Ok(v) => v,
                                    Err(e) if e.starts_with("__return__:") => { crate::main::dtam::decode_value(&e[11..]).unwrap_or(Value::None) }
                                    Err(e) => { vm.pop_scope(); vm.call_stack.pop(); return Err(e); }
                                }
                            }
                        }
                    };
                    vm.pop_scope();
                    vm.call_stack.pop();
                    return Ok(result);
                }
                return Err(format!("Function '{}' not found", name));
            }
        }
    }

    // ~return x
    // Песочница, блокировка опасных команд
    if vm.sandbox_on {
        let cmd = VM::extract_cmd_name(code.trim_start_matches('~')).to_string();
        if sandbox_blocked(&cmd) && !vm.sandbox_allow.contains(&cmd) {
            return Err(format!("Sandbox: command '{}' is blocked", cmd));
        }
    }
    // Нормализация имён, убираем обязательную ~ и переводим новые имена в старые.
    // Пользователь пишет list_push(...) или print(...), резолверы получают ~add(...).
    let code = if code.starts_with('~') || code.starts_with("kg.") {
        code.to_string()
    } else {
        translate_alias(code)
    };

    // Кешированная диспетчеризация команд
    if code.starts_with('~') || code.starts_with("kg.") {
        let cmd_name = VM::extract_cmd_name(&code).to_string();
        if let Some(&module_idx) = vm.cmd_cache.get(&cmd_name) {
            if let Some(result) = commands::RESOLVERS[module_idx as usize](vm, &code) {
                return result;
            }
            return Err(format!("\x1B[91mUnknown command: {}\x1B[0m", code.trim_start_matches('~')));
        }
        for (i, &resolver) in commands::RESOLVERS.iter().enumerate() {
            if let Some(result) = resolver(vm, &code) {
                vm.cmd_cache.insert(cmd_name, i as u16);
                return result;
            }
        }
        return Err(format!("\x1B[91mUnknown command: {}\x1B[0m", code.trim_start_matches('~')));
    }

    // Авто-добавление ~ для любой команды (все команды работают без ~)
    if let Some(res) = commands::fast::resolve(vm, &code) {
        return res;
    }
    return evaluate(vm, &format!("~{}", code));

    // Err(format!("\x1B[91mUnknown command, {}\x1B[0m", code)), недостижимо
}

/// БЫСТРЫЙ диспетч команды из байткода, имя + уже вычисленные аргументы (Value).
/// Минует sic(), алиасы, цепочки ';', import/export, sandbox, весь пре-скан evaluate
/// который на каждой команде в горячем цикле транслятора повторяется впустую.
pub fn eval_cmd_fast(vm: &mut VM, name: &str, args: &[Value]) -> Result<Value, String> {
    // собираем строку вызова, name(arg1, arg2, ...)
    // Голые идентификаторы (имена переменных, числа) оставляем как есть
    // их резолвит повторный парсинг. Всё остальное (пустая строка, пробелы
    // запятые, скобки) оборачиваем в кавычки, иначе оно исчезает или ломает
    // разбор, пустое превращалось в print() > none. Уже закавыченные
    // (конвенция Str) оставляем как есть.
    let parts: Vec<String> = args.iter().map(|v| match v {
        Value::Text(t) => {
            let b = t.as_bytes();
            if b.len() >= 2
                && ((b[0] == b'"' && b[b.len() - 1] == b'"')
                    || (b[0] == b'\'' && b[b.len() - 1] == b'\''))
            {
                t.clone()
            } else if !t.is_empty()
                && t.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.')
            {
                t.clone()
            } else {
                format!("\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\""))
            }
        }
        _ => v.display(),
    }).collect();
    let call = format!("{}({})", name, parts.join(", "));
    // переводим новое имя в старое (~см. до), как в evaluate
    let call = if call.starts_with('~') || call.starts_with("kg.") {
        call
    } else {
        translate_alias(&call)
    };

    if !call.starts_with('~') && !call.starts_with("kg.") {
        // не команда (например функция), пусть обычный evaluate разберётся
        return evaluate(vm, &call);
    }

    // Песочница, та же проверка blocked, что и в evaluate (иначе байткодный Call
    // обходит sandbox). Проверяем ПОСЛЕ translate_alias, по итоговому имени.
    if vm.sandbox_on {
        let cmd = VM::extract_cmd_name(call.trim_start_matches('~')).to_string();
        if sandbox_blocked(&cmd) && !vm.sandbox_allow.contains(&cmd) {
            return Err(format!("Sandbox: command '{}' is blocked", cmd));
        }
    }

    let cmd_name = VM::extract_cmd_name(&call).to_string();
    if let Some(&module_idx) = vm.cmd_cache.get(&cmd_name) {
        if let Some(result) = commands::RESOLVERS[module_idx as usize](vm, &call) {
            return result;
        }
        return Err(format!("\x1B[91mUnknown command: {}\x1B[0m", call.trim_start_matches('~')));
    }
    for (i, &resolver) in commands::RESOLVERS.iter().enumerate() {
        if let Some(result) = resolver(vm, &call) {
            vm.cmd_cache.insert(cmd_name, i as u16);
            return result;
        }
    }
    // не нашли резолвер, пусть обычный evaluate попробует (функции/спец-имена)
    evaluate(vm, &call)
}