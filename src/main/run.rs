use crate::main::vm::{VM, Value};
use crate::main::eval::evaluate;
use std::io::{self, IsTerminal, Write};
use std::fs;

/// Верхнеуровневый перевод управляющих сигналов в пользовательские ошибки.
/// `break`/`continue`, долетевшие до верха (вне цикла), ошибка, а не тишина
/// (раньше давились молча во всех 4 точках запуска).
/// `__return__` вне функции оставляем молчаливым (историческое поведение).
fn top_level_error(e: &str) -> Option<String> {
    if e.starts_with("__break__") {
        Some("break outside loop".to_string())
    } else if e.starts_with("__continue__") {
        Some("continue outside loop".to_string())
    } else if e.starts_with("__return__") {
        None
    } else {
        Some(e.to_string())
    }
}

pub fn run() {
    let all_args: Vec<String> = std::env::args().collect();

    // встроенный payload (exe, собранный через `treekraft build`)
    // Если в хвосте собственного бинарника есть payload, исполнить его и выйти.
    if let Some((flags, mut payload)) = read_own_payload() {
        run_payload(&all_args, flags, &mut payload);
        return;
    }

    // tkp, пакетный менеджер (GitHub как сервер)
    if all_args.get(1).map(|a| a.as_str()) == Some("tkp") {
        tkp_run(&all_args[2..]);
        return;
    }

    // build, компиляция .tk > .exe (stub + payload)
    if all_args.get(1).map(|a| a.as_str()) == Some("build") {
        build_run(&all_args[2..]);
        return;
    }

    // test, тест-раннер (прогон файлов, сводный отчёт, exit-код)
    if all_args.get(1).map(|a| a.as_str()) == Some("test") {
        test_run(&all_args[2..]);
        return;
    }

    let script_path = all_args.get(1).filter(|a| {
        let l = a.to_lowercase();
        l.ends_with(".tk") || l.ends_with(".trek") || l.ends_with(".treekraft")
    });
    if let Some(path) = script_path {
        // Абсолютный путь, чтобы после смены CWD чтение файла не сломалось
        let abs = std::path::Path::new(path).canonicalize()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default().join(path));
        // Рабочая папка = папка скрипта, чтобы относительные пути к ассетам работали
        if let Some(parent) = abs.parent() {
            if !parent.as_os_str().is_empty() { let _ = std::env::set_current_dir(parent); }
        }
        let content = match fs::read_to_string(&abs) {
            Ok(c) => c,
            Err(e) => { println!("\x1B[91merror: {}\x1B[0m", e); return; }
        };
        let mut vm = VM::new();
        vm.noecho = true; // скрипт, команды не эхо-печатают значения (REPL оставляет эхо)
        vm.script_name = abs.to_string_lossy().to_string();
        vm.args = all_args[2..].to_vec();
        vm.set_var("score", Value::Number(0.0));
        vm.set_var("lives", Value::Number(3.0));
        vm.set_var("hp", Value::Number(100.0));
        vm.set_var("xp", Value::Number(0.0));
        vm.set_var("gamestate", Value::Text("main".to_string()));
        vm.for_stack.clear();
        if let Err(e) = vm.run_script(&content) {
            if let Some(msg) = top_level_error(&e) {
                println!("\x1B[91merror: {}\x1B[0m", msg);
            }
        }
        // Консоль не захлопывается при запуске двойным кликом (или вручную из терминала)
        if std::io::stdin().is_terminal() {
            println!("\nPress Enter to close...");
            let _ = io::stdin().read_line(&mut String::new());
        }
        return;
    }

    // REPL
    println!("\n  TreeKraft u1 (x86_64) on windows");
    println!("  Type help, license, exit\n");

    let mut vm = VM::new();
    vm.args = all_args[1..].to_vec();
    vm.set_var("score", Value::Number(0.0));
    vm.set_var("lives", Value::Number(3.0));
    vm.set_var("hp", Value::Number(100.0));
    vm.set_var("xp", Value::Number(0.0));
    vm.set_var("gamestate", Value::Text("main".to_string()));
    vm.set_var("level", Value::Number(1.0));

    loop {
        print!(">>> ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() { break; }
        let mut line = input.trim_end_matches(['\r', '\n']).to_string();
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() { continue; }
        if VM::block_opener(&trimmed) || trimmed == "~{" {
            match VM::collect_block_from_stdin(trimmed.clone()) {
                Ok(b) => line = b,
                Err(e) => { println!("\x1B[91merror: {}\x1B[0m", e); continue; }
            }
        } else {
            line = trimmed;
        }
        if line == "exit()" || line == "quit()" || line == "exit" { println!("Goodbye!"); break; }
        if !line.contains('\n') { vm.history.push(line.clone()); }
        vm.for_stack.clear();
        let result = if line.contains('\n') {
            vm.run_script(&line)
        } else {
            evaluate(&mut vm, &line)
        };
        match result {
            Ok(v) => { if !matches!(v, Value::None) { vm.set_var("_", v); } }
            Err(e) => {
                if let Some(msg) = top_level_error(&e) {
                    println!("\x1B[91merror: {}\x1B[0m", msg);
                }
            }
        }
        vm.autosave_counter += 1;
        if let Some(ref f) = vm.autosave_file.clone() {
            if vm.autosave_counter % 50 == 0 {
                let _ = crate::main::dtam::save_state(&vm, f);
            }
        }
    }
}

/// tkp, пакетный менеджер. GitHub как сервер, raw.githubusercontent.com/<ник>/<имя>/<ref>/<имя>.tk
/// База для скачивания переопределяется через TREK_TKP_BASE (тесты/локальные зеркала).
fn tkp_base() -> String {
    std::env::var("TREK_TKP_BASE").unwrap_or_else(|_| "https://raw.githubusercontent.com".to_string())
}

fn valid_pkg_component(s: &str) -> bool {
    !s.is_empty() && !s.contains('/') && !s.contains('\\') && !s.contains(':')
        && !s.contains("..") && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
}

/// Запись реестра, ник + путь + тег/ветка, с которой установлено.
/// registry.json хранит либо строку-путь (старый формат, ник/tag условно пусто/main)
/// либо объект {"nick", ..., "path", ..., "tag", ...} (новый).
#[derive(Clone)]
struct TkpEntry { nick: String, path: String, tag: String }

fn read_tkp_registry(path: &std::path::Path) -> std::collections::HashMap<String, TkpEntry> {
    let mut out = std::collections::HashMap::new();
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(crate::main::vm::Value::Dict(d)) = crate::main::vm::json_parse(&data) {
            for (k, v) in d {
                match v {
                    crate::main::vm::Value::Text(t) => {
                        out.insert(k, TkpEntry { nick: String::new(), path: t, tag: "main".to_string() });
                    }
                    crate::main::vm::Value::Dict(o) => {
                        let path = o.get("path").map(|x| x.display()).unwrap_or_default();
                        if path.is_empty() { continue; }
                        let nick = o.get("nick").map(|x| x.display()).unwrap_or_default();
                        let tag = o.get("tag").map(|x| x.display()).unwrap_or_else(|| "main".to_string());
                        out.insert(k, TkpEntry { nick, path, tag });
                    }
                    _ => {}
                }
            }
        }
    }
    out
}

fn write_tkp_registry(path: &std::path::Path, reg: &std::collections::HashMap<String, TkpEntry>) {
    let mut v = std::collections::HashMap::new();
    for (k, e) in reg {
        let mut o = std::collections::HashMap::new();
        o.insert("nick".to_string(), crate::main::vm::Value::Text(e.nick.clone()));
        o.insert("path".to_string(), crate::main::vm::Value::Text(e.path.clone()));
        o.insert("tag".to_string(), crate::main::vm::Value::Text(e.tag.clone()));
        v.insert(k.clone(), crate::main::vm::Value::Dict(o));
    }
    let json = crate::main::vm::simple_json_stringify(&crate::main::vm::Value::Dict(v));
    let _ = fs::write(path, json);
}

fn tkp_run(args: &[String]) {
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_else(|_| ".".to_string());
    let pkg_dir = std::path::Path::new(&home).join(".treekraft").join("pkgs");
    let registry = pkg_dir.join("registry.json");

    let cmd = args.first().map(|a| a.as_str()).unwrap_or("");
    match cmd {
        "install" => {
            let spec = args.get(1).cloned().unwrap_or_default();
            if spec.is_empty() {
                println!("usage: tkp install ник/имя[@версия]");
                return;
            }
            // формат, ник/имя[@tag]
            let (nick_repo, tag) = match spec.split_once('@') {
                Some((a, b)) => (a.to_string(), b.to_string()),
                None => (spec.clone(), "main".to_string()),
            };
            let (nick, repo) = match nick_repo.split_once('/') {
                Some((n, r)) => (n.to_string(), r.to_string()),
                None => {
                    // без ника, локальная установка из packages/<имя>.tk, <имя>.tk или ~/.treekraft/pkgs
                    let name = nick_repo.clone();
                    if !valid_pkg_component(&name) {
                        println!("Invalid package name: '{}'", name);
                        return;
                    }
                    let candidates = [
                        format!("packages/{}.tk", name),
                        format!("{}.tk", name),
                        pkg_dir.join(format!("{}.tk", name)).to_string_lossy().to_string(),
                    ];
                    let mut src = None;
                    for c in candidates.iter() {
                        if std::path::Path::new(c).exists() { src = Some(c.clone()); break; }
                    }
                    let src = match src {
                        Some(s) => s,
                        None => {
                            println!("usage: tkp install ник/имя  (например: имя/имя)");
                            return;
                        }
                    };
                    let content = match fs::read_to_string(&src) { Ok(c) => c, Err(e) => { println!("Read error: {}", e); return; } };
                    std::fs::create_dir_all(&pkg_dir).ok();
                    let dest = pkg_dir.join(format!("{}.tk", name));
                    if let Err(e) = fs::write(&dest, &content) { println!("Write error: {}", e); return; }
                    let mut reg = read_tkp_registry(&registry);
                    reg.insert(name.clone(), TkpEntry { nick: String::new(), path: dest.to_string_lossy().to_string(), tag: "local".to_string() });
                    write_tkp_registry(&registry, &reg);
                    println!("Package '{}' installed (local, {} commands)", name, content.lines().count());
                    return;
                }
            };
            if !valid_pkg_component(&nick) || !valid_pkg_component(&repo) || !valid_pkg_component(&tag) {
                println!("Invalid package spec: '{}/{}@{}'", nick, repo, tag);
                return;
            }
            let url = format!("{}/{}/{}/{}/{}.tk", tkp_base(), nick, repo, tag, repo);
            println!("Downloading {} ...", url);
            match http_get_text(&url) {
                Ok(content) => {
                    std::fs::create_dir_all(&pkg_dir).ok();
                    let dest = pkg_dir.join(format!("{}.tk", repo));
                    if let Err(e) = fs::write(&dest, &content) {
                        println!("Write error: {}", e);
                        return;
                    }
                    let mut reg = read_tkp_registry(&registry);
                    reg.insert(repo.clone(), TkpEntry { nick: nick.clone(), path: dest.to_string_lossy().to_string(), tag });
                    write_tkp_registry(&registry, &reg);
                    println!("Installed '{}' ({} commands)", repo, content.lines().count());
                }
                Err(e) => println!("{}", e),
            }
        }
        "update" => {
            // tkp update, переустановить ВСЕ пакеты (по сохранённому нику/тегу)
            // tkp update ник/имя, переустановить один (или установить, если нет)
            let spec = args.get(1).map(|s| s.as_str()).unwrap_or("");
            let mut reg = read_tkp_registry(&registry);
            let names: Vec<(String, String)> = if spec.is_empty() {
                reg.iter().map(|(k, e)| (k.clone(), e.nick.clone())).collect()
            } else if let Some((nick, repo)) = spec.split_once('/') {
                if !valid_pkg_component(nick) || !valid_pkg_component(repo) {
                    println!("Invalid package spec: '{}'", spec);
                    return;
                }
                if let Some(e) = reg.get(repo) {
                    vec![(repo.to_string(), e.nick.clone())]
                } else {
                    // не установлен, https-путь как install (без подтверждения)
                    let url = format!("{}/{}/{}/main/{}.tk", tkp_base(), nick, repo, repo);
                    match http_get_text(&url) {
                        Ok(content) => {
                            std::fs::create_dir_all(&pkg_dir).ok();
                            let dest = pkg_dir.join(format!("{}.tk", repo));
                            if let Err(e) = fs::write(&dest, &content) { println!("Write error: {}", e); return; }
                            let mut r2 = read_tkp_registry(&registry);
                            r2.insert(repo.to_string(), TkpEntry { nick: nick.to_string(), path: dest.to_string_lossy().to_string(), tag: "main".to_string() });
                            write_tkp_registry(&registry, &r2);
                            println!("Installed '{}' ({} commands)", repo, content.lines().count());
                        }
                        Err(e) => println!("{}", e),
                    }
                    return;
                }
            } else {
                let repo = spec.to_string();
                if !valid_pkg_component(&repo) {
                    println!("Invalid package name: '{}'", repo);
                    return;
                }
                match reg.get(&repo) {
                    Some(e) => vec![(repo, e.nick.clone())],
                    None => {
                        println!("Not installed: '{}'. Используйте tkp update ник/имя или tkp install ник/имя", repo);
                        return;
                    }
                }
            };
            if names.is_empty() {
                println!("  (no packages installed)");
                return;
            }
            let mut updated = 0;
            for (name, nick) in &names {
                let entry = match reg.get(name).cloned() { Some(e) => e, None => continue };
                if entry.nick.is_empty() {
                    println!("  '{}': локальная установка, пропуск", name);
                    continue;
                }
                let nick = if nick.is_empty() { &entry.nick } else { nick };
                let url = format!("{}/{}/{}/{}/{}.tk", tkp_base(), nick, name, entry.tag, name);
                println!("  updating {} @{} ...", name, entry.tag);
                match http_get_text(&url) {
                    Ok(content) => {
                        if let Err(e) = fs::write(&entry.path, &content) {
                            println!("  Write error for '{}': {}", name, e);
                            continue;
                        }
                        updated += 1;
                    }
                    Err(e) => println!("  {}: {}", name, e),
                }
            }
            println!("update: {} package(s) updated", updated);
        }
        "remove" => {
            let name = args.get(1).cloned().unwrap_or_default();
            if name.is_empty() { println!("usage: tkp remove имя"); return; }
            let mut reg = read_tkp_registry(&registry);
            if let Some(entry) = reg.remove(&name) {
                let _ = fs::remove_file(&entry.path);
                write_tkp_registry(&registry, &reg);
                println!("Removed '{}'", name);
            } else {
                println!("Package '{}' not installed", name);
            }
        }
        "list" => {
            let reg = read_tkp_registry(&registry);
            if reg.is_empty() { println!("  (no packages installed)"); return; }
            for (k, e) in &reg { println!("  {} -> {} @{}", k, e.path, e.tag); }
        }
        "path" => {
            let name = args.get(1).cloned().unwrap_or_default();
            if name.is_empty() { println!("usage: tkp path имя"); return; }
            let reg = read_tkp_registry(&registry);
            match reg.get(&name) {
                Some(e) => println!("{}", e.path),
                None => println!("Package '{}' not installed", name),
            }
        }
        _ => {
            println!("tkp — пакетный менеджер TreeKraft");
            println!("  tkp install ник/имя[@ветка]   — установить из GitHub (по умолчанию main)");
            println!("  tkp update [ник/имя]          — переустановить всё / один пакет");
            println!("  tkp remove имя               — удалить");
            println!("  tkp list                     — список установленных");
            println!("  tkp path имя                 — показать путь установленного пакета");
        }
    }
}

/// Скачивание файла по HTTP(S) через системный curl (есть в Windows 10+/msys).
fn http_get_text(url: &str) -> Result<String, String> {
    let out = std::process::Command::new("curl")
        .args(["-fsSL", "--max-time", "30", "--max-redirs", "5", url])
        .output()
        .map_err(|e| format!("curl: {}", e))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        // curl -f, 404/403 > понятная диагностика без сырого stderr
        if err.contains("404") && !err.contains("Failed to connect") {
            return Err(format!("Not found: {} (проверьте ник/имя/ветку)", url));
        }
        if err.contains("403") && !err.contains("Failed to connect") {
            return Err(format!("Access denied (403): {}", url));
        }
        Err(err)
    }
}

// build, компиляция .tk > .exe
// Формат, [stub = копия treekraft.exe][payload][footer 25 байт]
// payload = "TREK1" + исходник (открытым текстом, без защиты)
// footer = len, u64 LE | flags, u8 | key, u64 LE (резерв) | магия "TREKEXE!"
// Защиты нет сознательно, исходники рантайма открыты, любая схема вскрывалась бы
// чтением кода расшифровки. flags != 0 = чужой/неподдерживаемый payload.

const EXE_MAGIC: &[u8; 8] = b"TREKEXE!";
const INNER_MAGIC: &[u8; 5] = b"TREK1";

/// Читать payload из хвоста собственного бинарника. None = обычный запуск.
fn read_own_payload() -> Option<(u8, Vec<u8>)> {
    let exe = std::env::current_exe().ok()?;
    let data = std::fs::read(&exe).ok()?;
    if data.len() < 25 { return None; }
    let f = &data[data.len() - 25..];
    if &f[17..25] != EXE_MAGIC { return None; }
    let len = u64::from_le_bytes(f[0..8].try_into().ok()?) as usize;
    let flags = f[8];
    if len == 0 || len > data.len() - 25 { return None; }
    let start = data.len() - 25 - len;
    Some((flags, data[start..start + len].to_vec()))
}

/// Исполнить встроенный payload (путь собранного exe).
fn run_payload(all_args: &[String], flags: u8, payload: &mut [u8]) {
    if flags != 0 {
        println!("Unsupported payload (built with protection, rebuild without it)");
        return;
    }
    if payload.len() < 5 || &payload[..5] != INNER_MAGIC {
        println!("Broken payload (bad magic)");
        return;
    }
    let src = String::from_utf8_lossy(&payload[5..]).to_string();
    let mut vm = VM::new();
    vm.noecho = true;
    if let Ok(exe) = std::env::current_exe() {
        vm.script_name = exe.to_string_lossy().to_string();
    }
    vm.args = all_args.iter().skip(1).cloned().collect();
    vm.set_var("score", Value::Number(0.0));
    vm.set_var("lives", Value::Number(3.0));
    vm.set_var("hp", Value::Number(100.0));
    vm.set_var("xp", Value::Number(0.0));
    vm.set_var("gamestate", Value::Text("main".to_string()));
    vm.for_stack.clear();
    if let Err(e) = vm.run_script(&src) {
        if let Some(msg) = top_level_error(&e) {
            println!("\x1B[91merror: {}\x1B[0m", msg);
        }
    }
    if std::io::stdin().is_terminal() {
        println!("\nPress Enter to close...");
        let _ = io::stdin().read_line(&mut String::new());
    }
}

/// `treekraft build app.tk [-o app.exe]`
fn build_run(args: &[String]) {
    let mut src = String::new();
    let mut out = String::new();
    let mut it = args.iter().peekable();
    while let Some(a) = it.next() {
        if a == "-o" || a == "--out" {
            if let Some(v) = it.next() { out = v.clone(); }
        } else if let Some(v) = a.strip_prefix("-o") {
            if !v.is_empty() { out = v.to_string(); }
        } else if a.starts_with('-') {
            println!("Unknown flag: {} \nusage: treekraft build app.tk [-o app.exe]", a);
            return;
        } else if src.is_empty() {
            src = a.clone();
        }
    }
    if src.is_empty() {
        println!("usage: treekraft build app.tk [-o app.exe]");
        return;
    }
    let l = src.to_lowercase();
    if !(l.ends_with(".tk") || l.ends_with(".trek") || l.ends_with(".treekraft")) {
        println!("Source must be .tk/.trek/.treekraft, got: {}", src);
        return;
    }
    let source = match fs::read_to_string(&src) {
        Ok(c) => c,
        Err(e) => { println!("Read error: {}", e); return; }
    };
    if out.is_empty() {
        let stem = std::path::Path::new(&src)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "app".to_string());
        out = format!("{}.exe", stem);
    }
    // stub = собственный бинарник (payload-exe сюда не попадают, они исполняют payload)
    let stub = match std::env::current_exe().ok().and_then(|p| fs::read(&p).ok()) {
        Some(b) => b,
        None => { println!("Cannot read own binary as stub"); return; }
    };
    let mut inner = Vec::with_capacity(5 + source.len());
    inner.extend_from_slice(INNER_MAGIC);
    inner.extend_from_slice(source.as_bytes());
    let mut exe = Vec::with_capacity(stub.len() + inner.len() + 25);
    exe.extend_from_slice(&stub);
    exe.extend_from_slice(&inner);
    exe.extend_from_slice(&(inner.len() as u64).to_le_bytes());
    exe.push(0u8); // flags = 0 (без защиты)
    exe.extend_from_slice(&0u64.to_le_bytes()); // key, резерв
    exe.extend_from_slice(EXE_MAGIC);
    if let Err(e) = fs::write(&out, &exe) {
        println!("Write error: {}", e);
        return;
    }
    println!("Built '{}' ({} bytes)", out, exe.len());
}

/// `treekraft test a.tk [b.tk ...]`, тест-раннер.
/// Каждый файл, изолированная VM с test_continue=true (упавший assert не рвёт
/// прогон, все падения собираются в отчёт). Итог, сводка + exit-код
/// (0, всё зелёное, 1, есть падения/ошибки).
fn test_run(files: &[String]) {
    if files.is_empty() {
        println!("usage: treekraft test a.tk [b.tk ...]");
        println!("  собирает все assert_* падения за один прогон, в конце — сводка");
        return;
    }
    let mut files_ok = 0;
    let mut files_fail = 0;
    let mut asserts_ok = 0;
    let mut asserts_fail = 0;
    for f in files {
        let l = f.to_lowercase();
        if !(l.ends_with(".tk") || l.ends_with(".trek") || l.ends_with(".treekraft")) {
            println!("  \x1B[91m✗ {}\x1B[0m — не .tk-файл, пропуск", f);
            files_fail += 1;
            continue;
        }
        let content = match fs::read_to_string(f) {
            Ok(c) => c,
            Err(e) => {
                println!("  \x1B[91m✗ {}\x1B[0m — не читается: {}", f, e);
                files_fail += 1;
                continue;
            }
        };
        let mut vm = VM::new();
        vm.noecho = true;
        vm.test_continue = true;
        vm.script_name = f.clone();
        vm.args = Vec::new();
        vm.set_var("score", Value::Number(0.0));
        vm.set_var("lives", Value::Number(3.0));
        vm.set_var("hp", Value::Number(100.0));
        vm.set_var("xp", Value::Number(0.0));
        vm.set_var("gamestate", Value::Text("main".to_string()));
        vm.for_stack.clear();
        let start = std::time::Instant::now();
        let run_err = vm.run_script(&content).err().and_then(|e| top_level_error(&e));
        let ms = start.elapsed().as_millis();
        let p = vm.test_results.iter().filter(|r| r.1).count();
        let fl = vm.test_results.iter().filter(|r| !r.1).count();
        asserts_ok += p;
        asserts_fail += fl;
        match run_err {
            Some(e) => {
                // жёсткая ошибка (не assert), файл оборвался
                println!("  \x1B[91m✗ {}\x1B[0m — ERROR: {} (прошло {}/{}, {}ms)", f, e, p, p + fl, ms);
                files_fail += 1;
            }
            None if fl > 0 => {
                println!("  \x1B[91m✗ {}\x1B[0m — падений: {} (прошло {}, {}ms)", f, fl, p, ms);
                files_fail += 1;
            }
            None => {
                println!("  \x1B[92m✓ {}\x1B[0m — {} asserts, {}ms", f, p, ms);
                files_ok += 1;
            }
        }
    }
    println!("  ─────────────────────────────────");
    println!(
        "  Файлов: {} ok / {} fail; asserts: {} ok / {} fail",
        files_ok, files_fail, asserts_ok, asserts_fail
    );
    if files_fail > 0 {
        std::process::exit(1);
    }
}
