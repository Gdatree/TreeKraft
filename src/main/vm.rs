use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::process::Command;
use std::net::TcpStream;
use std::time::{Duration, Instant};

// Реэкспорты из модулей
pub use super::value::Value;
pub use super::expr::ExprOp;

// ======================== ВСПОМОГАТЕЛЬНЫЕ СТРУКТУРЫ ========================
#[derive(Clone)]
pub struct FunctionDef {
    pub params: Vec<String>,
    pub body: String,
    /// Кэш скомпилированного тела функции (байткод). Some > JIT-вызов через bytecode::run_func.
    pub code: Option<crate::main::bytecode::BodyCache>,
}

#[derive(Clone)]
pub struct AnimState {
    pub frames: Vec<String>,
    pub fps: f64,
    pub current: usize,
    pub playing: bool,
    pub looping: bool,
    pub pingpong: bool,
    pub pp_dir: i32,
    pub last_update: Instant,
    pub done: bool,
}

#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub ch: char,
    pub life: i32,
}

#[derive(Clone)]
pub struct GameObject {
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub r: f32,
    pub text: String,
    pub color: String,
    pub ch: char,
    pub fg: String,
    pub bg: String,
}

#[derive(Clone)]
pub struct MapObject {
    pub ch: char,
    pub x: usize,
    pub y: usize,
}

#[derive(Clone)]
pub struct Entity {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub ax: f64,
    pub ay: f64,
    pub rot: f64,
    pub sx: f64,
    pub sy: f64,
    pub sprite: String,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            x: 0.0, y: 0.0, vx: 0.0, vy: 0.0,
            ax: 0.0, ay: 0.0, rot: 0.0, sx: 1.0, sy: 1.0,
            sprite: String::new(),
        }
    }
}

// ======================== ПЛАТФОРМОЗАВИСИМЫЕ МОДУЛИ ========================
// Перенесены в src/main/platform.rs (win_key, win_sound)
pub use crate::main::platform::{win_key, win_sound};

// Хелперы
/// Снимает завершающий токен `end` только если он вне строкового литерала
/// (в отличие от trim_end_matches(" end"), который съедал ` end` внутри кавычек).
fn strip_trailing_end(s: &str) -> &str {
    let s = s.trim_end();
    if !s.ends_with("end") { return s; }
    let end_pos = s.len() - 3;
    let before = &s[..end_pos];
    if !(before.is_empty() || before.ends_with(char::is_whitespace)) { return s; }
    let mut q: Option<char> = None;
    let mut prev = '\0';
    for ch in before.chars() {
        if let Some(qc) = q {
            if ch == qc && prev != '\\' { q = None; }
        } else if ch == '"' || ch == '\'' || ch == '`' {
            q = Some(ch);
        }
        prev = ch;
    }
    if q.is_some() { return s; }
    before.trim_end()
}

pub fn run_shell(cmd: &str) -> String {
    #[cfg(windows)]
    let out = Command::new("cmd").args(["/C", cmd]).output();
    #[cfg(not(windows))]
    let out = Command::new("sh").args(["-c", cmd]).output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

pub fn run_shell_status(cmd: &str) -> i32 {
    #[cfg(windows)]
    let out = Command::new("cmd").args(["/C", cmd]).status();
    #[cfg(not(windows))]
    let out = Command::new("sh").args(["-c", cmd]).status();
    match out {
        Ok(s) => s.code().unwrap_or(-1),
        Err(_) => -1,
    }
}

pub fn simple_http_get(url: &str) -> Result<String, String> {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let (host, path) = if let Some(p) = url.find('/') { (&url[..p], &url[p..]) } else { (url, "/") };
    let addr = format!("{}:80", host);
    let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
    let req = format!(
        "GET {} HTTP/1.0\r\nHost: {}\r\nUser-Agent: TreeKraft/1.0.0\r\nConnection: close\r\n\r\n",
        path, host
    );
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut resp = String::new();
    use std::io::Read;
    stream.read_to_string(&mut resp).map_err(|e| e.to_string())?;
    Ok(if let Some(pos) = resp.find("\r\n\r\n") { resp[pos + 4..].to_string() } else { resp })
}

pub fn json_parse(s: &str) -> Result<Value, String> {
    // (реализация из старого main.rs, без изменений)
    let chars: Vec<char> = s.trim().chars().collect();
    let mut pos = 0usize;
    fn skip(c: &[char], p: &mut usize) { while *p < c.len() && c[*p].is_whitespace() { *p += 1; } }
    fn parse_str(c: &[char], p: &mut usize) -> Result<String, String> {
        *p += 1;
        let mut s = String::new();
        while *p < c.len() && c[*p] != '"' {
            if c[*p] == '\\' && *p + 1 < c.len() {
                *p += 1;
                match c[*p] {
                    'n' => s.push('\n'), 't' => s.push('\t'), 'r' => s.push('\r'),
                    '"' => s.push('"'), '\\' => s.push('\\'), '/' => s.push('/'),
                    other => s.push(other),
                }
            } else { s.push(c[*p]); }
            *p += 1;
        }
        *p += 1;
        Ok(s)
    }
    fn parse_num(c: &[char], p: &mut usize) -> Result<Value, String> {
        let start = *p;
        if *p < c.len() && c[*p] == '-' { *p += 1; }
        while *p < c.len() && (c[*p].is_ascii_digit() || c[*p] == '.' || c[*p] == 'e' || c[*p] == 'E' || c[*p] == '+' || c[*p] == '-') { *p += 1; }
        let ns: String = c[start..*p].iter().collect();
        ns.parse::<f64>().map(Value::Number).map_err(|_| format!("Bad number: {}", ns))
    }
    fn parse_val(c: &[char], p: &mut usize) -> Result<Value, String> {
        skip(c, p);
        if *p >= c.len() { return Err("Unexpected EOF".to_string()); }
        match c[*p] {
            '{' => parse_obj(c, p),
            '[' => parse_arr(c, p),
            '"' => Ok(Value::Text(parse_str(c, p)?)),
            't' => if c.len() >= *p+4 && c[*p..*p+4] == ['t','r','u','e'] { *p+=4; Ok(Value::Bool(true)) } else { Err("bad literal".into()) },
            'f' => if c.len() >= *p+5 && c[*p..*p+5] == ['f','a','l','s','e'] { *p+=5; Ok(Value::Bool(false)) } else { Err("bad literal".into()) },
            'n' => if c.len() >= *p+4 && c[*p..*p+4] == ['n','u','l','l'] { *p+=4; Ok(Value::None) } else { Err("bad literal".into()) },
            _ => parse_num(c, p),
        }
    }
    fn parse_arr(c: &[char], p: &mut usize) -> Result<Value, String> {
        *p += 1;
        let mut list = Vec::new();
        skip(c, p);
        if *p < c.len() && c[*p] == ']' { *p+=1; return Ok(Value::List(list)); }
        loop {
            list.push(parse_val(c, p)?);
            skip(c, p);
            if *p < c.len() && c[*p] == ',' { *p+=1; skip(c, p); continue; }
            if *p < c.len() && c[*p] == ']' { *p+=1; break; }
            return Err("Expected , or ]".to_string());
        }
        Ok(Value::List(list))
    }
    fn parse_obj(c: &[char], p: &mut usize) -> Result<Value, String> {
        *p += 1;
        let mut dict = std::collections::HashMap::new();
        skip(c, p);
        if *p < c.len() && c[*p] == '}' { *p+=1; return Ok(Value::Dict(dict)); }
        loop {
            skip(c, p);
            if *p >= c.len() || c[*p] != '"' { return Err("Expected string key".to_string()); }
            let key = parse_str(c, p)?;
            skip(c, p);
            if *p >= c.len() || c[*p] != ':' { return Err("Expected :".to_string()); }
            *p += 1;
            let val = parse_val(c, p)?;
            dict.insert(key, val);
            skip(c, p);
            if *p < c.len() && c[*p] == ',' { *p+=1; continue; }
            if *p < c.len() && c[*p] == '}' { *p+=1; break; }
            return Err("Expected , or }".to_string());
        }
        Ok(Value::Dict(dict))
    }
    parse_val(&chars, &mut pos)
}

pub fn simple_json_stringify(v: &Value) -> String {
    match v {
        Value::None => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => Value::fmt_num(*n),
        Value::Text(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")),
        Value::List(items) => format!("[{}]", items.iter().map(simple_json_stringify).collect::<Vec<_>>().join(",")),
        Value::Dict(d) => { let mut keys: Vec<&String> = d.keys().collect(); keys.sort(); format!("{{{}}}", keys.iter().map(|k| format!("\"{}\":{}", k, simple_json_stringify(&d[*k]))).collect::<Vec<_>>().join(",")) },
        Value::Lambda(..) => "null".to_string(),
    }
}

// ======================== VM ========================
/// Данные лямбды-замыкания, захваченные переменные + их original depth
/// для корректного восстановления после вызова.
#[derive(Clone, Debug)]
pub struct ClosureData {
    pub vars: HashMap<String, Value>,
    pub depth: usize,
}

pub struct VM {
    // FLATTENED SCOPES, O(1) get_var/set_var
    pub variables: HashMap<String, Value>,
    pub scope_depth: usize,
    pub depth_keys: Vec<Vec<(String, Option<Value>)>>,
    pub closures: HashMap<u64, ClosureData>,
    pub fast_regs: Vec<f64>,
    /// Текущая глубина вложенных вызовов функций/лямбд (guard против переполнения стека).
    pub call_depth: usize,
    // legacy scopes (kept for compatibility, will be removed later)
    pub scopes: Vec<HashMap<String, Value>>,
    pub functions: HashMap<String, FunctionDef>,
    pub constants: HashSet<String>,
    pub history: Vec<String>,
    pub args: Vec<String>,
    pub trace: bool,
    pub help_shown: u8,
    pub game_objects: Vec<GameObject>,
    pub game_mode: bool,
    pub sprites: HashMap<String, Vec<String>>,
    pub animations: HashMap<String, AnimState>,
    pub particles: Vec<Particle>,
    pub objects: HashMap<String, MapObject>,
    pub map: Option<crate::main::screen::Screen>,
    pub screen: Option<crate::main::screen::Screen>,
    pub screen_char: char,
    pub cam_x: f64,
    pub cam_y: f64,
    pub method_tmp: usize,
    pub closure_next: u64,
    pub gen_stack: Vec<Vec<Value>>,
    pub viewport_w: usize,
    pub viewport_h: usize,
    pub shake_amount: i32,
    pub zoom_level: f64,
    pub last_frame: Instant,
    pub frame_count: u64,
    pub timers: HashMap<String, Instant>,
    pub autosave_file: Option<String>,
    pub autosave_counter: usize,
    pub menu_items: Vec<String>,
    pub menu_selected: usize,
    pub game_state: String,
    pub game_paused: bool,
    pub score: i64,
    pub lives: i64,
    pub hp: i64,
    pub xp: i64,
    pub stacks: HashMap<String, Vec<Value>>,
    pub queues: HashMap<String, VecDeque<Value>>,
    pub sets: HashMap<String, HashSet<String>>,
    pub cache: HashMap<String, (Value, Option<Instant>, Option<Duration>)>,
    pub log_level: u8,
    pub log_file: Option<String>,
    pub test_results: Vec<(String, bool, String)>,
    /// Режим тест-раннера, упавший assert не прерывает скрипт (push + Ok(false))
    /// чтобы собрать ВСЕ падения за один прогон. Ставит `treekraft test`.
    pub test_continue: bool,
    pub aliases: HashMap<String, String>,
    pub templates: HashMap<String, String>,
    pub template_vars: HashMap<String, Value>,
    pub events: HashMap<String, Vec<String>>,
    pub progress_bars: HashMap<String, (u64, u64)>,
    pub spinner_frames: Vec<&'static str>,
    pub spinner_idx: usize,
    pub config: HashMap<String, String>,
    pub ini_data: HashMap<String, HashMap<String, String>>,
    pub kv_file: Option<String>,
    pub kv_data: HashMap<String, String>,
    pub tables: HashMap<String, Vec<Vec<(String, Value)>>>,
    pub table_cols: HashMap<String, Vec<String>>,
    pub images: HashMap<String, (usize, usize, Vec<(u8, u8, u8)>)>,
    pub sound_volume: u8,
    pub mutexes: HashMap<String, bool>,
    pub thread_log: Vec<u64>,
    pub thread_counter: u64,
    pub chan_counter: u64,
    pub debounce_last: HashMap<String, Instant>,
    pub entities: HashMap<String, Entity>,
    pub gravity_x: f64,
    pub gravity_y: f64,
    pub physics_on: bool,
    pub maps: HashMap<String, Vec<Vec<char>>>,
    pub game_score: Option<f64>,
    pub game_lives: Option<i32>,
    pub game_fps: Option<f64>,
    pub cmd_cache: HashMap<String, u16>,
    pub expr_cache: RefCell<HashMap<String, Vec<ExprOp>>>,
    pub noecho: bool,
    pub modules: HashMap<String, HashMap<String, Value>>,
    pub exports: std::collections::HashSet<String>,
    pub script_line: usize,
    pub script_name: String,
    pub last_error: String,
    pub structs: HashMap<String, Vec<String>>,
    pub enums: HashMap<String, Vec<String>>,
    pub watches: Vec<String>,
    pub call_stack: Vec<String>,
    pub breakpoints: Vec<String>,
    pub sandbox_on: bool,
    pub sandbox_allow: HashSet<String>,
    /// Кадры for-циклов байткода (ForBegin/ForNext), стек для вложенных for.
    pub for_stack: Vec<ForIter>,
}

pub enum ForSource {
    List(Vec<Value>),
    Range(i64, i64),
}

/// Кадр for-цикла в байткоде (ForBegin/ForNext).
pub enum ForIter {
    Range { i: i64, step: i64, to: i64 },
    List { i: usize, items: Vec<Value> },
}

impl VM {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            scope_depth: 0,
            depth_keys: vec![Vec::new()],
            closures: HashMap::new(),
            fast_regs: Vec::new(),
            call_depth: 0,
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            constants: HashSet::new(),
            history: Vec::new(),
            args: Vec::new(),
            trace: false,
            help_shown: 0,
            game_objects: Vec::new(),
            game_mode: false,
            sprites: HashMap::new(),
            animations: HashMap::new(),
            particles: Vec::new(),
            objects: HashMap::new(),
            map: None,
            screen: None,
            screen_char: '*',
            cam_x: 0.0, cam_y: 0.0,
            method_tmp: 0,
            closure_next: 1,
            gen_stack: vec![],
            viewport_w: 80, viewport_h: 24,
            shake_amount: 0,
            zoom_level: 1.0,
            last_frame: Instant::now(),
            frame_count: 0,
            timers: HashMap::new(),
            autosave_file: None,
            autosave_counter: 0,
            menu_items: Vec::new(),
            menu_selected: 0,
            game_state: "main".to_string(),
            game_paused: false,
            score: 0, lives: 3, hp: 100, xp: 0,
            stacks: HashMap::new(),
            queues: HashMap::new(),
            sets: HashMap::new(),
            cache: HashMap::new(),
            log_level: 1,
            log_file: None,
            test_results: Vec::new(),
            test_continue: false,
            aliases: HashMap::new(),
            templates: HashMap::new(),
            template_vars: HashMap::new(),
            events: HashMap::new(),
            progress_bars: HashMap::new(),
            spinner_frames: vec!["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"],
            spinner_idx: 0,
            config: HashMap::new(),
            ini_data: HashMap::new(),
            kv_file: None,
            kv_data: HashMap::new(),
            tables: HashMap::new(),
            table_cols: HashMap::new(),
            images: HashMap::new(),
            sound_volume: 100,
            mutexes: HashMap::new(),
            thread_log: Vec::new(),
            thread_counter: 0,
            chan_counter: 0,
            debounce_last: HashMap::new(),
            entities: HashMap::new(),
            gravity_x: 0.0, gravity_y: 0.0,
            physics_on: false,
            maps: HashMap::new(),
            game_score: None,
            game_lives: None,
            game_fps: None,
            cmd_cache: HashMap::new(),
            expr_cache: RefCell::new(HashMap::new()),
            noecho: false,
            modules: HashMap::new(),
            exports: std::collections::HashSet::new(),
            script_line: 0,
            script_name: String::new(),
            last_error: String::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            watches: Vec::new(),
            call_stack: Vec::new(),
            breakpoints: Vec::new(),
            sandbox_on: false,
            sandbox_allow: HashSet::new(),
            for_stack: Vec::new(),
        }
    }

    // переменные, FLAT O(1) lookup
    pub fn get_var(&self, name: &str) -> Option<Value> {
        // dotted name, module.field or obj.field
        if let Some((ns, var)) = name.split_once('.') {
            if let Some(module) = self.modules.get(ns) {
                if let Some(v) = module.get(var) { return Some(v.clone()); }
            }
            if let Some(obj) = self.get_var(ns) {
                match &obj {
                    Value::Dict(d) => {
                        if let Some(v) = d.get(var) { return Some(v.clone()); }
                    }
                    Value::List(l) => {
                        if let Ok(i) = var.parse::<usize>() {
                            if i < l.len() { return Some(l[i].clone()); }
                        }
                    }
                    _ => {}
                }
            }
            return None;
        }
        // simple name, O(1) HashMap lookup
        self.variables.get(name).cloned()
    }
    pub fn has_var(&self, name: &str) -> bool { self.get_var(name).is_some() }
    /// Мутабельная ссылка на переменную (FLAT O(1), без копирования).
    /// Используется для in-place изменения списков/словарей (~add, ~set, ~dset, ...).
    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut Value> {
        if name.contains('.') { return None; }
        self.variables.get_mut(name)
    }
    pub fn set_var(&mut self, name: &str, value: Value) {
        // dotted, p.x = value
        if let Some((obj_name, field)) = name.split_once('.') {
            if let Some(Value::Dict(mut d)) = self.get_var(obj_name) {
                d.insert(field.to_string(), value);
                if let Some(v) = self.variables.get_mut(obj_name) { *v = Value::Dict(d); return; }
                self.variables.insert(obj_name.to_string(), Value::Dict(d));
                return;
            }
        }
        // FLAT, update existing or insert at current depth
        if let Some(v) = self.variables.get_mut(name) { *v = value; return; }
        self.variables.insert(name.to_string(), value);
        if let Some(keys) = self.depth_keys.last_mut() { keys.push((name.to_string(), None)); }
    }

    // FLAT, пишет в текущую область (для параметров функций).
    // Сохраняет ПРЕДЫДУЩЕЕ значение, чтобы pop_scope() его восстановил
    // (иначе рекурсивные функции несколько вызовов затирают параметры).
    pub fn set_var_local(&mut self, name: &str, value: Value) {
        // dotted-имена (obj.field) обслуживает set_var (обновление вложенного словаря)
        if name.contains('.') {
            self.set_var(name, value);
            return;
        }
        if let Some(old) = self.variables.get(name).cloned() {
            self.variables.insert(name.to_string(), value);
            if let Some(keys) = self.depth_keys.last_mut() { keys.push((name.to_string(), Some(old))); }
            return;
        }
        self.variables.insert(name.to_string(), value);
        if let Some(keys) = self.depth_keys.last_mut() { keys.push((name.to_string(), None)); }
    }
    pub fn try_set_var(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.constants.contains(name) { return Err(format!("Cannot modify constant '{}'", name)); }
        self.set_var(name, value);
        Ok(())
    }
    pub fn del_var(&mut self, name: &str) {
        self.variables.remove(name);
    }

    // FAST numeric registers (for bytecode hot loops)
    pub fn fast_reg(&self, idx: u16) -> f64 {
        self.fast_regs.get(idx as usize).copied().unwrap_or(0.0)
    }
    pub fn set_fast_reg(&mut self, idx: u16, v: f64) {
        let i = idx as usize;
        if self.fast_regs.len() <= i { self.fast_regs.resize(i + 1, 0.0); }
        self.fast_regs[i] = v;
    }
    pub fn add_fast_reg(&mut self, idx: u16, n: f64) {
        let i = idx as usize;
        if self.fast_regs.len() <= i {
            self.fast_regs.resize(i + 1, 0.0);
            self.fast_regs[i] = n;
        } else {
            self.fast_regs[i] += n;
        }
    }
    pub fn mul_fast_reg(&mut self, idx: u16, n: f64) {
        let i = idx as usize;
        if self.fast_regs.len() <= i {
            self.fast_regs.resize(i + 1, 0.0);
            self.fast_regs[i] = 0.0;
        } else {
            self.fast_regs[i] *= n;
        }
    }
    pub fn div_fast_reg(&mut self, idx: u16, n: f64) {
        let i = idx as usize;
        if self.fast_regs.len() <= i {
            self.fast_regs.resize(i + 1, 0.0);
            self.fast_regs[i] = 0.0;
        } else {
            self.fast_regs[i] /= n;
        }
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.scope_depth += 1;
        self.depth_keys.push(Vec::new());
    }
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            if let Some(keys) = self.depth_keys.pop() {
                // ВОССТАНАВЛИВАЕМ предыдущие значения вместо удаления
                // (name, Some(old)), старое значение; (name, None), переменная создана локально
                for (k, old) in keys {
                    match old {
                        Some(v) => { self.variables.insert(k, v); }
                        None => { self.variables.remove(&k); }
                    }
                }
            }
            if self.scope_depth > 0 { self.scope_depth -= 1; }
            self.scopes.pop();
        }
    }

    // строковые утилиты
    pub fn tq(s: &str) -> String {
        let t = s.trim();
        if t.len() >= 2 {
            let b = t.as_bytes();
            if (b[0] == b'"' && b[t.len()-1] == b'"') || (b[0] == b'\'' && b[t.len()-1] == b'\'') || (b[0] == b'`' && b[t.len()-1] == b'`') {
                return t[1..t.len()-1].to_string();
            }
        }
        t.to_string()
    }

    pub fn eq(s: &str) -> Option<String> {
        let t = s.trim();
        if Self::is_quoted_literal(t) {
            return Some(t[1..t.len()-1].to_string());
        }
        None
    }

    // Проверка, что строка, чистый строковый литерал ("abc"), а не "a" + b + "c"
    pub fn is_quoted_literal(t: &str) -> bool {
        let b = t.as_bytes();
        if b.len() < 2 { return false; }
        let q = b[0];
        if q != b'"' && q != b'\'' && q != b'`' { return false; }
        if b[b.len()-1] != q { return false; }
        let mut i = 1;
        while i < b.len() - 1 {
            if b[i] == q && b[i-1] != b'\\' { return false; }
            i += 1;
        }
        true
    }

    pub fn sic(s: &str) -> &str {
        // Быстрый путь, нет комментариев, возвращаем как есть
        if !s.contains("//") && !s.contains('#') { return s; }
        // Обрезаем комментарии // и #, если они вне кавычек
        let bytes = s.as_bytes();
        let mut q: Option<char> = None;
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i] as char;
            match q {
                Some(qc) => { if c == qc { q = None; } }
                None => {
                    if c == '"' || c == '\'' || c == '`' { q = Some(c); }
                    else if c == '/' && i+1 < bytes.len() && bytes[i+1] == b'/' { return s[..i].trim_end(); }
                    else if c == '#' && (i == 0 || bytes[i-1] == b' ' || bytes[i-1] == b'\t') { return s[..i].trim_end(); }
                }
            }
            i += 1;
        }
        s
    }

    pub fn interp(&self, s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut name = String::new();
                let mut found = false;
                loop {
                    match chars.next() {
                        Some('}') => { found = true; break; }
                        Some(c) => name.push(c),
                        None => break,
                    }
                }
                if found && !name.is_empty() {
                    if let Some(v) = self.get_var(&name) { result.push_str(&v.display()); }
                    else { result.push('{'); result.push_str(&name); result.push('}'); }
                } else { result.push('{'); result.push_str(&name); }
            } else { result.push(ch); }
        }
        result
    }

    pub fn split_top_level(text: &str, delim: char) -> Vec<String> {
        let mut parts = Vec::new();
        let mut cur = String::new();
        let mut depth = 0i32;
        let mut q: Option<char> = None;
        let mut pipe = false;
        let mut prev = '\0';
        for ch in text.chars() {
            if let Some(qc) = q {
                cur.push(ch);
                if ch == qc && prev != '\\' { q = None; }
                prev = ch;
                continue;
            }
            if pipe {
                cur.push(ch);
                if ch == '|' { pipe = false; }
                prev = ch;
                continue;
            }
            match ch {
                '"' | '\'' | '`' => { q = Some(ch); cur.push(ch); }
                '|' => { pipe = true; cur.push(ch); }
                '(' | '[' | '{' => { depth += 1; cur.push(ch); }
                ')' | ']' | '}' => { depth -= 1; cur.push(ch); }
                _ if ch == delim && depth == 0 => { parts.push(cur.trim().to_string()); cur.clear(); }
                _ => cur.push(ch),
            }
            prev = ch;
        }
        if !cur.trim().is_empty() { parts.push(cur.trim().to_string()); }
        parts
    }

    pub fn sft(text: &str, delim: char) -> Option<(String, String)> {
        let mut depth = 0i32;
        let mut q: Option<char> = None;
        let mut pipe = false;
        let mut prev = '\0';
        for (i, ch) in text.char_indices() {
            if let Some(qc) = q {
                if ch == qc && prev != '\\' { q = None; }
                prev = ch;
                continue;
            }
            if pipe {
                if ch == '|' { pipe = false; }
                prev = ch;
                continue;
            }
            match ch {
                '"' | '\'' | '`' => q = Some(ch),
                '|' => pipe = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ if ch == delim && depth == 0 => {
                    // для '=' не режем составные операторы сравнения/стрелку
                    // ==, !=, >=, <=, =>.
                    if delim == '=' {
                        let next = text[i + ch.len_utf8()..].chars().next();
                        let in_comp = prev == '!' || prev == '>' || prev == '<' || prev == '='
                            || next == Some('=') || next == Some('>');
                        if in_comp { prev = ch; continue; }
                    }
                    return Some((text[..i].trim().to_string(), text[i+ch.len_utf8()..].trim().to_string()));
                }
                _ => {}
            }
            prev = ch;
        }
        None
    }

    pub fn is_assignable(name: &str) -> bool {
        let n = name.trim();
        !n.is_empty() && !n.starts_with('~') && !n.contains(' ') && !n.contains('(')
            && n.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.')
    }

    /// "Мягкие" ошибки вычисления, значение, не выражение (неизвестные имена/
    /// токены, пустота), вызывающий вправе откатиться на сырой текст или
    /// run_script. ЖЁСТКИЕ (деление на ноль, индексы, ключи, арность, типы...)
    /// надо ПРОБРАСЫВАТЬ, иначе slow молча врёт (печатает сырьё вместо ошибки).
    pub fn is_soft_eval_error(e: &str) -> bool {
        e.contains("Unknown variable") || e.contains("Unknown token")
            || e.contains("Empty expression") || e.contains("No result")
            || e.contains("Missing operand") || e.contains("not numeric")
            || e.contains("Unknown command") || e.contains("Unknown function")
            || e.contains("Too many nested calls")
            // Сигналы "это не вызов min ws-expression, иди дальше"
            // хвост eval_func_expr и пустая работа после подстановок.
            || e.contains("Not numeric") || e == "Empty"
    }

    pub fn eval_value(&mut self, expr: &str) -> Result<Value, String> {
        let expr = expr.trim();
        if expr.is_empty() { return Ok(Value::None); }
        // Нормализация новых имён команд (без ~) в старые для резолверов
        if !expr.starts_with('~') && !expr.starts_with("kg.") {
            let translated = crate::main::eval::translate_alias(expr);
            if translated != expr {
                return crate::main::eval::evaluate(self, &translated);
            }
        }
        if let Some(s) = Self::eq(expr) { return Ok(Value::Text(self.interp(&s))); }
        // inline if как выражение, if c then A elif c2 then B else C end
        if expr.starts_with("if ") {
            if let Some((cond, rest)) = Self::parse_if(expr) {
                return self.eval_inline_if(&cond, &rest);
            }
        }
        // Быстрый путь, простое числовое/переменное выражение без спец-конструкций
        if !expr.contains('"') && !expr.contains('\'') && !expr.contains('`')
            && !expr.contains('[') && !expr.contains('(') && !expr.contains('|')
            && !expr.contains("=>") && !expr.contains("==") && !expr.contains("!=")
            && !expr.contains(">") && !expr.contains("<")
        {
            // чистое имя переменной > возвращаем значение с типом
            if expr.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                if let Some(v) = self.get_var(expr) { return Ok(v); }
                if let Ok(n) = expr.parse::<f64>() { return Ok(Value::Number(n)); }
            } else {
                // Жёсткие арифметические ошибки не маскируем в сырой текст
                // иначе slow молча печатает "1/0" вместо ошибки (паритет с fast).
                match self.eval_expr(expr) {
                    Ok(n) => return Ok(Value::Number(n)),
                    Err(e) if e == "Division by zero" || e == "Modulo by zero" => return Err(e),
                    Err(_) => {}
                }
            }
        }
        // строковый repeat "ha" * 3
        if let Some((s_val, n_str)) = expr.rsplit_once(" * ") {
            if let Some(text) = Self::tq_opt(s_val) {
                if let Ok(n) = n_str.trim().parse::<usize>() {
                    if n <= 1000 { return Ok(Value::Text(text.repeat(n))); }
                }
            }
        }
        // строковая конкатенация "a" + "b" + a[0] + x.
        // Но если НИ ОДНА часть, не литерал (все вычислились, вызовы/переменные)
        // и все результаты, числа, это арифметика (паритет с байткод-Add)
        // `dget(d, "a") + dget(d, "b")` = 15, а не "510".
        if expr.contains(" + ") && (expr.contains('"') || expr.contains('[')) {
            let parts: Vec<&str> = expr.split(" + ").collect();
            if parts.len() >= 2 {
                let mut result = String::new();
                let mut ok = true;
                let mut any_literal = false;
                let mut vals: Vec<Value> = Vec::with_capacity(parts.len());
                for p in &parts {
                    let p = p.trim();
                    if let Some(s2) = Self::tq_opt(p) {
                        any_literal = true;
                        result.push_str(&self.interp(&s2));
                        vals.push(Value::Text(s2));
                    }
                    else if let Ok(v) = self.eval_value(p) {
                        result.push_str(&v.display());
                        vals.push(v);
                    }
                    else { ok = false; break; }
                }
                if ok {
                    if !any_literal {
                        let nums: Vec<f64> = vals.iter().filter_map(|v| v.as_number()).collect();
                        if nums.len() == vals.len() {
                            return Ok(Value::Number(nums.iter().sum()));
                        }
                    }
                    return Ok(Value::Text(result));
                }
            }
        }
        // список-литерал [a, b, c] и пустой []
        if expr.starts_with('[') && expr.ends_with(']') {
            let inner = &expr[1..expr.len()-1];
            if inner.trim().is_empty() {
                return Ok(Value::List(vec![]));
            }
            let items: Vec<Value> = VM::split_top_level(inner, ',').into_iter()
                .map(|s| self.eval_value(s.trim()).unwrap_or(Value::None)).collect();
            return Ok(Value::List(items));
        }
        // индексация a[0] и срезы a[0, 3] в выражениях
        if let Some(open) = expr.rfind('[') {
            if expr.ends_with(']') && !open.to_string().contains(' ') && expr[..open].trim().chars().all(|c| c.is_alphanumeric() || c == '_') {
                let name = expr[..open].trim().to_string();
                let idx_str = expr[open+1..expr.len()-1].trim();
                if !idx_str.contains(' ') {
                    if let Some(val) = self.get_var(&name) {
                        if let Some((start_s, end_s)) = idx_str.split_once(':') {
                            let a = self.eval_expr(start_s.trim()).unwrap_or_else(|_| self.eval_value(start_s.trim()).ok().and_then(|v| v.as_number()).unwrap_or(0.0)) as isize;
                            let b = if end_s.trim().is_empty() { isize::MAX } else { self.eval_expr(end_s.trim()).unwrap_or_else(|_| self.eval_value(end_s.trim()).ok().and_then(|v| v.as_number()).unwrap_or(isize::MAX as f64)) as isize };
                            return match val {
                                Value::List(l) => {
                                    let len = l.len() as isize;
                                    let a = (if a < 0 { len + a } else { a }).max(0).min(len) as usize;
                                    let b = (if b < 0 { len + b } else { b }).max(0).min(len) as usize;
                                    if a >= b { Ok(Value::List(vec![])) }
                                    else { Ok(Value::List(l[a..b].to_vec())) }
                                }
                                Value::Text(t) => {
                                    let chars: Vec<char> = t.chars().collect();
                                    let len = chars.len() as isize;
                                    let a = (if a < 0 { len + a } else { a }).max(0).min(len) as usize;
                                    let b = (if b < 0 { len + b } else { b }).max(0).min(len) as usize;
                                    if a >= b { Ok(Value::Text(String::new())) }
                                    else { Ok(Value::Text(chars[a..b].iter().collect())) }
                                }
                                _ => Ok(val)
                            };
                        } else if let Ok(idx) = idx_str.parse::<isize>().or_else(|_| self.eval_expr(idx_str).map(|v| v as isize)) {
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
                                _ => Ok(val)
                            };
                        }
                    }
                }
            }
        }
        // строковая конкатенация "hello" + a + "world"
        if expr == "true" { return Ok(Value::Bool(true)); }
        if expr == "false" { return Ok(Value::Bool(false)); }
        if expr == "none" { return Ok(Value::None); }
        if expr.starts_with('~') || expr.starts_with("kg.") {
            return crate::main::eval::evaluate(self, expr);
        }
        // лямбда-синтаксис, lambda(x) => x*2 и fn(x) => x*2 и |x| x*2
        if let Some(eq_idx) = expr.find("=>") {
            if let Some(open) = expr.find('(') {
                if open < eq_idx && expr[..open].trim() == "lambda" || (expr[..open].trim() == "fn" && open < eq_idx) {
                    if let Some(close) = expr[open..eq_idx].find(')') {
                        let params_str = expr[open+1..open+close].trim();
                        let body = expr[eq_idx+2..].trim();
                        let params: Vec<String> = if params_str.is_empty() { vec![] } else { VM::split_top_level(params_str, ',').into_iter().map(|p| p.trim().to_string()).collect() };
                        return Ok(self.make_lambda(params, body.to_string()));
                    }
                }
            }
        }
        // короткая лямбда, |x| x*2 или |a, b| a+b
        if expr.starts_with('|') {
            if let Some(close) = expr[1..].find('|').map(|i| i + 1) {
                if close > 0 {
                    let params_str = expr[1..close].trim();
                    let body = expr[close+1..].trim();
                    if !body.is_empty() {
                        let params: Vec<String> = if params_str.is_empty() { vec![] } else { VM::split_top_level(params_str, ',').into_iter().map(|p| p.trim().to_string()).collect() };
                        return Ok(self.make_lambda(params, body.to_string()));
                    }
                }
            }
        }
        // кортеж, (1, "a", true) > список
        if expr.starts_with('(') && expr.ends_with(')') {
            let inner = &expr[1..expr.len()-1];
            if VM::split_top_level(inner, ',').len() >= 2 {
                let items: Vec<Value> = VM::split_top_level(inner, ',').into_iter()
                    .map(|s| self.eval_value(s.trim()).unwrap_or(Value::None)).collect();
                return Ok(Value::List(items));
            }
        }
        // метод-синтаксис на сложных ресиверах, [1, 2].max(), 3.5.round(), (-5).abs(), "s".upper()
        if let Some(dot) = expr.rfind('.') {
            let after = &expr[dot+1..];
            if let Some(pl) = after.find('(') {
                let method = &after[..pl];
                if expr.ends_with(')') && !method.is_empty()
                    && method.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let rc = &expr[..dot];
                    let is_simple = !rc.is_empty() && rc.split('.').all(|seg|
                        !seg.is_empty() && seg.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
                        && seg.chars().all(|c| c.is_alphanumeric() || c == '_'));
                    if !is_simple {
                        if let Some(cmd) = Self::resolve_method(method) {
                            let after_len = after.len();
                            let args = after[pl+1..after_len-1].trim();
                            // сложный ресивер > во временную переменную (команды работают по имени)
                            let tmp = format!("__m{}", self.method_tmp);
                            self.method_tmp += 1;
                            let rc_val = self.eval_value(rc)?;
                            self.set_var(&tmp, rc_val);
                            let rewritten = if args.is_empty() {
                                format!("~{}({})", cmd, tmp)
                            } else {
                                format!("~{}({}, {})", cmd, tmp, args)
                            };
                            return crate::main::eval::evaluate(self, &rewritten);
                        }
                    }
                }
            }
        }
        // вызов лямбды или функции, f(5) или map(x, f)
        if let Some(open) = expr.find('(') {
            if expr.ends_with(')') {
                let name = expr[..open].trim();
                let args_str = expr[open+1..expr.len()-1].trim();
                if !name.contains(' ') && !name.contains('(') {
                    // метод-синтаксис, obj.method(args) > ~cmd(obj, args)
                    if self.get_var(name).is_none() && !self.functions.contains_key(name) {
                        if let Some(rewritten) = Self::method_rewrite(name, args_str) {
                            return crate::main::eval::evaluate(self, &rewritten);
                        }
                    }
                    // проверяем сбалансированность скобок (чтобы не захватить fib(n-1)+fib(n-2))
                    let mut depth = 0i32;
                    let mut balanced = true;
                    for ch in args_str.chars() {
                        if ch == '(' { depth += 1; }
                        if ch == ')' { depth -= 1; if depth < 0 { balanced = false; break; } }
                    }
                    if balanced && depth == 0 {
                        // Аргументы вычисляем ЛЕНИВО, только для лямбд/функций.
                        // Eager-парсинг здесь выполнял сайд-эффекты (print, IO)
                        // а потом команда/фолбэк вычисляли их ВТОРОЙ раз
                        // (напр. `t = thread(print("hi"))` печатало дважды).
                        let is_lambda = matches!(self.get_var(name), Some(Value::Lambda(..)));
                        let is_func = self.functions.contains_key(name);
                        if !is_lambda && !is_func {
                            // не функция, дальше auto-~ / eval_func_expr / raw
                        } else {
                            let specs = self.parse_arg_specs(args_str)?;
                            let pos_args: Vec<Value> = specs.iter().filter_map(|(n, v)| if n.is_none() { Some(v.clone()) } else { None }).collect();
                            if let Some(Value::Lambda(params, body, cap)) = self.get_var(name) {
                                return self.call_lambda(params, body, pos_args, cap);
                            }
                            if let Some(func) = self.functions.get(name).cloned() {
                                self.call_stack.push(name.to_string());
                                self.push_scope();
                                self.bind_named_params(&func.params, &specs)?;
                                let result = if func.body.contains("return") || func.body.contains('\n') {
                                    match self.run_script(&func.body) {
                                        Ok(v) => v,
                                        Err(e) if e.starts_with("__return__:") => crate::main::dtam::decode_value(&e[11..]).unwrap_or(Value::None),
                                        Err(e) => { self.pop_scope(); self.call_stack.pop(); return Err(e); }
                                    }
                                } else {
                                    match self.eval_value(&func.body) {
                                        Ok(v) => v,
                                        Err(e) if !Self::is_soft_eval_error(&e) => {
                                            self.pop_scope(); self.call_stack.pop(); return Err(e);
                                        }
                                        Err(_) => {
                                            match self.run_script(&func.body) {
                                                Ok(v) => v,
                                                Err(e) if e.starts_with("__return__:") => crate::main::dtam::decode_value(&e[11..]).unwrap_or(Value::None),
                                                Err(e) => { self.pop_scope(); self.call_stack.pop(); return Err(e); }
                                            }
                                        }
                                    }
                                };
                                self.pop_scope();
                                self.call_stack.pop();
                                return Ok(result);
                            }
                        }
                    }
                }
            }
        }
        // сравнения в выражениях, v > 2, a == b, x != y, (a > b)
        let cmp_expr = if expr.starts_with('(') && expr.ends_with(')') { expr[1..expr.len()-1].trim() } else { expr };
        let has_shifts = cmp_expr.contains("<<") || cmp_expr.contains(">>");
            for op in ["==", "!=", ">=", "<=", ">", "<"] {
                if has_shifts && (op == "<" || op == ">") { continue; }
                if cmp_expr.contains(op) {
                    if let Ok(b) = self.eval_cond(cmp_expr) { return Ok(Value::Bool(b)); }
                }
            }
        match self.eval_expr(expr) {
            Ok(n) => return Ok(Value::Number(n)),
            Err(e) if e == "Division by zero" || e == "Modulo by zero" => return Err(e),
            Err(_) => {}
        }
        if let Some(v) = self.get_var(expr) { return Ok(v); }
        // Авто-~ для команд в выражениях, get(x) > ~get(x), win_key("left") > ~win_key("left")
        if expr.contains('(') && expr.ends_with(')') {
            match crate::main::eval::evaluate(self, &format!("~{}", expr)) {
                Ok(v) => return Ok(v),
                // Пробрасываем ошибки известных хендлеров (арность, типы, ключи...).
                // Глотаем только, Unknown command (сырой текст) и Function'*'
                // `call f(x)` по переменной-лямбде ~call не видит (ищет в functions)
                // значение даёт поздний путь (лямбды/eval_func_expr).
                Err(e) if !e.contains("Unknown command") && !e.starts_with("Function '") => return Err(e),
                Err(_) => {}
            }
        }
        // выражения с вызовами функций, fib(n-1) + fib(n-2).
        // Промах "это не вызов", идём дальше к конкату/сырому тексту.
        // Реальные ошибки (Unknown command для явного `call`, деление в теле
        // функции...), наружу, иначе slow молча печатает сырьё.
        // NB, здесь НЕ is_soft_eval_error, "Unknown command" из eval_func_expr
        // жёсткий (вызов найден, цели нет), а не "может, не вызов".
        if expr.contains('(') {
            match self.eval_func_expr(expr) {
                Ok(v) => return Ok(v),
                Err(e) if e == "Unknown function" || e == "Not numeric" || e == "Empty"
                    || e.contains("Too many nested calls") => {}
                Err(e) => return Err(e),
            }
        }
        // общая конкатенация через " + " (зеркало байткод-Add), только то, что иначе
        // ушло бы в сырой текст ниже. Числа уже обработаны выше (as_number-коэрсия)
        // сравнения и вызовы, тоже, так что здесь безопасно собирать display.
        // Пропускаем выражения с операторами сравнения/лямбдами, чтобы их не разрезать.
        if expr.contains(" + ")
            && !expr.contains("==") && !expr.contains("!=")
            && !expr.contains(">=") && !expr.contains("<=")
            && !expr.contains('>') && !expr.contains('<')
            && !expr.contains("=>")
        {
            let parts: Vec<&str> = expr.split(" + ").collect();
            if parts.len() >= 2 {
                let mut vals = Vec::with_capacity(parts.len());
                let mut ok = true;
                for p in &parts {
                    match self.eval_value(p.trim()) {
                        Ok(v) => vals.push(v),
                        Err(_) => { ok = false; break; }
                    }
                }
                if ok {
                    // как в байткод-Add, все числа > сумма, иначе конкатенация
                    let nums: Vec<f64> = vals.iter().filter_map(|v| v.as_number()).collect();
                    if nums.len() == vals.len() {
                        return Ok(Value::Number(nums.iter().sum()));
                    }
                    let mut result = String::new();
                    for v in &vals {
                        match v {
                            Value::Text(t) => result.push_str(&Self::tq(t)),
                            _ => result.push_str(&v.display()),
                        }
                    }
                    return Ok(Value::Text(result));
                }
            }
        }
        Ok(Value::Text(expr.to_string()))
    }

    // вычисляет выражения, содержащие вызовы функций/лямбд, fib(10) * 2 + g(x)
    pub fn eval_func_expr(&mut self, expr: &str) -> Result<Value, String> {
        let expr_t = expr.trim();
        // выражение-ветвь if/then/else, сначала выбрать ветку (иначе находит вызовы
        // в НЕвычисляемых ветках и рекурсия не уменьшает аргументы, бесконечный цикл)
        if expr_t.starts_with("if ") || expr_t.starts_with("~if ") {
            return crate::main::eval::evaluate(self, expr_t);
        }
        let mut work = expr.trim().to_string();
        let mut guard = 0usize;
        loop {
            guard += 1;
            if guard > 1000 { return Err("Too many nested calls".to_string()); }
            let chars: Vec<char> = work.chars().collect();
            let mut found = false;
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == '(' && i > 0 && (chars[i-1].is_alphanumeric() || chars[i-1] == '_') {
                    let mut j = i - 1;
                    while j > 0 && (chars[j-1].is_alphanumeric() || chars[j-1] == '_' || chars[j-1] == '.') { j -= 1; }
                    let name: String = chars[j..i].iter().collect();
                    if !name.is_empty() && !name.starts_with('~') {
                        // синтаксис `call name(...)`, захватываем и префикс `call `
                        // чтобы после подстановки не оставалось висячего `call`
                        let mut start = if j >= 5 && chars[j-5..j].iter().collect::<String>().eq_ignore_ascii_case("call ") { j - 5 } else { j };
                        // ищем закрывающую скобку
                        let mut depth = 1i32;
                        let mut k = i + 1;
                        while k < chars.len() && depth > 0 {
                            if chars[k] == '(' { depth += 1; }
                            if chars[k] == ')' { depth -= 1; }
                            k += 1;
                        }
                        if depth == 0 {
                            let args_str: String = chars[i+1..k-1].iter().collect();
                            if !args_str.contains('"') && !args_str.contains('{') {
                                // вычисляем аргументы рекурсивно
                                let args_raw: Vec<Value> = VM::split_top_level(&args_str, ',')
                                    .into_iter().map(|a| {
                                        let a = a.trim();
                                        if a.contains('(') { self.eval_func_expr(a).unwrap_or_else(|_| Value::Text(a.to_string())) }
                                        else { self.eval_value(a).unwrap_or(Value::Text(a.to_string())) }
                                    }).collect();
                                // вызываем функцию/лямбду
                                let result = if let Some(Value::Lambda(params, body, cap)) = self.get_var(&name) {
                                    self.call_lambda(params, body, args_raw, cap)
                                } else if let Some(func) = self.functions.get(&name).cloned() {
                                    if self.call_stack.len() >= 1000 {
                                        Err("recursion depth limit exceeded".to_string())
                                    } else {
                                        self.call_stack.push(name.clone());
                                        self.push_scope();
                                        self.bind_params(&func.params, &args_raw)?;
                                        let r = if func.body.contains("return") || func.body.contains('\n') {
                                            match self.run_script(&func.body) {
                                                Ok(v) => v,
                                                Err(e) if e.starts_with("__return__:") => crate::main::dtam::decode_value(&e[11..]).unwrap_or(Value::None),
                                                Err(e) => { self.pop_scope(); self.call_stack.pop(); return Err(e); }
                                            }
                                        } else {
                                            // Жёсткие ошибки тела пробрасываем (иначе вызов
                                            // молча даёт None вместо ошибки, как fast).
                                            match self.eval_value(&func.body) {
                                                Ok(v) => v,
                                                Err(e) if !Self::is_soft_eval_error(&e) => {
                                                    self.pop_scope(); self.call_stack.pop(); return Err(e);
                                                }
                                                Err(_) => Value::None,
                                            }
                                        };
                                        self.pop_scope();
                                        self.call_stack.pop();
                                        Ok(r)
                                    }
                                } else if start != j {
                                    // Явный `call name(...)`, а такой функции/лямбды нет.
                                    // Жёсткая ошибка в формате fast (иначе вызов молча
                                    // превращается в сырой текст). Без префикса `call`
                                    // оставляем мягкий промах, это может быть команда
                                    // (len/print/...), её разберут поздние пути.
                                    // Имя/аргументы нормализуем как fast (алиас fib >
                                    // fibonacci, запятые через ", "), иначе текст
                                    // расходится при побайтовом сравнении.
                                    let norm_args = VM::split_top_level(args_str.as_str(), ',').iter()
                                        .map(|s| s.trim().to_string()).collect::<Vec<_>>().join(", ");
                                    let shown = crate::main::eval::translate_alias(
                                        &format!("{}({})", name, norm_args));
                                    let shown = shown.trim_start_matches('~');
                                    return Err(format!("\x1B[91mUnknown command: {}\x1B[0m", shown));
                                } else {
                                    Err("Unknown function".to_string())
                                };
                                if let Ok(v) = result {
                                    // замена вызова результатом (включая префикс `call `)
                                    let replacement = if let Some(n) = v.as_number() { Value::fmt_num(n) } else { v.display() };
                                    let mut new_work = String::new();
                                    new_work.push_str(&work[..start]);
                                    new_work.push_str(&replacement);
                                    new_work.push_str(&work[k..]);
                                    work = new_work;
                                    found = true;
                                    break;
                                } else if let Err(e) = result {
                                    // Вызов найден, исполнение упало (тело лямбды/функции
                                    // арность, рекурсия), пробрасываем, иначе ошибка
                                    // маскируется в сырой текст. Исключение, "Unknown
                                    // function", это не вызов, а имя команды/текст
                                    // его разбирают поздние пути (auto-~).
                                    if e != "Unknown function" { return Err(e); }
                                }
                            }
                        }
                    }
                }
                i += 1;
            }
            if !found { break; }
        }
        // после подстановок пробуем как число
        let work_trim = work.trim();
        if work_trim.is_empty() { return Err("Empty".to_string()); }
        if work_trim.starts_with('"') { return Ok(Value::Text(work_trim.trim_matches('"').to_string())); }
        match self.eval_expr(&work_trim) {
            Ok(n) => Ok(Value::Number(n)),
            Err(_) => Err("Not numeric".to_string()),
        }
    }

    // eval_expr с компиляцией и кешированием
    pub fn eval_expr(&self, expr: &str) -> Result<f64, String> {
        let expr = expr.trim();
        if expr.is_empty() { return Err("Empty expression".to_string()); }

        // Быстрый путь, число или переменная
        if let Ok(n) = expr.parse::<f64>() { return Ok(n); }
        if let Some(v) = self.get_var(expr) { if let Some(n) = v.as_number() { return Ok(n); } }

        // Проверка кеша
        if let Some(ops) = self.expr_cache.borrow().get(expr) {
            return self.exec_expr_ops(ops);
        }

        // Компиляция и кеширование
        match self.compile_expr(expr) {
            Ok(ops) => {
                if self.expr_cache.borrow().len() < 512 {
                    self.expr_cache.borrow_mut().insert(expr.to_string(), ops.clone());
                }
                self.exec_expr_ops(&ops)
            }
            Err(e) => Err(e),
        }
    }

    // разбор параметра функции, "x=5" > ("x", Some(5)); "y" > ("y", None)
    pub fn parse_param(&mut self, spec: &str) -> (String, Option<Value>) {
        let spec = spec.trim();
        if let Some((name, default)) = spec.split_once('=') {
            let name = name.trim().to_string();
            let val = match self.eval_value(default.trim()) {
                Ok(v) => Some(v),
                Err(_) => Some(Value::Text(default.trim().to_string())),
            };
            (name, val)
        } else {
            (spec.to_string(), None)
        }
    }

    // связывание аргументов с параметрами (с поддержкой default args)
    pub fn bind_params(&mut self, params: &[String], args: &[Value]) -> Result<(), String> {
        for (i, p) in params.iter().enumerate() {
            let (name, default) = self.parse_param(p);
            let val = if i < args.len() { args[i].clone() } else {
                match default { Some(d) => d, None => Value::None }
            };
            self.set_var_local(&name, val);
        }
        Ok(())
    }

    // Разбор аргументов с поддержкой именованных, f(5, y=2) > [(None, 5), (Some(y), 2)]
    pub fn parse_arg_specs(&mut self, args_str: &str) -> Result<Vec<(Option<String>, Value)>, String> {
        let mut out = Vec::new();
        let args_str = args_str.trim();
        if args_str.is_empty() { return Ok(out); }
        for part in VM::split_top_level(args_str, ',') {
            let part = part.trim();
            let mut name = None;
            let mut expr = part.to_string();
            if let Some((n, v)) = VM::sft(&expr, '=') {
                let n = n.trim();
                if VM::is_assignable(n) { name = Some(n.to_string()); expr = v.trim().to_string(); }
            }
            let val = self.eval_value(&expr)?;
            out.push((name, val));
        }
        Ok(out)
    }

    // Связывание с поддержкой именованных аргументов
    pub fn bind_named_params(&mut self, params: &[String], args: &[(Option<String>, Value)]) -> Result<(), String> {
        let mut named: HashMap<String, Value> = HashMap::new();
        let mut pos: Vec<Value> = Vec::new();
        for (n, v) in args {
            match n {
                Some(k) => { named.insert(k.clone(), v.clone()); }
                None => pos.push(v.clone()),
            }
        }
        let mut p = 0;
        for spec in params {
            let (name, default) = self.parse_param(spec);
            let val = if let Some(v) = named.get(&name) { v.clone() }
                else if p < pos.len() { let v = pos[p].clone(); p += 1; v }
                else { match default { Some(d) => d, None => Value::None } };
            self.set_var_local(&name, val);
        }
        Ok(())
    }

    pub fn eval_cond(&mut self, cond: &str) -> Result<bool, String> {
        let cond = cond.trim();
        // литеральные true/false (иначе "true" уходит в eval_expr/get_var > false)
        if cond == "true" { return Ok(true); }
        if cond == "false" { return Ok(false); }
        // скобки вокруг всего условия, (len(x)), ((a > b)), снимаем и повторяем.
        // Проверка баланса, "(a) + (b)" не обёрнуто (глубина падает раньше конца).
        if cond.starts_with('(') && cond.ends_with(')') && cond.len() >= 2 {
            let mut d = 0i32;
            let mut wraps = true;
            for c in cond[1..cond.len() - 1].chars() {
                match c { '(' => d += 1, ')' => { d -= 1; if d < 0 { wraps = false; break; } }, _ => {} }
            }
            if wraps && d == 0 {
                return self.eval_cond(&cond[1..cond.len() - 1]);
            }
        }
        if let Some(r) = cond.strip_prefix("not ") { return Ok(!self.eval_cond(r.trim())?); }
        for sep in [" and ", " && "] {
            if let Some((a, b)) = cond.split_once(sep) { return Ok(self.eval_cond(a.trim())? && self.eval_cond(b.trim())?); }
        }
        for sep in [" or ", " || "] {
            if let Some((a, b)) = cond.split_once(sep) { return Ok(self.eval_cond(a.trim())? || self.eval_cond(b.trim())?); }
        }
        for op in ["==", "!=", ">=", "<=", ">", "<"] {
            if let Some((a, b)) = cond.split_once(op) {
                let (la, rb) = (a.trim(), b.trim());
                // число через eval_expr (арифметика) или eval_value (команды, len(x), ceil(x), ...).
                // Жёсткие ошибки операндов (деление на ноль и т.п.) пробрасываем
                // иначе `if 1/0 == 2` молча false, хотя fast ошибается.
                // (Без замыкания, оно заняло бы &mut self и не дало передать self.)
                let lv = match self.eval_expr(la) {
                    Ok(n) => Some(n),
                    Err(_) => match self.eval_value(la) {
                        Ok(v) => v.as_number(),
                        Err(e) if !Self::is_soft_eval_error(&e) => return Err(e),
                        Err(_) => None,
                    },
                };
                let rv = match self.eval_expr(rb) {
                    Ok(n) => Some(n),
                    Err(_) => match self.eval_value(rb) {
                        Ok(v) => v.as_number(),
                        Err(e) if !Self::is_soft_eval_error(&e) => return Err(e),
                        Err(_) => None,
                    },
                };
                if let (Some(lv), Some(rv)) = (lv, rv) {
                    return Ok(match op { "==" => lv == rv, "!=" => lv != rv, ">=" => lv >= rv, "<=" => lv <= rv, ">" => lv > rv, "<" => lv < rv, _ => false });
                }
                let lt = match self.get_var(la) { Some(v) => v.as_text().unwrap_or_else(|| v.display()), None => Self::tq(la) };
                let rt = match self.get_var(rb) { Some(v) => v.as_text().unwrap_or_else(|| v.display()), None => Self::tq(rb) };
                return Ok(match op { "==" => lt == rt, "!=" => lt != rt, _ => false });
            }
        }
        if let Some(v) = self.get_var(cond) { return Ok(v.truthy()); }
        if let Ok(n) = self.eval_expr(cond) { return Ok(n != 0.0); }
        // Вызовы/команды в условии, while call c(i) do, if len(x) then.
        // Без этого, молча false (цикл ни разу не выполняется), хотя fast работает.
        if cond.contains('(') {
            // Сначала как вызов функции/лямбды (аргументы вычисляются, как у fast).
            match self.eval_func_expr(cond) {
                Ok(v) => return Ok(v.truthy()),
                Err(e) if e == "Unknown function" || e == "Not numeric" || e == "Empty"
                    || e.contains("Too many nested calls") => {}
                Err(e) => return Err(e),
            }
            // Затем как команда (len(x), ceil(x), ...) через eval_value.
            // Невычисленный вызов (сырой текст = исходник), ошибка, а не false
            // иначе `while call nosuch(i)` молча пропускается вместо ошибки.
            match self.eval_value(cond) {
                Ok(Value::Text(t)) if t == cond => {
                    let shown = cond.strip_prefix("call ").unwrap_or(cond);
                    return Err(format!("Unknown command: {}", shown));
                }
                Ok(v) => return Ok(v.truthy()),
                Err(e) => return Err(e),
            }
        }
        Ok(false)
    }

    pub fn parse_if(line: &str) -> Option<(String, String)> {
        let s = line.trim();
        if !s.starts_with("if ") { return None; }
        let rest = &s[3..];
        if let Some(p) = rest.find(" then ") { Some((rest[..p].trim().to_string(), rest[p+6..].trim().to_string())) }
        else if rest.ends_with(" then") { Some((rest[..rest.len()-5].trim().to_string(), String::new())) }
        else { None }
    }

    // Таблица методов, obj.method(args) > ~cmd(obj, args)
    pub fn resolve_method(method: &str) -> Option<&'static str> {
        Some(match method {
            // мутирующие (по имени переменной)
            "push" => "add",
            "pop" => "pop",
            "sort" => "sort",
            "insert" => "insert",
            "remove" => "remove",
            "shift" => "list_shift",
            "unshift" => "list_unshift",
            "set" => "dset",
            "get" => "dget",
            "del" => "ddel",
            // по значению
            "len" | "size" | "length" => "len",
            "upper" => "upper",
            "lower" => "lower",
            "trim" => "trim",
            "split" => "split",
            "join" => "join",
            "replace" => "replace",
            "reverse" => "reverse",
            "concat" => "concat",
            "keys" => "keys",
            "values" => "values",
            "count" => "count",
            "first" => "first",
            "last" => "last",
            "contains" => "list_contains",
            "index" => "list_index",
            "abs" => "abs",
            "floor" => "floor",
            "ceil" => "ceil",
            "round" => "round",
            "max" => "list_max",
            "min" => "list_min",
            "sum" => "list_sum",
            "avg" | "mean" => "list_avg",
            _ => return None,
        })
    }

    // "obj.method(args)" > "~cmd(obj, args)"
    pub fn method_rewrite(name: &str, args_str: &str) -> Option<String> {
        if !name.contains('.') { return None; }
        let (obj_src, method) = name.rsplit_once('.')?;
        let cmd = Self::resolve_method(method)?;
        let args = if args_str.trim().is_empty() { obj_src.to_string() } else { format!("{}, {}", obj_src, args_str) };
        Some(format!("~{}({})", cmd, args))
    }

    // Инлайн-ветка if/elif/else, `cond` + rest ("A elif c2 then B else C").
    pub fn eval_inline_if(&mut self, head_cond: &str, rest: &str) -> Result<Value, String> {
        let rest = strip_trailing_end(rest);
        let top = Self::inline_if_token(rest);
        let run = |vm: &mut Self, part: &str| -> Result<Value, String> {
            let part = part.trim();
            if part.is_empty() { return Ok(Value::None); }
            match vm.eval_value(part) {
                Ok(v) => Ok(v),
                Err(_) => crate::main::eval::evaluate(vm, part),
            }
        };
        if self.eval_cond(head_cond.trim())? {
            let then_part = match &top { Some((i, _)) => rest[..*i].trim(), None => rest.trim() };
            return run(self, then_part);
        }
        match top {
            Some((i, "else")) => run(self, &rest[i+6..]),
            Some((i, "elif")) => {
                let tail = rest[i+6..].trim();
                if let Some(p) = tail.find(" then ") {
                    let c2 = tail[..p].trim().to_string();
                    let r2 = tail[p+6..].trim().to_string();
                    self.eval_inline_if(&c2, &r2)
                } else if let Some(c2) = tail.strip_suffix(" then") {
                    self.eval_inline_if(c2.trim(), "")
                } else { Ok(Value::None) }
            }
            _ => Ok(Value::None),
        }
    }

    // Инлайн-ветка if/elif/else для STATEMENT-ов, тело исполняется сначала через
    // evaluate() (присваивания, команды), и только при "Unknown command", через
    // eval_value (чистое значение в ветке). eval_inline_if (выше), НЕ годится для
    // statement'ов, его run() сначала зовёт eval_value, а он для "x = 111" возвращает
    // Ok(None), и настоящее присваивание никогда не выполняется.
    pub fn eval_inline_if_stmt(&mut self, head_cond: &str, rest: &str) -> Result<Value, String> {
        let rest = strip_trailing_end(rest);
        let top = Self::inline_if_token(rest);
        let run = |vm: &mut Self, part: &str| -> Result<Value, String> {
            let part = part.trim();
            if part.is_empty() { return Ok(Value::None); }
            match crate::main::eval::evaluate(vm, part) {
                Ok(v) => Ok(v),
                Err(e) if e.contains("Unknown command") => vm.eval_value(part),
                Err(e) => Err(e),
            }
        };
        if self.eval_cond(head_cond.trim())? {
            let then_part = match &top { Some((i, _)) => rest[..*i].trim(), None => rest.trim() };
            return run(self, then_part);
        }
        match top {
            Some((i, "else")) => run(self, &rest[i+6..]),
            Some((i, "elif")) => {
                let tail = rest[i+6..].trim();
                if let Some(p) = tail.find(" then ") {
                    let c2 = tail[..p].trim().to_string();
                    let r2 = tail[p+6..].trim().to_string();
                    self.eval_inline_if_stmt(&c2, &r2)
                } else if let Some(c2) = tail.strip_suffix(" then") {
                    self.eval_inline_if_stmt(c2.trim(), "")
                } else { Ok(Value::None) }
            }
            _ => Ok(Value::None),
        }
    }
    // Ищет первый top-level (не в скобках/кавычках) токен " elif " или " else ".
    fn inline_if_token(rest: &str) -> Option<(usize, &'static str)> {
        let bytes = rest.as_bytes();
        let mut depth = 0i32;
        let mut q: Option<u8> = None;
        let mut i = 0usize;
        while i < bytes.len() {
            let b = bytes[i];
            if let Some(qc) = q {
                if b == qc { q = None; }
                i += 1;
                continue;
            }
            match b {
                b'"' | b'\'' | b'`' => { q = Some(b); i += 1; }
                b'(' | b'[' | b'{' => { depth += 1; i += 1; }
                b')' | b']' | b'}' => { depth -= 1; i += 1; }
                _ => {
                    if depth == 0 && (b == b' ' || b == b'\t') && i + 6 <= bytes.len() {
                        let s = &bytes[i+1..i+5];
                        if s == b"elif" || s == b"else" {
                            return Some((i, if s == b"elif" { "elif" } else { "else" }));
                        }
                    }
                    i += 1;
                }
            }
        }
        None
    }

    pub fn parse_while(line: &str) -> Option<(String, String)> {
        let s = line.trim();
        if !s.starts_with("while ") { return None; }
        let rest = &s[6..];
        if let Some(p) = rest.find(" do ") { Some((rest[..p].trim().to_string(), rest[p+4..].trim().to_string())) }
        else if rest.ends_with(" do") { Some((rest[..rest.len()-3].trim().to_string(), String::new())) }
        else { None }
    }

    pub fn parse_for(line: &str) -> Option<(String, String)> {
        let s = line.trim();
        if !s.starts_with("for ") { return None; }
        let rest = s[4..].trim();
        if let Some(p) = rest.find(" do ") { Some((rest[..p].trim().to_string(), rest[p+4..].trim().to_string())) }
        else if rest.ends_with(" do") { Some((rest[..rest.len()-3].trim().to_string(), String::new())) }
        else { None }
    }

    pub fn parse_for_head(head: &str) -> Option<(String, String)> {
        if let Some((v, s)) = head.split_once(" in ") { return Some((v.trim().to_string(), s.trim().to_string())); }
        if let Some((v, s)) = head.split_once(" from ") {
            if let Some((a, b)) = s.split_once(" to ") { return Some((v.trim().to_string(), format!("{}..{}", a.trim(), b.trim()))); }
        }
        None
    }

    pub fn block_opener(line: &str) -> bool {
        let t = line.trim();
        (t == "try" || t.starts_with("try "))
            || t == "~{" || t == "loop" || t.starts_with("~try") || t.starts_with("~case ") || t == "on_start"
            || t.starts_with("switch") && t.contains('(')
            || (t.starts_with("if ") && t.ends_with(" then"))
            || (t.starts_with("while ") && t.ends_with(" do"))
            || (t.starts_with("for ") && t.ends_with(" do"))
            || t.starts_with("text-block(") || t.starts_with("textblock(")
            || t.starts_with("~text-block(") || t.starts_with("~textblock(")
    }

    pub fn collect_block_from_stdin(first_line: String) -> Result<String, String> {
        use std::io::{self, Write};
        let is_text_block = {
            let t = first_line.trim();
            t.starts_with("text-block(") || t.starts_with("textblock(")
                || t.starts_with("~text-block(") || t.starts_with("~textblock(")
        };
        let mut lines = vec![first_line];
        let mut depth = 1;
        loop {
            print!("... ");
            let _ = io::stdout().flush();
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                return Err("Failed to read line".to_string());
            }
            let line = input.trim_end_matches(['\r', '\n']).to_string();
            let trimmed = line.trim();
            if is_text_block && trimmed.ends_with(')') {
                lines.push(line);
                break;
            }
            if Self::block_opener(trimmed) {
                depth += 1;
            }
            if trimmed == "end" {
                depth -= 1;
                if depth == 0 {
                    lines.push(line);
                    break;
                }
            }
            lines.push(line);
        }
        Ok(lines.join("\n"))
    }

    pub fn eval_for_source(&mut self, source: &str) -> Result<ForSource, String> {
        let s = source.trim();
        if let Some((a, b)) = s.split_once("..") { return Ok(ForSource::Range(self.eval_expr(a.trim())? as i64, self.eval_expr(b.trim())? as i64)); }
        // генератор, вызов функции, в теле которой есть `yield` > собираем значения
        if s.ends_with(')') && s.contains('(') {
            let open = s.find('(').unwrap();
            let name = s[..open].trim();
            let is_gen = !name.contains(' ') && self.functions.contains_key(name)
                && self.functions.get(name).map(|f| f.body.contains("yield")).unwrap_or(false);
            if is_gen {
                self.gen_stack.push(Vec::new());
                let r = self.eval_value(s);
                let vals = self.gen_stack.pop().unwrap_or_default();
                r?;
                return Ok(ForSource::List(vals));
            }
        }
        match self.eval_value(s)? {
            Value::List(l) => Ok(ForSource::List(l)),
            Value::Text(t) => Ok(ForSource::List(t.split(',').map(|x| Value::Text(x.trim().to_string())).collect())),
            other => Ok(ForSource::List(vec![other])),
        }
    }

    pub fn run_script(&mut self, script: &str) -> Result<Value, String> {
        let lines: Vec<String> = script.lines().map(|l| l.trim_end_matches('\r').to_string()).collect();
        // БЫСТРЫЙ ПУТЬ, если скрипт компилируется в байткод, исполняем его
        // (без строкового парсинга на каждой итерации). Иначе, старый интерпретатор.
        // Медленный путь, если TREK_SLOW=1, принудительно интерпретатор
        // (для дифференциального тестирования против байткода).
        if std::env::var("TREK_SLOW").is_err() {
            if let Some(code) = crate::main::bytecode::compile(&lines) {
                return crate::main::bytecode::run(self, &code);
            }
        }
        self.exec_lines(&lines, 0).map(|(_, v)| v)
    }

    pub fn exec_lines(&mut self, lines: &[String], start: usize) -> Result<(usize, Value), String> {
        let mut labels: HashMap<String, usize> = HashMap::new();
        let has_labels = lines.iter().any(|l| { let t = l.trim(); t.starts_with(':') && t.len() > 1 && !t[1..].contains(' ') });
        if has_labels {
            for (i, line) in lines.iter().enumerate() {
                let t = line.trim();
                if t.starts_with(':') && t.len() > 1 && !t[1..].contains(' ') { labels.insert(t[1..].to_string(), i); }
            }
        }
        let mut idx = start;
        let mut last = Value::None;
        let mut goto_guard = 0usize;

        while idx < lines.len() {
            self.script_line = start + idx + 1;
            let line = lines[idx].trim();
            if line.is_empty() || line.starts_with('#') { idx += 1; continue; }
            if line == "end" || line == "else" || line == "}~" || line == "~catch" { break; }
            if line.starts_with(':') && !line[1..].contains(' ') { idx += 1; continue; }

            if line.starts_with("~goto ") {
                let label = line[6..].trim();
                goto_guard += 1;
                if goto_guard > 100_000 { return Err("~goto limit exceeded".to_string()); }
                if let Some(&t) = labels.get(label) { idx = t + 1; continue; }
                return Err(format!("Label '{}' not found", label));
            }

            if line == "on_start" {
                idx += 1;
                let (ni, body) = self.collect_simple_block(lines, idx)?;
                idx = ni;
                if !body.trim().is_empty() { last = self.run_script(&body)?; }
                continue;
            }

            if line == "~{" {
                idx += 1;
                let mut body = Vec::new();
                let mut depth = 1i32;
                while idx < lines.len() {
                    let l = lines[idx].trim();
                    if l == "~{" { depth += 1; }
                    if l == "}~" {
                        depth -= 1;
                        if depth == 0 { idx += 1; break; }
                    }
                    if depth > 0 { body.push(lines[idx].clone()); }
                    idx += 1;
                }
                if !body.is_empty() { last = self.run_script(&body.join("\n"))?; }
                continue;
            }

            // fn name(params) { ... }, новый синтаксис функций
            if line == "fn" || line.starts_with("fn ") {
                let rest = line[2..].trim();
                let (fn_name, fn_params) = if let Some(open) = rest.find('(') {
                    if let Some(close) = rest.find(')') {
                        let name = rest[..open].trim().to_string();
                        let p = rest[open+1..close].trim();
                        let params: Vec<String> = if p.is_empty() { Vec::new() } else { p.split(',').map(|s| s.trim().to_string()).collect() };
                        (name, params)
                    } else { (rest.to_string(), Vec::new()) }
                } else {
                    (rest.trim_end_matches('{').trim().to_string(), Vec::new())
                };
                // Проверяем, тело инлайновое (на той же строке) или на следующих строках
                let after_params = rest.find(')').map(|c| &rest[c+1..]).unwrap_or("");
                let body_str;
                if after_params.contains('{') && after_params.contains('}') {
                    // Инлайновая форма, fn name(params) { body }. Порядок проверяем
                    // `fn f() } {` давал бы инвертированный срез и панику.
                    let open_b = after_params.find('{').unwrap();
                    let close_b = after_params.rfind('}').unwrap();
                    if close_b <= open_b { return Err("Bad fn body: misplaced braces".to_string()); }
                    body_str = after_params[open_b+1..close_b].trim().to_string();
                    idx += 1;
                } else {
                    idx += 1;
                    let mut body = Vec::new();
                    let mut depth = 1i32;
                    while idx < lines.len() {
                        let l = lines[idx].trim();
                        if l == "{" { depth += 1; }
                        if l == "}" { depth -= 1; if depth == 0 { idx += 1; break; } }
                        body.push(l.to_string());
                        idx += 1;
                    }
                    body_str = body.join("\n");
                }
                let cache = crate::main::bytecode::compile_body(&body_str, &fn_params);
                self.functions.insert(fn_name.clone(), crate::main::vm::FunctionDef { params: fn_params, body: body_str, code: cache });
                println!("Function '{}' created", fn_name);
                continue;
            }
            if line == "loop" {
                idx += 1;
                let (ni, body_lines) = self.collect_simple_lines(lines, idx)?;
                idx = ni;
                let mut iters: i64 = 0;
                loop {
                    iters += 1;
                    if iters > 200_000_000_000 { return Err("loop: iteration limit exceeded".to_string()); }
                    match self.exec_lines(&body_lines, 0) {
                        Ok((_, v)) => last = v,
                        Err(e) if e.starts_with("__break__") => break,
                        Err(e) if e.starts_with("__continue__") => continue,
                        Err(e) => return Err(e),
                    }
                }
                continue;
            }

            // Многострочный текстовый блок, text-block( ... )
            {
                let mut opener_len = None;
                for o in ["text-block(", "textblock(", "~text-block(", "~textblock("] {
                    if line.starts_with(o) { opener_len = Some(o.len()); break; }
                }
                if let Some(ol) = opener_len {
                    let rest = &line[ol..];
                    let mut body = String::new();
                    if let Some(close) = rest.rfind(')') {
                        // однострочная форма, text-block(текст)
                        body.push_str(&rest[..close]);
                        idx += 1;
                    } else {
                        body.push_str(rest);
                        idx += 1;
                        while idx < lines.len() {
                            let l = lines[idx].trim();
                            if let Some(close) = l.rfind(')') {
                                if !body.is_empty() || !l[..close].trim().is_empty() {
                                    if !body.is_empty() { body.push('\n'); }
                                    body.push_str(&l[..close]);
                                }
                                idx += 1;
                                break;
                            }
                            if !body.is_empty() { body.push('\n'); }
                            body.push_str(l);
                            idx += 1;
                        }
                    }
                    let outp = self.interp(&body);
                    println!("{}", outp);
                    last = Value::Text(outp);
                    continue;
                }
            }

            if let Some((cond, rest)) = Self::parse_if(line) {
                if rest.is_empty() {
                    idx += 1;
                    let (ni, arms) = self.collect_if_arms(lines, idx, Some(cond))?;
                    idx = ni;
                    let mut matched = false;
                    for (acond, abody) in arms {
                        if matched { continue; }
                        let take = match &acond {
                            Some(c) => self.eval_cond(c)?,
                            None => true, // else, берём, если ещё ничего не сработало
                        };
                        if take {
                            matched = true;
                            if !abody.is_empty() { last = self.exec_lines(&abody, 0).map(|(_, v)| v)?; }
                        }
                    }
                } else {
                    // STATEMENT-ветка, через _stmt-версию (evaluate-first)
                    // иначе `continue`/`break`/присваивания в ветке глотаются
                    // (expression-версия идёт через eval_value > сырой текст).
                    last = self.eval_inline_if_stmt(&cond, &rest)?;
                    idx += 1;
                }
                continue;
            }

            if let Some((cond, rest)) = Self::parse_while(line) {
                let mut witers: i64 = 0;
                if rest.is_empty() {
                    idx += 1;
                    let (ni, body_lines) = self.collect_simple_lines(lines, idx)?;
                    idx = ni;
                    while self.eval_cond(&cond)? {
                        witers += 1;
                        if witers > 200_000_000_000 { return Err("while: iteration limit exceeded".to_string()); }
                        match self.exec_lines(&body_lines, 0) {
                            Ok((_, v)) => last = v,
                            Err(e) if e.starts_with("__break__") => break,
                            Err(e) if e.starts_with("__continue__") => continue,
                            Err(e) => return Err(e),
                        }
                    }
                } else {
                    while self.eval_cond(&cond)? {
                        witers += 1;
                        if witers > 200_000_000_000 { return Err("while: iteration limit exceeded".to_string()); }
                        match crate::main::eval::evaluate(self, &rest) {
                            Ok(v) => last = v,
                            Err(e) if e.starts_with("__break__") => break,
                            Err(e) if e.starts_with("__continue__") => continue,
                            Err(e) => return Err(e),
                        }
                    }
                    idx += 1;
                }
                continue;
            }

            if let Some((head, rest)) = Self::parse_for(line) {
                let (var, src) = Self::parse_for_head(&head)
                    .ok_or_else(|| format!("Bad for header: {}", head))?;
                let source = self.eval_for_source(&src)?;
                let body_inline = !rest.is_empty();
                let body_lines: Vec<String>;
                if body_inline {
                    body_lines = vec![strip_trailing_end(&rest).to_string()];
                    idx += 1;
                } else {
                    idx += 1;
                    let (ni, bl) = self.collect_simple_lines(lines, idx)?;
                    idx = ni;
                    body_lines = bl;
                }
                match source {
                    ForSource::Range(a, b) => {
                        let step = if a <= b { 1i64 } else { -1i64 };
                        let mut i = a;
                        loop {
                            if (step > 0 && i > b) || (step < 0 && i < b) { break; }
                            self.set_var(&var, Value::Number(i as f64));
                            match self.exec_lines(&body_lines, 0) {
                                Ok((_, v)) => last = v,
                                Err(e) if e.starts_with("__break__") => break,
                                Err(e) if e.starts_with("__continue__") => { i += step; continue; }
                                Err(e) => return Err(e),
                            }
                            i += step;
                        }
                    }
                    ForSource::List(items) => {
                        for item in items {
                            self.set_var(&var, item);
                            match self.exec_lines(&body_lines, 0) {
                                Ok((_, v)) => last = v,
                                Err(e) if e.starts_with("__break__") => break,
                                Err(e) if e.starts_with("__continue__") => continue,
                                Err(e) => return Err(e),
                            }
                        }
                    }
                }
                continue;
            }

            if line.starts_with("~try") || line == "try" || line.starts_with("try ") {
                let inline = if line.starts_with("~try") { line.trim_start_matches("~try").trim().to_string() } else { line[3..].trim().to_string() };
                let (try_lines, catch_lines, err_var, ni) = if inline.is_empty() {
                    idx += 1;
                    let (ni, tl, cl, ev) = self.collect_try_lines(lines, idx)?;
                    (tl, cl, ev, ni)
                } else if let Some((t, c)) = inline.split_once("~catch").or_else(|| inline.split_once(" catch ")) {
                    let c = c.trim_start_matches('~');
                    let err_var = if let Some(as_var) = c.strip_prefix("as ") { as_var.trim().to_string() } else { String::new() };
                    let c_trimmed = if let Some((cmd, _)) = c.split_once(" end") { cmd } else { c };
                    idx += 1;
                    (vec![t.trim().to_string()], vec![c_trimmed.trim().to_string()], err_var, idx)
                } else {
                    idx += 1;
                    (vec![inline], Vec::new(), String::new(), idx)
                };
                idx = ni;
                match self.exec_lines(&try_lines, 0) {
                    Ok((_, v)) => last = v,
                    Err(e) if e.starts_with("__break__") || e.starts_with("__continue__") || e.starts_with("__return__") => return Err(e),
                    Err(e) => {
                        // try БЕЗ catch, ошибку глотать нельзя (молча продолжалось бы)
                        // пробрасываем наружу, нечего ловить. Проверка ПЕРВОЙ
                        // иначе e уедет в set_var ниже (use-after-move).
                        if catch_lines.is_empty() && err_var.is_empty() { return Err(e); }
                        if !err_var.is_empty() { self.set_var(&err_var, Value::Text(e)); }
                        if !catch_lines.is_empty() { last = self.exec_lines(&catch_lines, 0).map(|(_, v)| v)?; }
                    }
                }
                continue;
            }

            if line.starts_with("switch") {
                if let Some(open) = line.find('(') {
                    if let Some(close) = line[open..].find(')').map(|i| open + i) {
                        let subject = line[open+1..close].trim().to_string();
                        let brace_form = line[close+1..].trim() == "{";
                        idx += 1;
                        let (ni, arms) = self.collect_switch_arms(lines, idx, brace_form)?;
                        idx = ni;
                        let subj_val = self.eval_value(&subject)?;
                        let mut matched = false;
                        for (pat, body) in arms {
                            let take = if let Some(p) = &pat {
                                if matched { false } else {
                                    let pv = self.eval_value(p).unwrap_or(Value::None);
                                    format!("{:?}", pv) == format!("{:?}", subj_val)
                                        || pv.as_text() == subj_val.as_text()
                                        || (pv.as_number().is_some() && pv.as_number() == subj_val.as_number())
                                }
                            } else { !matched };
                            if take {
                                matched = true;
                                if !body.is_empty() { last = self.exec_lines(&body, 0).map(|(_, v)| v)?; }
                            }
                        }
                    }
                }
                continue;
            }

            if line.starts_with("~case ") {
                let subject = line[6..].trim().to_string();
                idx += 1;
                let (ni, arms) = self.collect_case_block(lines, idx)?;
                idx = ni;
                let subj_val = self.eval_value(&subject)?;
                let mut matched = false;
                for (pat, body) in arms {
                    if pat == "_" {
                        if !matched && !body.is_empty() {
                            last = crate::main::eval::evaluate(self, &body)?;
                        }
                        break;
                    }
                    let pval = self.eval_value(&pat)?;
                    if format!("{:?}", pval) == format!("{:?}", subj_val)
                        || pval.as_text() == subj_val.as_text()
                        || (pval.as_number().is_some() && pval.as_number() == subj_val.as_number())
                    {
                        matched = true;
                        if !body.is_empty() {
                            last = crate::main::eval::evaluate(self, &body)?;
                        }
                        break;
                    }
                }
                continue;
            }

            // yield <expr>, генераторы, сохраняет значение в буфер и продолжает
            if line == "yield" || line.starts_with("yield ") {
                let val = if line == "yield" { Value::None } else { self.eval_value(line[6..].trim())? };
                if let Some(buf) = self.gen_stack.last_mut() {
                    buf.push(val);
                } else {
                    return Err("yield outside generator".to_string());
                }
                idx += 1;
                continue;
            }

            match crate::main::eval::evaluate(self, line) {
                Ok(v) => last = v,
                Err(e) if e.starts_with("__break__") || e.starts_with("__continue__") || e.starts_with("__return__") => return Err(e),
                Err(e) => {
                    // Уже обёрнутая ошибка (вложенный import/run_script), второй
                    // префикс [file, line] не добавляем, иначе дубли как
                    // "[a.tk, 2] [a.tk, 2] Division by zero | ... | ...".
                    if e.contains("  |  ") { self.last_error = e.clone(); return Err(e); }
                    let ctx = format!("[{}:{}] {}  |  {}", self.script_name, self.script_line, e, line);
                    self.last_error = ctx.clone();
                    return Err(ctx);
                },
            }
            idx += 1;
        }
        Ok((idx, last))
    }

    // Дополнительные методы

    pub fn ansi_fg(c: &str) -> &'static str {
        match c.trim() {
            "red" => "\x1B[91m", "green" => "\x1B[92m", "blue" => "\x1B[94m",
            "yellow" => "\x1B[93m", "white" => "\x1B[97m", "black" => "\x1B[30m",
            "cyan" => "\x1B[96m", "magenta" => "\x1B[95m", "reset" => "\x1B[0m",
            _ => "\x1B[97m",
        }
    }

    pub fn ansi_bg(c: &str) -> &'static str {
        match c.trim() {
            "red" => "\x1B[41m", "green" => "\x1B[42m", "blue" => "\x1B[44m",
            "yellow" => "\x1B[43m", "white" => "\x1B[47m", "black" => "\x1B[40m",
            "cyan" => "\x1B[46m", "magenta" => "\x1B[45m",
            _ => "\x1B[40m",
        }
    }

    pub fn do_log(&self, level: u8, msg: &str) {
        if level < self.log_level { return; }
        let prefix = match level {
            0 => "[DEBUG]", 1 => "[INFO] ", 2 => "[WARN] ", 3 => "[ERROR]", _ => "[LOG]  ",
        };
        let line = format!("{} {}", prefix, msg);
        if level >= 2 {
            eprintln!("\x1B[{}m{}\x1B[0m", if level == 2 { 93 } else { 91 }, line);
        } else {
            println!("{}", line);
        }
    }

    pub fn update_anim(&mut self, name: &str) {
        if let Some(anim) = self.animations.get_mut(name) {
            if !anim.playing { return; }
            let elapsed = anim.last_update.elapsed().as_secs_f64();
            if elapsed >= 1.0 / anim.fps.max(0.1) {
                anim.last_update = Instant::now();
                anim.current = (anim.current + 1) % anim.frames.len().max(1);
            }
        }
    }

        // Создаёт лямбду с захватом БЕЗОПАСНОГО минимума окружения (лексические замыкания).
        // C19, вместо клонирования ВСЕЙ variables (каждая лямбда) захватываем только
        // свободные переменные тела (см. capture_free_vars). Если таковых нет, cap_id=0.
    /// C19, свободные переменные тела лямбды. Сканирует строку тела на
    /// идентификаторы и возвращает только те, что РЕАЛЬНО есть в таблице переменных
    /// (`variables`) на момент создания, вместо клонирования всей области видимости.
    /// Dotted-имена разбираются прогрессивно (a.b.c > a.b > a).
    /// Безопасность, ложное срабатывание бесплатно (лишняя захваченная переменная)
    /// пропуск невозможен (переменная, видимая телу, есть в variables).
    pub fn capture_free_vars(variables: &HashMap<String, Value>, params: &[String], body: &str) -> HashMap<String, Value> {
        let mut out: HashMap<String, Value> = HashMap::new();
        let mut token = String::new();
        for c in body.chars() {
            if c.is_alphanumeric() || c == '_' || c == '.' {
                token.push(c);
                continue;
            }
            if !token.is_empty() {
                Self::flush_capture_var(&mut out, variables, &mut token);
            }
        }
        if !token.is_empty() {
            Self::flush_capture_var(&mut out, variables, &mut token);
        }
        if !params.is_empty() {
            out.retain(|k, _| !params.contains(k));
        }
        out
    }

    fn flush_capture_var(out: &mut HashMap<String, Value>, variables: &HashMap<String, Value>, token: &mut String) {
        let full: String = std::mem::take(token);
        let mut key = full.as_str();
        loop {
            if let Some(v) = variables.get(key) {
                out.entry(key.to_string()).or_insert_with(|| v.clone());
                break;
            }
            if let Some(p) = key.rfind('.') {
                key = &key[..p];
            } else {
                break;
            }
        }
    }

    pub fn make_lambda(&mut self, params: Vec<String>, body: String) -> Value {
        let cap_id = if self.scope_depth > 0 {
            let vars = Self::capture_free_vars(&self.variables, &params, &body);
            if vars.is_empty() {
                0
            } else {
                let id = self.closure_next;
                self.closure_next += 1;
                self.closures.insert(id, ClosureData { vars, depth: self.scope_depth });
                id
            }
        } else { 0 };
        Value::Lambda(params, body, cap_id)
    }

    pub fn call_lambda(&mut self, params: Vec<String>, body: String, args: Vec<Value>, cap_id: u64) -> Result<Value, String> {
        self.call_depth += 1;
        if self.call_depth > 1000 {
            self.call_depth -= 1;
            return Err("recursion depth limit exceeded".to_string());
        }
        let mut restore: Option<(HashMap<String, Value>, usize)> = None;
        if cap_id != 0 {
            let saved_scope = std::mem::replace(&mut self.variables, HashMap::new());
            // КЛОНИРУЕМ данные замыкания (remove уничтожал их на первом вызове
            // повторные или рекурсивные вызовы лямбды теряли захват).
            if let Some(cd) = self.closures.get(&cap_id).cloned() {
                self.variables = cd.vars;
                restore = Some((saved_scope, cd.depth));
                self.scope_depth = cd.depth;
            } else {
                self.variables = saved_scope;
            }
        }
        self.push_scope();
        for (p, a) in params.iter().zip(args.iter()) {
            self.set_var_local(p, a.clone());
        }
        let is_assign = body.contains('=')
            && !body.contains("==") && !body.contains("!=")
            && !body.contains(">=") && !body.contains("<=") && !body.contains("=>");
        let result = if !is_assign && !body.contains('\n') && !body.contains(";") && !body.contains("then") && !body.contains(" do ") {
            match self.eval_value(&body) {
                Ok(v) => Ok(v),
                Err(_) => crate::main::eval::evaluate(self, &body),
            }
        } else {
            crate::main::eval::evaluate(self, &body)
        };
        self.pop_scope();
        if let Some((saved_scope, depth)) = restore {
            self.variables = saved_scope;
            self.scope_depth = depth;
        }
        self.call_depth -= 1;
        result
    }

    pub fn physics_step(&mut self, dt: f64) {
        if !self.physics_on { return; }
        let (gx, gy) = (self.gravity_x, self.gravity_y);
        for e in self.entities.values_mut() {
            e.vx += (e.ax + gx) * dt;
            e.vy += (e.ay + gy) * dt;
            e.x += e.vx * dt;
            e.y += e.vy * dt;
        }
    }

    pub fn print_vars_table(&self) {
        let mut pairs: Vec<(String, String)> = Vec::new();
        for (k, v) in self.variables.iter() {
            pairs.push((k.clone(), v.display()));
        }
        if pairs.is_empty() {
            println!("  (no variables)");
            return;
        }
        for (k, v) in &pairs {
            println!("  {} = {}", k, v);
        }
    }

    pub fn tq_opt(s: &str) -> Option<String> {
        let t = s.trim();
        if Self::is_quoted_literal(t) {
            return Some(t[1..t.len()-1].to_string());
        }
        None
    }

    /// Нормализует команду, добавляет ~ если нет, и возвращает обе версии
    /// Например, "beep" > ("beep", "~beep"), "~beep" > ("beep", "~beep")
    pub fn normalize_command(code: &str) -> (String, String) {
        let trimmed = code.trim();
        if trimmed.starts_with('~') {
            let without = trimmed[1..].to_string();
            (without, trimmed.to_string())
        } else {
            (trimmed.to_string(), format!("~{}", trimmed))
        }
    }

    /// Проверяет, соответствует ли код команде (с ~ или без)
    /// matches_cmd("beep", "~beep") > true
    /// matches_cmd("~beep", "beep") > true
    /// matches_cmd("beep", "beep") > true
    pub fn matches_cmd(code: &str, cmd: &str) -> bool {
        let code_trimmed = code.trim();
        let cmd_trimmed = cmd.trim();

        if code_trimmed == cmd_trimmed {
            return true;
        }

        // Проверяем варианты с ~ и без
        if code_trimmed.starts_with('~') {
            code_trimmed == cmd_trimmed || &code_trimmed[1..] == cmd_trimmed
        } else if cmd_trimmed.starts_with('~') {
            code_trimmed == &cmd_trimmed[1..]
        } else {
            false
        }
    }

    /// Проверяет, начинается ли код с команды (для команд с параметрами)
    /// matches_cmd_prefix("beep(440)", "beep(") > true
    /// matches_cmd_prefix("~beep(440)", "beep(") > true
    /// Печатает только если noecho выключен (для игр)
    pub fn echo(&self, s: impl std::fmt::Display) {
        if !self.noecho {
            println!("{}", s);
        }
    }

    /// Извлекает имя команды (до первой скобки или пробела)
    pub fn extract_cmd_name(code: &str) -> &str {
        let trimmed = code.trim();
        if let Some(idx) = trimmed.find(|c: char| c == '(' || c == ' ') {
            &trimmed[..idx]
        } else {
            trimmed
        }
    }

    pub fn matches_cmd_prefix(code: &str, prefix: &str) -> bool {
        let code_trimmed = code.trim();

        // Убираем ~ из префикса если есть
        let clean_prefix = if prefix.starts_with('~') {
            &prefix[1..]
        } else {
            prefix
        };

        // Проверяем обе версии
        code_trimmed.starts_with(clean_prefix) ||
        code_trimmed.starts_with(&format!("~{}", clean_prefix))
    }

    /// Извлекает аргументы из команды, работает с ~ и без
    /// extract_cmd_args("beep(440, 200)", "beep(") > Some("440, 200")
    /// extract_cmd_args("~beep(440, 200)", "beep(") > Some("440, 200")
    pub fn extract_cmd_args(code: &str, cmd_prefix: &str) -> Option<String> {
        let code_trimmed = code.trim();
        if !code_trimmed.ends_with(')') {
            return None;
        }

        // Пробуем с ~
        let prefix_with_tilde = format!("~{}", cmd_prefix);
        let prefix_without_tilde = cmd_prefix.trim_start_matches('~');

        if let Some(args) = code_trimmed.strip_prefix(&prefix_with_tilde) {
            return Some(args.trim_end_matches(')').to_string());
        }

        if let Some(args) = code_trimmed.strip_prefix(prefix_without_tilde) {
            return Some(args.trim_end_matches(')').to_string());
        }

        None
    }
}
