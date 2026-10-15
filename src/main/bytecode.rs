use crate::main::vm::{VM, Value, ForSource, ForIter};
use crate::main::expr::{ExprOp};
use std::collections::HashMap;
use std::rc::Rc;

/// Микрокоманда локального цикла сплавления (FusedLoopG).
/// Работает с локальным массивом регистров и локальным стеком, без HashMap.
#[derive(Clone, Debug)]
pub enum FuseOp {
    Const(f64),
    LoadR(u16),
    StoreR(u16),
    Add, Sub, Mul, Div,
    AddConstR(u16, f64),
    SubConstR(u16, f64),
    MulConstR(u16, f64),
    DivConstR(u16, f64),
    AddRegR(u16, u16),
    SubRegR(u16, u16),
}

/// Опкоды виртуальной машины чтобы исполнять скрипт без парсинга строк каждый раз.
/// Компилируем скрипт один раз, затем исполняем без строкового парсинга.
#[derive(Clone, Debug)]
pub enum Instr {
    /// Загрузить константу (число)
    Const(f64),
    /// Загрузить переменную по имени
    Load(String),
    /// Положить строковый литерал на стек (уже готовый)
    Str(String),
    /// Бинарные арифметические/логические операции над вершиной стека
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, Ne, Lt, Le, Gt, Ge,
    /// Унарный минус
    Neg,
    /// Присвоить переменную (значение уже на стеке)
    Store(String),
    /// Вызов команды, имя + n аргументов (аргументы на стеке)
    Call(String, usize),
    /// Метка цикла (адрес)
    Label(usize),
    /// Условный переход назад (если значение на стеке == 0) на метку
    JmpBackIfZero(usize),
    /// Условный переход вперёд (если значение на стеке == 0)
    JmpIfZero(usize),
    /// Безусловный переход вперёд (для break/continue, пока не нужно)
    Jmp(usize),
    /// Пустая операция чтобы держать место когда прыжок патчится позже
    Nop,
    // FAST REGISTER ops (числовые локальные регистры)
    /// reg[idx] = const
    SetConstR(u16, f64),
    /// reg[dst] = reg[src]
    CopyR(u16, u16),
    /// reg[idx] += const
    AddConstR(u16, f64),
    /// reg[idx] -= const
    SubConstR(u16, f64),
    /// reg[idx] *= const
    MulConstR(u16, f64),
    /// reg[idx] /= const
    DivConstR(u16, f64),
    /// if reg[src] < const { jump }, числовое сравнение без стека
    JrLtConst(u16, f64, usize),
    /// if reg[src] >= const { jump }
    JrGeConst(u16, f64, usize),
    /// синхронизировать регистр в именованную переменную (на выходе из цикла)
    SyncRegVar(String, u16),
    /// вытолкнуть значение числового регистра как Number на обычный стек
    LoadRegR(u16),
    /// сохранить число со стека в числовой регистр (ускоритель аккумулятора)
    StoreRegR(u16),
    /// reg[dst] += reg[src] (аккумулятор)
    AddRegR(u16, u16),
    /// reg[dst] -= reg[src] (аккумулятор)
    SubRegR(u16, u16),
    /// Сплавленный while-цикл, исполняется как нативный Rust-цикл, без диспатча
    /// на итерацию. Допустимые тела, счётчик (+=/-= константа) и аккумуляторы
    /// `acc += counter` / `acc -= counter` / `acc += константа`.
    /// Поля, step (шаг счётчика, >0), limit (верхняя граница, эксклюзивная)
    /// lin (аккумуляторы, множитель на счётчик), add (аккумуляторы, константа на итерацию).
    FusedLoop { step: f64, limit: f64, lin: Vec<(u16, f64)>, add: Vec<(u16, f64)> },
    /// Сверхплотный вариант, только счётчик reg0 += 1, пока reg0 < limit.
    /// Гвард лимита инструкций проверяется один раз до цикла (трип-каунт)
    /// поэтому тело, без per-iter диспатча и без счётчика итераций.
    FastLoopCounter { limit: f64 },
    /// Сверхплотный вариант, acc_reg += reg0, reg0 += 1, пока reg0 < limit.
    /// (частный случай FusedLoop, step==1, один линейный аккумулятор c coef==1, add пуст)
    FastLoopAcc { limit: f64, acc_reg: u16 },
    /// Общий сплавленный цикл, тело, микрокоманды FuseOp, исполняется локальным
    /// диспатчем без обращения к HashMap и глобальному циклу ВМ. Производные
    /// переменные `x = <expr>` (не читаются в цикле) хостируются в tail
    /// вычисляются один раз на выходе (strength reduction).
    /// limit, верхняя граница счётчика (reg0), ось эксклюзивная.
    FusedLoopG { limit: f64, ops: Vec<FuseOp>, tail: Vec<FuseOp>, nregs: u16 },
    /// булевый литерал true/false из исходника в байткоде (иначе "true" читается
    /// как имя переменной Load("true") > None, и `x = true` молча ломается)
    LitBool(bool),
    /// выполнить строку через старый evaluate (гибридный fallback для некомпилируемых строк)
    Slow(String),
    /// многострочный блок (if/for/loop/try/case/while-вложенный), текст региона
    /// (opener..end) целиком. В рантайме исполняется через exec_lines (блочный
    /// интерпретатор). Позволяет НЕ убивать байткод всего файла из-за одного блока.
    SlowBlock(String),
    /// for-цикл в байткоде. ForBegin, материаллизует источник (eval_for_source)
    /// пустой > переход на end, иначе, кадр в vm.for_stack, элемент > var, ip = body.
    /// ForNext, следующий элемент; есть > var + ip = body, исчерпан > pop кадра, fallthrough.
    /// ForBreak, выход из for-тела, pop кадра + прыжок на end. ОБЯЗАТЕЛЕН вместо
    /// голого Jmp(end), без pop кадр остаётся в for_stack, и вложенный for с break
    /// внутри вешает внешний цикл (ForNext читает чужой кадр) + течь стека.
    ForBegin { var: String, source: String, body: usize, end: usize },
    ForNext { var: String, body: usize },
    ForBreak { end: usize },
}

/// Компилирует список строк скрипта в опкоды.
/// ЛИНИЯ-ТОЛЕРАНТНО, неподдерживаемые строки превращаются в Slow (старый evaluate)
/// поэтому циклы/арифметика в скрипте всё равно компилируются в байткод.
/// Возвращает None только если встретили многострочный блок, который не умеем (if/def/for).
pub fn compile(lines: &[String]) -> Option<Vec<Instr>> {
    // `fn name() { ... }`-блоки умеет только slow-путь (там сборщик fn-блоков
    // в exec_lines); байткод падал бы с Unknown command. Целиком отдаём файл
    // медленному пути, паритет важнее скорости для нового синтаксиса.
    // (Комменты и `fn = 5`/`fn(x)` под маску не попадают.)
    if lines.iter().any(|l| { let t = l.trim(); t == "fn" || t.starts_with("fn ") }) {
        return None;
    }
    let mut code = Vec::new();
    let mut idx = 0usize;
    while idx < lines.len() {
        let line = lines[idx].trim();
        if line.is_empty() || line.starts_with('#') { idx += 1; continue; }
        // REGISTER-FAST, счетчик-цикл (числовой, без команд внутри)
        if let Some(code2) = compile_reg_counter(&mut idx, lines, code.len()) {
            code.extend(code2);
            continue;
        }
        // odnostrochny while, while <cond> do <body>
        // Схема как у многострочного, сначала прыжок на проверку (тело НЕ
        // выполняется до/после ложного условия), иначе off-by-one (тело раз
        // лишнее, проверка стоит ПОСЛЕ тела).
        if let Some(rest) = line.strip_prefix("while ") {
            if let Some(p) = rest.find(" do ") {
                let cond = rest[..p].trim();
                let body = rest[p + 4..].trim();
                let jmp_first = code.len();
                code.push(Instr::Jmp(0)); // fixup > cond_check
                let body_start = code.len();
                // Тело, пробно со свежей рамкой, если в нём голые break/continue
                // (в т.ч. внутри однострочного if), однострочной схеме некуда их
                // патчить > отказ (весь файл в slow, а он умеет).
                let mut blctx = LoopCtx::default();
                if !compile_stmt(&mut code, body, Some(&mut blctx)) { return None; }
                if !blctx.breaks.is_empty() || !blctx.continues.is_empty() { return None; }
                let cond_check = code.len();
                if !compile_expr_to(&mut code, cond) { return None; }
                code.push(Instr::JmpBackIfZero(body_start));
                code[jmp_first] = Instr::Jmp(cond_check);
                idx += 1;
                continue;
            }
            // многострочный while, while <cond> do \n ...body... \n end
            if let Some(cond) = rest.strip_suffix(" do") {
                let cond = cond.trim();
                // собираем тело до "end" (с учётом вложенности). Вложенные блоки
                // (if/for/loop/while/try/case) захватываются РЕГИОНОМ целиком >
                // один SlowBlock, а не per-line Slow (иначе ломается семантика).
                idx += 1;
                let mut body_lines: Vec<String> = Vec::new();
                let mut depth = 0i32;
                loop {
                    if idx >= lines.len() { return None; }
                    let l = lines[idx].trim();
                    if depth == 0 && l == "end" { idx += 1; break; }
                    if depth == 0 && VM::block_opener(l) {
                        if let Some(region) = collect_block_region(lines, &mut idx) {
                            body_lines.push(region);
                            continue;
                        }
                    }
                    if VM::block_opener(l) { depth += 1; }
                    if l == "end" { depth -= 1; }
                    if !l.is_empty() && !l.starts_with('#') { body_lines.push(l.to_string()); }
                    idx += 1;
                }
                // Схема (условие в НАЧАЛЕ итерации)
                // Jmp(cond_check) ; первая итерация, сначала проверить условие
                // body_start
                // ...тело...
                // cond_check
                // <cond> ; пушим bool
                // JmpBackIfZero(body_start) ; true > на тело, false > выход
                let jmp_first = code.len();
                code.push(Instr::Jmp(0)); // fixup > cond_check
                let body_start = code.len();
                // Тело, со своей рамкой break/continue (breaks > за цикл
                // continues > на cond_check; рамка живёт только внутри цикла).
                let mut lctx = LoopCtx::default();
                for bl in &body_lines {
                    // регион вложенного блока (содержит \n), if/while/for/loop
                    // компилируем в байткод, прочие, целиком через блочный исполнитель
                    if bl.contains('\n') {
                        let blines: Vec<String> = bl.lines().map(|l| l.trim().to_string()).collect();
                        let mut bidx = 0usize;
                        if !compile_block_statement(&mut code, &blines, &mut bidx, &mut lctx) {
                            code.push(Instr::SlowBlock(bl.clone()));
                        }
                        continue;
                    }
                    // голые break/continue напрямую в теле while
                    if bl.trim() == "break" {
                        lctx.breaks.push(code.len());
                        code.push(Instr::Jmp(0)); // fixup > за цикл
                        continue;
                    }
                    if bl.trim() == "continue" {
                        lctx.continues.push(code.len());
                        code.push(Instr::Jmp(0)); // fixup > cond_check
                        continue;
                    }
                    if !compile_stmt(&mut code, bl, Some(&mut lctx)) {
                        code.push(Instr::Slow(bl.clone()));
                    }
                }
                let cond_check = code.len();
                if !compile_expr_to(&mut code, cond) { return None; }
                code.push(Instr::JmpBackIfZero(body_start));
                let end_pos = code.len();
                for pos in lctx.breaks.drain(..) { code[pos] = Instr::Jmp(end_pos); }
                for pos in lctx.continues.drain(..) { code[pos] = Instr::Jmp(cond_check); }
                code[jmp_first] = Instr::Jmp(cond_check);
                continue;
            }
            code.push(Instr::Slow(line.to_string()));
            idx += 1;
            continue;
        }
        // if/def/for/loop, многострочные блоки. if-блоки компилируются в байткод
        // (JmpIfZero/Jmp), прочие (for/loop/...) НЕ убивают байткод всего файла
        // регион целиком уходит в SlowBlock (в рантайме, блочный exec_lines)
        // остаток файла компилируется.
        if line.starts_with("~def ") || line.starts_with("def ") {
            code.push(Instr::Slow(line.to_string()));
            idx += 1;
            continue;
        }
        if VM::block_opener(line) {
            // Вне циклов рамки нет, голые break/continue внутри блока здесь
            // отказ в SlowBlock (семантика через exec_lines, как раньше).
            let mut lctx = LoopCtx::default();
            let start = code.len();
            let start_idx = idx;
            if !compile_block_statement(&mut code, lines, &mut idx, &mut lctx) {
                // несбалансированный блок, как раньше, отдаём весь файл медленному пути
                return None;
            }
            if !lctx.breaks.is_empty() || !lctx.continues.is_empty() {
                // break/continue без рамки цикла, откатываемся в SlowBlock
                code.truncate(start);
                idx = start_idx;
                if let Some(region) = collect_block_region(lines, &mut idx) {
                    code.push(Instr::SlowBlock(region));
                    continue;
                }
                return None;
            }
            continue;
        }
        if !compile_stmt(&mut code, line, None) {
            code.push(Instr::Slow(line.to_string()));
        }
        idx += 1;
    }
    #[cfg(debug_assertions)]
    if std::env::var("TREK_DUMP_BC").is_ok() {
        eprintln!("--- code ---");
        for (i, ins) in code.iter().enumerate() { eprintln!("{:4}: {:?}", i, ins); }
    }
    Some(code)
}

/// Собирает строки многострочного блока (от opener до matching "end") в единый текст.
/// idx указывает НА строку-опенер; по выходу idx стоит ЗА закрывающим "end".
/// Возвращает None, если блок не сбалансирован (тогда вызывающий откатывается на старое поведение).
fn collect_block_region(lines: &[String], idx: &mut usize) -> Option<String> {
    let start = *idx;
    let mut depth = 0i32;
    while *idx < lines.len() {
        let l = lines[*idx].trim();
        if VM::block_opener(l) { depth += 1; }
        if l == "end" {
            depth -= 1;
            if depth == 0 {
                *idx += 1;
                return Some(lines[start..*idx].join("\n"));
            }
        }
        *idx += 1;
    }
    None
}

/// Компилирует многострочный блок-стейтмент, открывающий lines[*idx].
/// if-блок > байткод (JmpIfZero/Jmp); for-блок > байткод (ForBegin/ForNext);
/// while-блок > байткод (Jmp/JmpBackIfZero); прочие (loop/try/...) > SlowBlock регион.
/// По выходу *idx, за закрывающим `end` (или за регионом).
fn compile_block_statement(code: &mut Vec<Instr>, lines: &[String], idx: &mut usize, lctx: &mut LoopCtx) -> bool {
    let start = code.len();
    let start_idx = *idx;
    let l = lines[*idx].trim();
    if l.starts_with("if ") && l.ends_with(" then") {
        if compile_if_statement(code, lines, idx, lctx) { return true; }
        code.truncate(start);
        *idx = start_idx;
    }
    if l.starts_with("while ") {
        if compile_while_statement(code, lines, idx, lctx) { return true; }
        code.truncate(start);
        *idx = start_idx;
    }
    if l.starts_with("for ") {
        if compile_for_statement(code, lines, idx, lctx) { return true; }
        code.truncate(start);
        *idx = start_idx;
    }
    if l == "loop" {
        if compile_loop_statement(code, lines, idx, lctx) { return true; }
        code.truncate(start);
        *idx = start_idx;
    }
    if let Some(region) = collect_block_region(lines, idx) {
        code.push(Instr::SlowBlock(region));
        return true;
    }
    false
}

/// Компилирует многострочный `while <cond> do ... end` в байткод.
/// Схема (как у топуровневого while), Jmp(cond_check); тело; cond_check, <cond>;
/// JmpBackIfZero(body_start). Вложенные if/while/for в теле, рекурсивно байткод.
/// break/continue в теле, Jmp с бэкпатчем (своя рамка; внешняя сохраняется).
fn compile_while_statement(code: &mut Vec<Instr>, lines: &[String], idx: &mut usize, outer: &mut LoopCtx) -> bool {
    let l = lines[*idx].trim();
    let cond_s = match l.strip_prefix("while ") {
        Some(rest) => rest.strip_suffix(" do").map(|s| s.trim().to_string()).unwrap_or_else(|| rest.trim().to_string()),
        None => return false,
    };
    if cond_s.is_empty() { return false; }
    *idx += 1;
    let (ni, body_lines) = match vm_collect_simple_lines(lines, *idx) { Some(p) => p, None => return false };
    *idx = ni;
    let start = code.len();
    code.push(Instr::Jmp(0)); // fixup > cond_check
    let body_start = code.len();
    let saved = std::mem::take(outer);
    if !compile_statements(code, &body_lines, &mut 0usize, outer) {
        *outer = saved;
        code.truncate(start);
        return false;
    }
    let cond_check = code.len();
    if !compile_expr_to(code, &cond_s) {
        *outer = saved;
        code.truncate(start);
        return false;
    }
    code.push(Instr::JmpBackIfZero(body_start));
    let end_pos = code.len();
    for pos in outer.breaks.drain(..) { code[pos] = Instr::Jmp(end_pos); }
    for pos in outer.continues.drain(..) { code[pos] = Instr::Jmp(cond_check); }
    *outer = saved;
    code[start] = Instr::Jmp(cond_check);
    true
}

/// Компилирует `for <var> in <source> do ... end` (и `for <v> from a to b do ...`) в байткод.
/// Тело, через compile_statements; break/continue, Jmp с бэкпатчем
/// (breaks > за цикл, continues > на ForNext; своя рамка, внешняя сохраняется).
fn compile_for_statement(code: &mut Vec<Instr>, lines: &[String], idx: &mut usize, outer: &mut LoopCtx) -> bool {
    let opener = lines[*idx].trim();
    let (head, rest) = match VM::parse_for(opener) { Some(p) => p, None => return false };
    let (var, src) = match VM::parse_for_head(&head) { Some(p) => p, None => return false };
    if var.is_empty() || src.is_empty() { return false; }
    // тело, inline (после " do ") или строки до "end"
    let mut body_lines: Vec<String> = Vec::new();
    if !rest.is_empty() {
        let mut r = rest.trim();
        if let Some(noend) = r.strip_suffix(" end") { r = noend.trim(); }
        if !r.is_empty() { body_lines.push(r.to_string()); }
        *idx += 1;
    } else {
        *idx += 1;
        let (ni, bl) = match vm_collect_simple_lines(lines, *idx) { Some(p) => p, None => return false };
        *idx = ni;
        body_lines = bl;
    }
    let start = code.len();
    code.push(Instr::ForBegin { var: var.clone(), source: src.clone(), body: 0, end: 0 });
    let body_start = code.len();
    let saved = std::mem::take(outer);
    if !compile_statements(code, &body_lines, &mut 0usize, outer) {
        *outer = saved;
        code.truncate(start);
        return false;
    }
    let next_pos = code.len();
    code.push(Instr::ForNext { var: var.clone(), body: body_start });
    let end_pos = code.len();
    for pos in outer.breaks.drain(..) { code[pos] = Instr::ForBreak { end: end_pos }; }
    for pos in outer.continues.drain(..) { code[pos] = Instr::Jmp(next_pos); }
    *outer = saved;
    match &mut code[start] {
        Instr::ForBegin { body, end, .. } => { *body = body_start; *end = end_pos; }
        _ => return false,
    }
    true
}

/// Компилирует многострочный `loop ... end` в байткод.
/// Схема, body_start, тело; Jmp(body_start); end. break > Jmp(end)
/// continue > Jmp(body_start) (своя рамка, внешняя сохраняется).
fn compile_loop_statement(code: &mut Vec<Instr>, lines: &[String], idx: &mut usize, outer: &mut LoopCtx) -> bool {
    let l = lines[*idx].trim();
    if l != "loop" { return false; }
    *idx += 1;
    let (ni, body_lines) = match vm_collect_simple_lines(lines, *idx) { Some(p) => p, None => return false };
    *idx = ni;
    if body_lines.is_empty() { return false; }
    let start = code.len();
    let body_start = code.len();
    let saved = std::mem::take(outer);
    if !compile_statements(code, &body_lines, &mut 0usize, outer) {
        *outer = saved;
        code.truncate(start);
        return false;
    }
    code.push(Instr::Jmp(body_start));
    let end_pos = code.len();
    for pos in outer.breaks.drain(..) { code[pos] = Instr::Jmp(end_pos); }
    for pos in outer.continues.drain(..) { code[pos] = Instr::Jmp(body_start); }
    *outer = saved;
    let _ = start;
    true
}

/// Собирает телo for (как VM::collect_simple_lines) без borrow VM.
fn vm_collect_simple_lines(lines: &[String], mut idx: usize) -> Option<(usize, Vec<String>)> {
    let mut body = Vec::new();
    let mut depth = 0i32;
    while idx < lines.len() {
        let line = lines[idx].trim();
        if depth == 0 && line == "end" { idx += 1; break; }
        if VM::block_opener(line) { depth += 1; }
        if line == "end" { depth -= 1; }
        body.push(line.to_string());
        idx += 1;
    }
    Some((idx, body))
}

/// Компилирует многострочный if/elif/else-блок СТЕЙТМЕНТОМ в байткод.
/// lines[*idx], строка-опенер `if <cond> then`; по выходу *idx, за `end`.
/// Ветки тела, обычные строки через compile_stmt; вложенные if-блоки, рекурсивно в
/// байткод; прочие вложенные блоки (while/for/loop/try/case/...), SlowBlock регион.
/// Схема (аналог collect_if_arms + eval_cond)
/// <cond1> ; JZ(next1) ; тело1 ; JMP(end)
/// next1, <cond2> ; JZ(next2) ; тело2 ; JMP(end)
/// ... [else, телоN без условия]
/// end
fn compile_if_statement(code: &mut Vec<Instr>, lines: &[String], idx: &mut usize, lctx: &mut LoopCtx) -> bool {
    let head = lines[*idx].trim();
    let mut cond: Option<String> = extract_if_cond(&head);
    let mut arms: Vec<(Option<String>, Vec<String>)> = Vec::new();
    let mut body: Vec<String> = Vec::new();
    let mut depth = 0i32;
    *idx += 1;
    while *idx < lines.len() {
        let l = lines[*idx].trim();
        if depth == 0 {
            if l == "end" {
                *idx += 1;
                arms.push((cond.take(), std::mem::take(&mut body)));
                break;
            }
            if let Some(c) = VM::elif_cond(l) {
                arms.push((cond.take(), std::mem::take(&mut body)));
                cond = Some(c);
                *idx += 1;
                continue;
            }
            if l == "else" {
                arms.push((cond.take(), std::mem::take(&mut body)));
                cond = None;
                *idx += 1;
                continue;
            }
        }
        if VM::block_opener(l) { depth += 1; }
        if l == "end" { depth -= 1; }
        body.push(l.to_string());
        *idx += 1;
    }
    if cond.is_some() || !body.is_empty() {
        arms.push((cond.take(), std::mem::take(&mut body)));
    }
    if arms.is_empty() { return false; }

    let start = code.len();
    // Снапшот рамки, при откате truncate(start) ниже надо откатить и fixup'ы
    // уже записанные ранними ветками, иначе висячие индексы тихо портят
    // чужие инструкции (или паника на drain).
    let (sbr, sco) = (lctx.breaks.len(), lctx.continues.len());
    let rollback = |code: &mut Vec<Instr>, lctx: &mut LoopCtx| {
        code.truncate(start);
        lctx.breaks.truncate(sbr);
        lctx.continues.truncate(sco);
    };
    let mut pending_jmps: Vec<usize> = Vec::new();
    let mut last_jz: Option<usize> = None;
    let last_arm = arms.len() - 1;
    for (i, (acond, blines)) in arms.iter().enumerate() {
        match acond {
            Some(c) => {
                if c.is_empty() { rollback(code, lctx); return false; }
                if !compile_expr_to(code, c) { rollback(code, lctx); return false; }
                let jz = code.len();
                code.push(Instr::JmpIfZero(0));
                if !compile_statements(code, blines, &mut 0usize, lctx) { rollback(code, lctx); return false; }
                if i == last_arm {
                    last_jz = Some(jz);
                } else {
                    // конец текущей ветки, JMP на конец блока; JZ, на начало cond следующей ветки
                    pending_jmps.push(code.len());
                    code.push(Instr::Jmp(0));
                    code[jz] = Instr::JmpIfZero(code.len());
                }
            }
            None => {
                // ветка else, выполняется без условия, проваливается к концу блока
                if !compile_statements(code, blines, &mut 0usize, lctx) { rollback(code, lctx); return false; }
            }
        }
    }
    let end = code.len();
    for j in pending_jmps { code[j] = Instr::Jmp(end); }
    if let Some(jz) = last_jz { code[jz] = Instr::JmpIfZero(end); }
    true
}

/// Компилирует список строк тела ветки. Вложенные if-блоки, рекурсивно в байткод
/// прочие блоки, регион в SlowBlock; некомпилируемые строки, Slow (per-line).
/// from, курсор по lines (продвигается также вложенными компиляторами).
fn compile_statements(code: &mut Vec<Instr>, lines: &[String], from: &mut usize, lctx: &mut LoopCtx) -> bool {
    let n = lines.len();
    while *from < n {
        let l = lines[*from].trim();
        if l.is_empty() || l.starts_with('#') { *from += 1; continue; }
        // голые break/continue, прыжок с бэкпатчем (позицию правит рамка цикла)
        if l == "break" {
            lctx.breaks.push(code.len());
            code.push(Instr::Jmp(0)); // fixup
            *from += 1;
            continue;
        }
        if l == "continue" {
            lctx.continues.push(code.len());
            code.push(Instr::Jmp(0)); // fixup
            *from += 1;
            continue;
        }
        if VM::block_opener(l) {
            // if/while/for/loop, байткод; прочие блоки (try/...), SlowBlock регион
            if !compile_block_statement(code, lines, from, lctx) { return false; }
            continue;
        }
        if !compile_stmt(code, l, Some(&mut *lctx)) {
            code.push(Instr::Slow(l.to_string()));
        }
        *from += 1;
    }
    true
}

/// Рамка цикла для бэкпатча break/continue, позиции Jmp-плейсхолдеров.
/// Каждый цикл (while/for/loop) заводит свою рамку; if, пробрасывает чужую.
#[derive(Default)]
struct LoopCtx {
    breaks: Vec<usize>,
    continues: Vec<usize>,
}

/// Читает условие из `if <cond> then` / `elif <cond> then` / `else if <cond> then`.
fn extract_if_cond(line: &str) -> Option<String> {
    let l = line.trim();
    let rest = if let Some(r) = l.strip_prefix("if ") { Some(r) }
        else if let Some(r) = l.strip_prefix("elif ") { Some(r) }
        else if let Some(r) = l.strip_prefix("else if ") { Some(r) }
        else { None }?;
    let c = rest.strip_suffix(" then").unwrap_or(rest).trim();
    if c.is_empty() { None } else { Some(c.to_string()) }
}

/// Компилирует ОДНОСТРОЧНЫЙ if-СТЕЙТМЕНТ
/// `if C1 then S1 [elif C2 then S2 ...] [else S3] end`.
/// Ветки, стейтменты (можно с `;`-цепочками). Голые break/continue в ветках
/// компилируются в рамку цикла через lctx; без рамки, Slow-фрагментом.
/// Не компилируется > false (Slow).
fn compile_inline_if_stmt(code: &mut Vec<Instr>, line: &str, mut lctx: Option<&mut LoopCtx>) -> bool {
    let l = line.trim();
    if !(l.starts_with("if ") && l.ends_with(" end")) { return false; }
    let inner = &l[3..l.len() - 4];
    let then_pos = match inner.find(" then ") { Some(p) => p, None => return false };
    let cond = inner[..then_pos].trim();
    if cond.is_empty() { return false; }
    let rest = inner[then_pos + 6..].trim();
    // Делим хвост на elif-ветки и else, `S1 elif C2 then S2 ... else S4`.
    let (chain, else_opt) = match find_top_else(rest) {
        Some(p) => (rest[..p].trim(), Some(rest[p + 6..].trim())),
        None => (rest, None),
    };
    if chain.is_empty() { return false; }
    let mut arms: Vec<(String, String)> = Vec::new();
    let mut cur_cond = cond.to_string();
    let mut cur = chain;
    loop {
        match find_top_elif(cur) {
            Some(p) => {
                let tail = cur[p + 6..].trim();
                let tp = match tail.find(" then ") { Some(p) => p, None => return false };
                let cc = tail[..tp].trim();
                if cc.is_empty() { return false; }
                arms.push((cur_cond, cur[..p].trim().to_string()));
                cur_cond = cc.to_string();
                cur = tail[tp + 6..].trim();
            }
            None => { arms.push((cur_cond, cur.to_string())); break; }
        }
    }
    if arms.iter().any(|(_, s)| s.trim().is_empty()) { return false; }
    let start = code.len();
    // Снапшот рамки (см. compile_if_statement), откат truncate обязан откатить
    // и fixup'ы ветки, иначе висячие индексы портят чужие инструкции.
    let snap = lctx.as_ref().map(|l| (l.breaks.len(), l.continues.len()));
    let rollback = |code: &mut Vec<Instr>, lctx: &mut Option<&mut LoopCtx>| {
        code.truncate(start);
        if let (Some((sbr, sco)), Some(l)) = (snap, lctx.as_deref_mut()) {
            l.breaks.truncate(sbr);
            l.continues.truncate(sco);
        }
    };
    // Схема, <c1>; JZ(next1); S1; JMP(end); next1, <c2>; JZ(next2); S2; ...
    let has_else = else_opt.map(|e| !e.is_empty()).unwrap_or(false);
    let mut pending: Vec<usize> = Vec::new();
    for (i, (c, s)) in arms.iter().enumerate() {
        if !compile_expr_to(code, c) { rollback(code, &mut lctx); return false; }
        let jz = code.len();
        code.push(Instr::JmpIfZero(0));
        if !compile_stmt_sections(code, s, lctx.as_deref_mut()) { rollback(code, &mut lctx); return false; }
        if i + 1 < arms.len() || has_else {
            pending.push(code.len());
            code.push(Instr::Jmp(0));
        }
        let next = code.len();
        code[jz] = Instr::JmpIfZero(next);
    }
    if let Some(bb) = else_opt {
        if !bb.is_empty() {
            if !compile_stmt_sections(code, bb, lctx.as_deref_mut()) { rollback(code, &mut lctx); return false; }
        }
    }
    let end = code.len();
    for pos in pending { code[pos] = Instr::Jmp(end); }
    true
}

/// Ищет топ-level ` elif ` (вне вложенных then/end), пара к find_top_else.
fn find_top_elif(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut skip_next_then = false;
    let mut i = 0;
    while i < bytes.len() {
        if s[i..].starts_with(" then ") {
            if skip_next_then { skip_next_then = false; }
            else { depth += 1; }
            i += 6;
            continue;
        }
        if s[i..].starts_with(" end") {
            depth -= 1;
            i += 4;
            continue;
        }
        if s[i..].starts_with(" elif ") {
            // elif любого уровня, следующий then, его собственный, не считаем.
            skip_next_then = true;
            if depth == 0 { return Some(i); }
            i += 6;
            continue;
        }
        i += 1;
    }
    None
}

/// Компилирует `S1; S2; ...`, цепочку стейтментов (для веток однострочного if).
/// Голые break/continue, в рамку lctx; нескомпилированное (и break/continue
/// без рамки), в Slow-инструкцию ВНУТРИ ветки, а не отказом всей строки
/// иначе висячий Slow на уровне строки роняет __break__/__continue__ наружу
/// (молча пустой вывод вместо управления циклом). Семантика та же, что у
/// compile_statements для тел (там построчный Slow).
fn compile_stmt_sections(code: &mut Vec<Instr>, s: &str, mut lctx: Option<&mut LoopCtx>) -> bool {
    let parts: Vec<String> = VM::split_top_level(s, ';')
        .iter().map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
    if parts.is_empty() { return false; }
    for p in &parts {
        let t = p.trim();
        // Остатки if-цепочек (elif приходит куском `s3 = 1 elif ...`, else-if
        // куском `if b then Y` без закрывающего end), НЕ самодостаточные
        // стейтменты, Slow-фрагментом их исполнять нельзя (молча неверно)
        // только отказом всей строки в Slow. Внутри кавычек та же подстрока
        // даёт лишь ложный отказ в Slow (медленно, но верно).
        if t.starts_with("elif ") || t.contains(" elif ")
            || (t.starts_with("if ") && !t.ends_with(" end"))
            || t == "else" || t.starts_with("else ")
        {
            return false;
        }
        if t == "break" {
            match lctx.as_deref_mut() {
                Some(l) => { l.breaks.push(code.len()); code.push(Instr::Jmp(0)); }
                None => code.push(Instr::Slow(t.to_string())),
            }
            continue;
        }
        if t == "continue" {
            match lctx.as_deref_mut() {
                Some(l) => { l.continues.push(code.len()); code.push(Instr::Jmp(0)); }
                None => code.push(Instr::Slow(t.to_string())),
            }
            continue;
        }
        if !compile_stmt(code, p, lctx.as_deref_mut()) {
            code.push(Instr::Slow(p.clone()));
        }
    }
    true
}

/// Регистровый быстрый компилятор счётчиков и аккумуляторов. Поддерживает
/// name = <init> (счётчик, живёт в fast_reg[0])
/// acc = <init> (аккумуляторы, fast_reg[1..]; опционально, несколько)
/// while name < <limit> do
/// name = name + <step> (или name = name, <step>)
/// acc = acc + ... (аккумулятор через регистр, без HashMap)
/// end
/// Возвращает Some(код) и двигает idx. Убирает HashMap-доступ и клонирование из горячего цикла.
fn compile_reg_counter(idx: &mut usize, lines: &[String], base: usize) -> Option<Vec<Instr>> {
    if *idx + 3 >= lines.len() { return None; }
    // 1) подряд идущие инициализации `x = <число>` перед while
    let mut inits: Vec<(String, f64)> = Vec::new();
    let mut scan = *idx;
    loop {
        if scan >= lines.len() { return None; }
        let l = lines[scan].trim();
        if let Some((name, expr)) = parse_assign(l) {
            let v: f64 = match expr.parse::<f64>() { Ok(v) => v, Err(_) => return None };
            inits.push((name, v));
            scan += 1;
            continue;
        }
        break;
    }
    if inits.is_empty() { return None; }
    if scan >= lines.len() { return None; }
    // 2) следующая строка, while
    let wline = lines[scan].trim();
    let rest = match wline.strip_prefix("while ") {
        Some(r) => r,
        None => return None,
    };
    // разделитель " do", cond, условие до него, body, всё, что после (может быть пусто)
    let (cond, body_after_do) = match rest.find(" do") {
        Some(q) if q + 4 <= rest.len() => (rest[..q].trim(), rest[q + 4..].trim()),
        Some(q) => (rest[..q].trim(), ""),
        None => return None,
    };
    let cond = cond.trim();
    let (cvar, cop, climit) = parse_simple_cond(cond)?;
    // 3) счётчик должен быть одной из инициализаций; ищем его и помещаем в reg0
    let cvar_idx = inits.iter().position(|(n, _)| *n == cvar)?;
    let (name, init_val) = inits[cvar_idx].clone();
    // карта имя > регистр, счётчик рег[0], остальные по порядку рег[1..]
    let mut name_to_reg: HashMap<String, u16> = HashMap::new();
    name_to_reg.insert(name.clone(), 0);
    let mut reg_inits: Vec<(String, f64)> = vec![(name.clone(), init_val)];
    for (i, (n, v)) in inits.iter().enumerate() {
        if i != cvar_idx {
            let reg = reg_inits.len() as u16;
            name_to_reg.insert(n.clone(), reg);
            reg_inits.push((n.clone(), *v));
        }
    }

    // собираем строки тела цикла, начинаем с body_after_do (если непуст), затем строки файла до "end"
    let mut body_lines: Vec<String> = Vec::new();
    let mut k = scan + 1;
    let mut pending: Option<String> = if body_after_do.is_empty() { None } else { Some(body_after_do.to_string()) };
    loop {
        if let Some(p) = pending.take() {
            // тело прямо в строке while
            if p == "end" { k += 1; break; }
            if let Some(b) = p.strip_suffix(" end") {
                if !b.trim().is_empty() { body_lines.push(b.trim().to_string()); }
                k += 1;
                break;
            }
            if !p.trim().is_empty() { body_lines.push(p.trim().to_string()); }
            k += 1;
            continue;
        }
        if k >= lines.len() { return None; }
        let l = lines[k].trim();
        if l == "end" { k += 1; break; }
        if l.is_empty() || l.starts_with('#') { k += 1; continue; }
        body_lines.push(l.to_string());
        k += 1;
    }

    // компилируем тело в операции. Счётчик живёт в fast_reg[0], аккумуляторы, в fast_reg[1..].
    let mut body_code: Vec<Instr> = Vec::new();
    let mut has_increment = false;
    for bl in &body_lines {
        if let Some(delta) = parse_counter_assign(bl, &name) {
            let d: f64 = delta.parse::<f64>().ok()?;
            body_code.push(Instr::AddConstR(0, d));
            has_increment = true;
            continue;
        }
        // присваивание x = <expr>, если x регистровая переменная, читаем/пишем регистр;
        // иначе обычная переменная (HashMap) с регистровым выражением в правой части.
        if let Some(p) = bl.find(" = ") {
            let lhs = bl[..p].trim().to_string();
            let expr = bl[p + 3..].trim();
            if VM::is_assignable(&lhs) {
                if let Some(&lreg) = name_to_reg.get(&lhs) {
                    // аккумулятор, x = x + y / x = x, y, одной инструкцией
                    if let Some((op, rreg)) = parse_acc_expr(expr, &lhs, &name_to_reg) {
                        body_code.push(match op { '+' => Instr::AddRegR(lreg, rreg), '-' => Instr::SubRegR(lreg, rreg), _ => unreachable!() });
                        continue;
                    }
                    if compile_reg_expr_map(&mut body_code, expr, &name_to_reg) {
                        body_code.push(Instr::StoreRegR(lreg));
                        continue;
                    }
                } else if compile_reg_expr_map(&mut body_code, expr, &name_to_reg) {
                    // производная переменная (x = i*2+3), выдаём ей новый регистр
                    // чтобы весь цикл держать в регистрах (без HashMap)
                    let r = name_to_reg.len() as u16;
                    name_to_reg.insert(lhs.clone(), r);
                    body_code.push(Instr::StoreRegR(r));
                    continue;
                }
            }
        }
        // команда или присваивание общего вида, регистровый путь не поддерживает
        // команды со ссылками на переменные корректно; возвращаемся на старый интерпретатор.
        return None;
    }
    if !has_increment { return None; }

    let limit_val = match cop {
        "<" => climit.parse::<f64>().ok()?,
        "<=" => climit.parse::<f64>().ok()? + 1.0,
        _ => return None,
    };

    let mut code = Vec::new();
    // инициализация всех регистров
    for (r, (_, v)) in reg_inits.iter().enumerate() {
        code.push(Instr::SetConstR(r as u16, *v));
    }
    // ОПТИМИЗАЦИЯ, если тело, простой счётчик+аккумуляторы, сплавляем весь
    // цикл в одну нативную итерацию (Instr::FusedLoop), без диспатча.
    if let Some((step, lin, add)) = try_fuse_loop(&body_code) {
        // Сверхплотные варианты (step==1, точный целочисленный диапазон f64)
        // без пер-итерного гварда и итераторов, трип-каунт считается один раз.
        let safe_range = limit_val.abs() < 9.0e15;
        let fast = step == 1.0 && safe_range;
        if fast && lin.is_empty() && add.is_empty() {
            code.push(Instr::FastLoopCounter { limit: limit_val });
        } else if fast && lin.len() == 1 && lin[0].1 == 1.0 && add.is_empty() {
            code.push(Instr::FastLoopAcc { limit: limit_val, acc_reg: lin[0].0 });
        } else {
            code.push(Instr::FusedLoop { step, limit: limit_val, lin, add });
        }
        for (n, r) in &name_to_reg {
            code.push(Instr::SyncRegVar(n.clone(), *r));
        }
        *idx = k;
        return Some(code);
    }
    // ОПТИМИЗАЦИЯ 2, если тело, произвольные числовые операнды над регистрами
    // (x = i*2+3, y = x + 1, ...), сплавляем в локальный микродиспатч FusedLoopG.
    if let Some((ops, nregs)) = try_fuse_general(&body_code) {
        code.push(Instr::FusedLoopG { limit: limit_val, ops, tail: Vec::new(), nregs });
        for (n, r) in &name_to_reg {
            code.push(Instr::SyncRegVar(n.clone(), *r));
        }
        *idx = k;
        return Some(code);
    }
    // JrLtConst(reg, limit, body_addr), если reg<limit, прыгает на body
    let jr_pos = code.len();
    code.push(Instr::JrLtConst(0, limit_val, 0)); // fixup
    // выход (условие ложно) > на sync
    let jmp_sync_pos = code.len();
    code.push(Instr::Jmp(0)); // fixup > sync
    // тело
    let body_len = body_code.len();
    code.extend(body_code);
    code.push(Instr::Jmp(0)); // fixup > cond_loop
    // sync, записать все регистры обратно в переменные
    let sync_pos = code.len();
    for (n, r) in &name_to_reg {
        code.push(Instr::SyncRegVar(n.clone(), *r));
    }
    let exit = code.len();
    // patch targets (в коде `code`, локальные индексы; target в инструкциях, глобальные с base)
    let body_start_local = jmp_sync_pos + 1;
    let back_jmp_local = body_start_local + body_len;
    let cond_loop = base + jr_pos;
    let sync_global = base + sync_pos;
    code[jr_pos] = Instr::JrLtConst(0, limit_val, (base + body_start_local) as usize);
    code[jmp_sync_pos] = Instr::Jmp(sync_global);
    code[back_jmp_local] = Instr::Jmp(cond_loop);
    let _ = exit;
    *idx = k;
    Some(code)
}

/// Компилирует числовое выражение, где имена переменных читаются из регистров (карта name > reg)
/// `j`, `j*2+3`, `n + j`, `acc+1`, скобки и +-*/%. Незнакомые имена возвращают false.
fn compile_reg_expr_map(code: &mut Vec<Instr>, expr: &str, map: &HashMap<String, u16>) -> bool {
    let e = expr.trim();
    if e.is_empty() { return false; }
    if let Ok(n) = e.parse::<f64>() {
        code.push(Instr::Const(n));
        return true;
    }
    // простая переменная или число > регистр
    if e.chars().all(|c| c.is_alphanumeric() || c == '_') {
        if let Some(&r) = map.get(e) {
            code.push(Instr::LoadRegR(r));
            return true;
        }
        return false;
    }
    // бинарная арифметика (строка с верхним приоритетом + или -)
    if let Some((pos, opc)) = find_top_binop(e) {
        let left = &e[..pos];
        let right = &e[pos + 1..];
        if left.trim().is_empty() || right.trim().is_empty() { return false; }
        if compile_reg_expr_map(code, left, map) && compile_reg_expr_map(code, right, map) {
            code.push(match opc {
                '+' => Instr::Add, '-' => Instr::Sub,
                '*' => Instr::Mul, '/' => Instr::Div,
                '%' => Instr::Mod, _ => return false,
            });
            return true;
        }
        return false;
    }
    // скобки
    if e.starts_with('(') && e.ends_with(')') {
        return compile_reg_expr_map(code, &e[1..e.len() - 1], map);
    }
    false
}



/// Находит бинарный оператор верхнего уровня (вне скобок)
/// + и, считаются низким приоритетом, * и / и %, высоким.
/// Возвращает позицию и символ оператора; приоритет, у крайнего правого низкого.
fn find_top_binop(expr: &str) -> Option<(usize, char)> {
    // ищем последний оператор на глубине 0 с учётом приоритета
    // + и, имеют низкий приоритет, * / %, высокий.
    // Разрезаем по самой правой позиции низкоприоритетного оператора; если их нет
    // по самой правой позиции * / %.
    let mut depth = 0i32;
    let mut low_pos: Option<(usize, char)> = None; // последний + или
    let mut high_pos: Option<(usize, char)> = None; // последний * или / или %
    for (i, c) in expr.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            '+' | '-' if depth == 0 => low_pos = Some((i, c)),
            '*' | '/' | '%' if depth == 0 => high_pos = Some((i, c)),
            _ => {}
        }
    }
    low_pos.or(high_pos)
}

fn parse_assign(line: &str) -> Option<(String, String)> {
    let p = line.find(" = ")?;
    let name = line[..p].trim().to_string();
    if !VM::is_assignable(&name) { return None; }
    let expr = line[p + 3..].trim().to_string();
    Some((name, expr))
}

fn parse_simple_cond(cond: &str) -> Option<(String, &str, &str)> {
    for op in ["<=", ">=", "<", ">"] {
        if let Some(p) = cond.find(op) {
            let var = cond[..p].trim();
            if !var.is_empty() && var.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let val = cond[p + op.len()..].trim();
                return Some((var.to_string(), op, val));
            }
        }
    }
    None
}

fn parse_counter_assign(line: &str, name: &str) -> Option<String> {
    let p = line.find(" = ")?;
    let bname = line[..p].trim().to_string();
    if bname != name { return None; }
    let rhs = line[p + 3..].trim();
    let exp1 = format!("{} + ", name);
    let exp2 = format!("{} - ", name);
    if let Some(r) = rhs.strip_prefix(&exp1) {
        return Some(r.trim().to_string());
    }
    if let Some(r) = rhs.strip_prefix(&exp2) {
        let mut d = r.trim().to_string();
        if !d.starts_with('-') { d = format!("-{}", d); }
        return Some(d);
    }
    None
}

/// Аккумуляторное присваивание `x = x <op> <regvar>` для регистровых переменных.
/// Возвращает (op, reg_rhs), где reg_rhs, регистр правого операнда.
fn parse_acc_expr(expr: &str, lhs: &str, map: &HashMap<String, u16>) -> Option<(char, u16)> {
    let e = expr.trim();
    let p = e.find('+').or_else(|| e.find('-'))?;
    let left = e[..p].trim();
    let right = e[p + 1..].trim();
    if left != lhs { return None; }
    let op = e[p..].chars().next()?;
    match map.get(right) {
        Some(&rreg) => Some((op, rreg)),
        None => None,
    }
}

/// Сплавляет тело регистрового счётчикового цикла в нативный Rust-цикл.
/// Разрешены только
/// `AddConstR(0, d)` / `SubConstR(0, d)`, шаг счётчика (reg0)
/// `AddRegR(i, 0)` / `SubRegR(i, 0)`, аккумулятор i += reg0 / i -= reg0
/// `AddConstR(i, d)` / `SubConstR(i, d)`, константное приращение аккумулятора i
/// Возвращает (step, lin, add) как для Instr::FusedLoop. Иначе None.
fn try_fuse_loop(body: &[Instr]) -> Option<(f64, Vec<(u16, f64)>, Vec<(u16, f64)>)> {
    let mut step = 0.0f64;
    let mut lin: Vec<(u16, f64)> = Vec::new(); // множители на счётчик
    let mut add: Vec<(u16, f64)> = Vec::new(); // константные приращения
    for ins in body {
        match ins {
            Instr::AddConstR(r, d) if *r == 0 => step += *d,
            Instr::SubConstR(r, d) if *r == 0 => step -= *d,
            Instr::AddConstR(r, d) => { push_acc(&mut add, *r, *d); }
            Instr::SubConstR(r, d) => { push_acc(&mut add, *r, -*d); }
            Instr::AddRegR(dst, 0) => { lin.push((*dst, 1.0)); }
            Instr::SubRegR(dst, 0) => { lin.push((*dst, -1.0)); }
            _ => return None,
        }
    }
    if step <= 0.0 { return None; } // нужен строго растущий счётчик
    Some((step, lin, add))
}

fn push_acc(v: &mut Vec<(u16, f64)>, r: u16, d: f64) {
    if let Some(k) = v.iter_mut().find(|(x, _)| *x == r) { k.1 += d; } else { v.push((r, d)); }
}

/// Конвертирует тело регистрового цикла (производные выражения, аккумуляторы)
/// в микрокоманды FuseOp для локального исполнения без HashMap и глобального диспатча.
fn try_fuse_general(body: &[Instr]) -> Option<(Vec<FuseOp>, u16)> {
    let mut ops: Vec<FuseOp> = Vec::with_capacity(body.len());
    let mut maxreg: u16 = 0;
    let mut step = 0.0f64;
    for ins in body {
        let (f, r) = match ins {
            Instr::Const(n) => (Some(FuseOp::Const(*n)), 0),
            Instr::AddConstR(r, d) => { if *r == 0 { step += *d; } (Some(FuseOp::AddConstR(*r, *d)), *r) }
            Instr::SubConstR(r, d) => { if *r == 0 { step -= *d; } (Some(FuseOp::SubConstR(*r, *d)), *r) }
            Instr::AddRegR(a, b) => (Some(FuseOp::AddRegR(*a, *b)), (*a).max(*b)),
            Instr::SubRegR(a, b) => (Some(FuseOp::SubRegR(*a, *b)), (*a).max(*b)),
            Instr::LoadRegR(r) => (Some(FuseOp::LoadR(*r)), *r),
            Instr::StoreRegR(r) => (Some(FuseOp::StoreR(*r)), *r),
            Instr::Add => (Some(FuseOp::Add), 0),
            Instr::Sub => (Some(FuseOp::Sub), 0),
            Instr::Mul => (Some(FuseOp::Mul), 0),
            Instr::Div => (Some(FuseOp::Div), 0),
            Instr::MulConstR(r, d) => (Some(FuseOp::MulConstR(*r, *d)), *r),
            Instr::DivConstR(r, d) => (Some(FuseOp::DivConstR(*r, *d)), *r),
            _ => return None,
        };
        ops.push(f?);
        if r + 1 > maxreg { maxreg = r + 1; }
    }
    if maxreg == 0 { return None; }
    if step <= 0.0 { return None; } // нужен строго растущий счётчик
    Some((ops, maxreg))
}


/// Компилирует одну инструкцию (присваивание или вызов команды).
/// lctx, рамка цикла для голых break/continue внутри однострочных if
/// (None, вне цикла, такой if уходит в Slow, как раньше).
fn compile_stmt(code: &mut Vec<Instr>, line: &str, lctx: Option<&mut LoopCtx>) -> bool {
    // однострочный if-СТЕЙТМЕНТ `if c then S [else S2] end`, в байткод
    // (иначе каждая итерация уходит в evaluate через eval_inline_if_stmt)
    if line.starts_with("if ") && line.ends_with(" end") {
        return compile_inline_if_stmt(code, line, lctx);
    }
    // присваивание, name = expr
    if let Some(p) = line.find(" = ") {
        let name = line[..p].trim();
        let expr = line[p + 3..].trim();
        if VM::is_assignable(name) {
            // строковый литерал в RHS присваивания, храним БЕЗ кавычек (семантика slow-пути)
            // иначе `a = "5"` накапливает кавычки в переменной и ломает арифметику.
            if (expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2)
                || (expr.starts_with('\'') && expr.ends_with('\'') && expr.len() >= 2)
            {
                code.push(Instr::Str(VM::tq(expr)));
                code.push(Instr::Store(name.to_string()));
                return true;
            }
            if !compile_expr_to(code, expr) { return false; }
            code.push(Instr::Store(name.to_string()));
            return true;
        }
    }
    // вызов команды, name(args) или name
    if let Some(open) = line.find('(') {
        if line.ends_with(')') {
            let name = line[..open].trim();
            let args_str = &line[open + 1..line.len() - 1];
            let args: Vec<String> = if args_str.trim().is_empty() {
                vec![]
            } else {
                VM::split_top_level(args_str, ',').iter().map(|s| s.trim().to_string()).collect()
            };
            // Аргументы, идентификаторы/литералы, сырым Str (имена переменных
            // для varref-команд резолвер берёт как есть, НЕ раскрывая в Load).
            // ЛОВУШКА, сложный первый аргумент (вложенный вызов trim(s)) НЕЛЬЗЯ пушить
            // сырым Str, JIT прочитает его как имя переменной, generic закавычит
            // и len посчитает длину ТЕКСТА вызова (len(trim(s)) давал 7 вместо 11).
            // Только простые идентификаторы/литералы идут в Str, остальное
            // в выражение (не скомпилировалось > false > Slow, а он умеет вложенность).
            for a in args.iter() {
                if is_simple_ident(a) || is_quoted_literal(a) {
                    code.push(Instr::Str(a.clone()));
                } else if !compile_expr_to(code, a) {
                    return false;
                }
            }
            code.push(Instr::Call(name.to_string(), args.len()));
            return true;
        }
    }
    // просто имя команды без скобок
    if line.chars().all(|c| c.is_alphanumeric() || c == '_') {
        code.push(Instr::Call(line.to_string(), 0));
        return true;
    }
    false
}

/// Есть ли в выражении and/or/not СЛОВАМИ (не внутри идентификатора).
/// Голое `and`/`or`/`not` как всё выражение, это переменная, не оператор.
fn has_bool_words(expr: &str) -> bool {
    let t = expr.trim();
    if t == "and" || t == "or" || t == "not" { return false; }
    let mut cur = String::new();
    for c in t.chars().chain(std::iter::once(' ')) {
        if c.is_alphanumeric() || c == '_' {
            cur.push(c);
        } else {
            if cur == "and" || cur == "or" || cur == "not" { return true; }
            cur.clear();
        }
    }
    false
}

/// Простой идентификатор (имя переменной), без скобок/операторов.
fn is_simple_ident(s: &str) -> bool {
    let t = s.trim();
    !t.is_empty() && t.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.')
}

/// Закавыченный строковый литерал "..." / '...' (идёт в Str как есть).
fn is_quoted_literal(s: &str) -> bool {
    let t = s.trim();
    t.len() >= 2 && ((t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')))
}

/// Начинается ли строка с вызова команды `имя(...)` (а не с группировки `(...)`).
fn is_command_call(s: &str) -> bool {
    let t = s.trim();
    if !t.ends_with(')') { return false; }
    let first = t.chars().next().unwrap_or(')');
    if !(first.is_alphabetic() || first == '_') { return false; }
    if let Some(open) = t.find('(') {
        let head = t[..open].trim();
        if head.is_empty() || !head.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
            return false;
        }
        // должно быть ИМЯ-команда, а не переменная с индексом/группировка
        return !t.starts_with("(");
    }
    false
}

/// Команды, у которых ПЕРВЫЙ аргумент, имя переменной (мутируемой/читаемой по ссылке)
/// а не значение. Для них первый аргумент передаём как строку-имя.
fn is_varref_first_arg(cmd: &str) -> bool {
    let c = cmd.trim_start_matches('~');
    matches!(
        c,
        "dict_set" | "dset" | "dict_get" | "dget" | "dict_del" | "ddel" | "dict_has" | "dhas"
            | "list_push" | "add" | "list_pop" | "pop" | "list_len" | "len" | "list_get" | "get"
            | "list_set" | "set" | "list_insert" | "insert" | "list_remove" | "remove"
            | "list_clear" | "dinc" | "ddec" | "set_add" | "stack_push" | "queue_push"
            | "list_join" | "list_reverse" | "sort" | "list_sort" | "list_sum" | "list_min"
            | "list_max" | "list_avg" | "list_empty" | "list_contains"
    )
}

/// Компилирует выражение (арифметика ИЛИ сравнение) в опкоды.
/// Поддерживаем, числа, переменные, строки, бинарные операции (+-*/%^ &|^)
/// сравнения (== != < <= > >=), унарный минус.
fn compile_expr_to(code: &mut Vec<Instr>, expr: &str) -> bool {
    let expr = expr.trim();
    if expr.is_empty() { return false; }
    // if-выражения (`if c then A else B end`) байткод НЕ компилирует, Slow-путь с корректным
    // eval_value (иначе VM_compile_ops молча собирает мусор, `v = if..end` давало false/none).
    if expr.starts_with("if ") { return false; }
    // число
    if let Ok(n) = expr.parse::<f64>() {
        code.push(Instr::Const(n));
        return true;
    }
    // строковый литерал "..." или '...', сохраняем КАВЫЧКИ внутри значения
    // чтобы build_call/translate_alias/evaluate на другом конце распознали строку.
    if (expr.starts_with('"') && expr.ends_with('"') && expr.len() >= 2)
        || (expr.starts_with('\'') && expr.ends_with('\'') && expr.len() >= 2)
    {
        code.push(Instr::Str(expr.to_string()));
        return true;
    }
    // булевые литералы, до ветки переменной (иначе "true" уходит в Load("true") > None)
    if expr == "true" { code.push(Instr::LitBool(true));  return true; }
    if expr == "false" { code.push(Instr::LitBool(false)); return true; }
    // переменная
    if expr.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
        code.push(Instr::Load(expr.to_string()));
        return true;
    }
    // and/or/not СЛОВАМИ, не операторы байткода (токенизатор собрал бы
    // PushVar("and") и тихо испортил стек, `s == 2 and a == 0` врало).
    // Отказываемся в Slow, там eval_cond их понимает. Голые переменные
    // and/or/not уже обработаны веткой выше; строки внутри кавычек могут дать
    // ложный отказ, это лишь перф (Slow верен).
    if has_bool_words(expr) { return false; }
    // сравнения (обрабатываем до арифметики)
    for op in ["==", "!=", "<=", ">=", "<", ">"] {
        if let Some(p) = expr.find(op) {
            let (a, b) = (&expr[..p], &expr[p + op.len()..]);
            if compile_expr_to(code, a) && compile_expr_to(code, b) {
                let instr = match op {
                    "==" => Instr::Eq, "!=" => Instr::Ne,
                    "<" => Instr::Lt, "<=" => Instr::Le,
                    ">" => Instr::Gt, ">=" => Instr::Ge,
                    _ => Instr::Nop,
                };
                code.push(instr);
                return true;
            }
            return false;
        }
    }
    if expr.contains('[') || expr.contains(']') { return false; }
    // Выражение с вызовами/скобками, рекурсивный бинарный сплит по нижнему
    // приоритету НА ТОП-уровне (вне скобок), чтобы `call f(a) + call g(b)` разделить на вызовы.
    if expr.contains('(') || expr.contains(')') {
        if let Some((p, opc)) = find_top_binop(expr) {
            let left = expr[..p].trim();
            let right = expr[p + 1..].trim();
            if left.is_empty() {
                // унарный минус в начале (напр. -call f(x))
                if opc == '-' && compile_expr_to(code, right) {
                    code.push(Instr::Neg);
                    return true;
                }
                return false;
            }
            if compile_expr_to(code, left) && compile_expr_to(code, right) {
                let instr = match opc {
                    '+' => Instr::Add, '-' => Instr::Sub, '*' => Instr::Mul,
                    '/' => Instr::Div, '%' => Instr::Mod, _ => return false,
                };
                code.push(instr);
                return true;
            }
            return false;
        }
        // внешние скобки (expr), снимаем и рекурсируем (иначе `(j % 3)` не компилируется)
        if expr.starts_with('(') && expr.ends_with(')') {
            let inner = &expr[1..expr.len() - 1];
            let mut d = 0i32;
            let mut wraps = true;
            for c in inner.chars() {
                match c { '(' => d += 1, ')' => { d -= 1; if d < 0 { wraps = false; break; } }, _ => {} }
            }
            if wraps && d == 0 && !inner.trim().is_empty() {
                if compile_expr_to(code, inner) { return true; }
                return false;
            }
        }
        // атомный вызов, call name(args)
        if let Some(rest) = expr.strip_prefix("call ") {
            return compile_call_to(code, rest.trim());
        }
        // команда в RHS (varref-чтение, get/list_get/dget/len/...), list_get(g, k) > Call
        if let Some(open) = expr.find('(') {
            let head = expr[..open].trim();
            if is_varref_first_arg(head) {
                return compile_cmd_call_to(code, expr);
            }
        }
        return false;
    }
    match VM_compile_ops(expr) {
        Some(ops) => {
            for op in ops {
                // Битовые операции (And/Or/Xor/Shl/Shr) не имеют опкодов, не эмитим Nop
                // (это давало тихо неверный результат), а откатываемся на Slow-интерпретатор.
                match Instr::from_exprop(&op) {
                    Some(ins) => code.push(ins),
                    None => return false,
                }
            }
            true
        }
        None => false,
    }
}

/// Вызывает VM::compile_expr (он в impl VM в expr.rs). Обёртка.
fn VM_compile_ops(expr: &str) -> Option<Vec<ExprOp>> {
    let vm = VM::new();
    vm.compile_expr(expr).ok()
}

/// Компилирует вызов `call name(args)` > Instr::Call(name, argc) с аргументами-выражениями.
/// Требуется ровно один вызов, после сбалансированной закрывающей скобки не должно быть
/// содержимого (иначе это составное выражение, обрабатывает find_top_binop).
fn compile_call_to(code: &mut Vec<Instr>, rest: &str) -> bool {
    let rest = rest.trim();
    if !rest.ends_with(')') { return false; }
    let open = match rest.find('(') { Some(i) => i, None => return false };
    let name = rest[..open].trim();
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
        return false;
    }
    // сбалансированная закрывающая скобка первого вызова, после неё ничего не должно быть
    let mut depth = 0i32;
    let mut close = None;
    for (bi, bc) in rest.bytes().enumerate() {
        if bi < open { continue; }
        match bc {
            b'(' => depth += 1,
            b')' => { depth -= 1; if depth == 0 { close = Some(bi); break; } }
            _ => {}
        }
    }
    let close = match close { Some(c) => c, None => return false };
    if rest[close + 1..].trim() != "" { return false; }
    let args_str = &rest[open + 1..close];
    if args_str.trim().is_empty() {
        code.push(Instr::Call(name.to_string(), 0));
        return true;
    }
    let args: Vec<String> = VM::split_top_level(args_str, ',').iter().map(|s| s.trim().to_string()).collect();
    for a in &args {
        if !compile_expr_to(code, a) { return false; }
    }
    code.push(Instr::Call(name.to_string(), args.len()));
    true
}

/// Компилирует команду в RHS-выражении (list_get(g, k) > Call("list_get", 2)).
/// Идентификаторы/литералы, в Str; сложные аргументы, в выражение, иначе false
/// (Slow). Сырой Str сложного аргумента давал неверные результаты (len считал
/// длину текста вложенного вызова).
/// compile_stmt; остальные, простые имена строками, выражения, опкодами.
fn compile_cmd_call_to(code: &mut Vec<Instr>, expr: &str) -> bool {
    let expr = expr.trim();
    let open = match expr.find('(') { Some(o) => o, None => return false };
    if !expr.ends_with(')') { return false; }
    let name = expr[..open].trim();
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
        return false;
    }
    let args_str = &expr[open + 1..expr.len() - 1];
    let args: Vec<String> = if args_str.trim().is_empty() {
        vec![]
    } else {
        VM::split_top_level(args_str, ',').iter().map(|s| s.trim().to_string()).collect()
    };
    // Только идентификаторы и литералы, в Str, остальное в выражение (false > Slow).
    for a in args.iter() {
        if is_simple_ident(a) || is_quoted_literal(a) {
            code.push(Instr::Str(a.clone()));
        } else if !compile_expr_to(code, a) {
            return false;
        }
    }
    code.push(Instr::Call(name.to_string(), args.len()));
    true
}

/// Кэш скомпилированного тела функции (~def). Два режима исполнения
/// `code`, HashMap/scope-режим (параметры через bind_params + переменные)
/// общий случай;
/// `code_reg`, ЧИСТО числовой режим, параметры в fast_reg[0..reg_n] через
/// LoadRegR/StoreRegR, БЕЗ push_scope/bind_params/pop_scope. Только если тело
/// компилируется целиком как числа/параметры/вызовы/if (без строк, команд, глобалов).
#[derive(Clone)]
pub struct BodyCache {
    pub code: Option<Rc<Vec<Instr>>>,
    pub code_reg: Option<(Rc<Vec<Instr>>, usize)>,
}

/// Компилирует тело функции (~def) в байткод для КЭША быстрых вызовов.
/// Возвращает None, если тело не компилируется (тогда функция уходит на старый
/// строковый путь без потери семантики). Поддерживаются
/// простое выражение, `a * 2`, `call fib(n-1)`;
/// однострочный if-выражение, `if c then A else B end` (else опционален);
/// регистровый режим для чистых числовых тел (params > fast_reg).
pub fn compile_body(body: &str, params: &[String]) -> Option<BodyCache> {
    let mut cache = BodyCache { code: None, code_reg: None };
    let b = body.trim();
    if b.is_empty() || b.contains('\n') || b.starts_with("return") {
        return (if cache.code.is_some() || cache.code_reg.is_some() { Some(cache) } else { None });
    }
    let mut code = Vec::new();
    if compile_body_expr(&mut code, b) {
        cache.code = Some(Rc::new(code));
    }
    // регистровый режим (чистая числовая арифметика)
    let mut reg_code = Vec::new();
    if let Some(n) = compile_body_reg(&mut reg_code, b, params) {
        cache.code_reg = Some((Rc::new(reg_code), n));
    }
    if cache.code.is_none() && cache.code_reg.is_none() { None } else { Some(cache) }
}

/// Регистровый компилятор тела, параметры превращаются в LoadRegR(idx)
/// всё остальное должно быть числами/вызовами/if. Возвращает None при любой
/// неподдерживаемой конструкции (тогда вызывающему нужно перейти на code-режим).
fn compile_body_reg(code: &mut Vec<Instr>, body: &str, params: &[String]) -> Option<usize> {
    if params.is_empty() { return None; }
    let mut reg_of: HashMap<&str, u16> = HashMap::new();
    for (i, p) in params.iter().enumerate() {
        if i >= 8 { return None; } // разумный предел для регистрового режима
        let pn = p.split_once('=').map(|x| x.0.trim()).unwrap_or(p.trim());
        if pn.is_empty() || !pn.chars().all(|c| c.is_alphanumeric() || c == '_') { return None; }
        reg_of.insert(pn, i as u16);
    }
    compile_expr_reg(code, body.trim(), &reg_of)?;
    let n = reg_of.values().map(|v| *v as usize).max().map(|m| m + 1)?;
    Some(n)
}

/// ЧИСТО ЧИСЛОВОЙ компилятор выражения, числа, параметры ( > LoadRegR)
/// `call f(...)`, if-выражение, сравнения, +‑*/, унарный минус. При любой
/// неподдерживаемой конструкции возвращает None.
fn compile_expr_reg(code: &mut Vec<Instr>, expr: &str, reg_of: &HashMap<&str, u16>) -> Option<()> {
    let expr = expr.trim();
    if expr.is_empty() { return None; }
    // число
    if let Ok(n) = expr.parse::<f64>() {
        code.push(Instr::Const(n));
        return Some(());
    }
    // простое имя, параметр или глобал (только параметр идёт в регистр)
    if expr.chars().all(|c| c.is_alphanumeric() || c == '_') {
        if let Some(r) = reg_of.get(expr) {
            code.push(Instr::LoadRegR(*r));
            return Some(());
        }
        return None; // глобальная переменная, вне регистрового режима
    }
    // строки/литералы, не числовое
    if expr.starts_with('"') || expr.starts_with('\'') || expr.starts_with('`') { return None; }
    // if-выражение обрабатываем ДО бинарного сплита
    if expr.starts_with("if ") && expr.ends_with(" end") {
        let inner = &expr[3..expr.len() - 4];
        let then_pos = inner.find(" then ")?;
        let cond = inner[..then_pos].trim();
        let rest = inner[then_pos + 6..].trim();
        let (a, b_opt) = match find_top_else(rest) {
            Some(p) => (rest[..p].trim(), Some(rest[p + 6..].trim())),
            None => (rest, None),
        };
        compile_expr_reg(code, cond, reg_of)?;
        let jz = code.len();
        code.push(Instr::JmpIfZero(0));
        compile_expr_reg(code, a, reg_of)?;
        let jmp = code.len();
        code.push(Instr::Jmp(0));
        let else_start = code.len();
        if let Some(bb) = b_opt {
            if bb.is_empty() { return None; }
            compile_expr_reg(code, bb, reg_of)?;
        }
        let end = code.len();
        code[jz] = Instr::JmpIfZero(else_start);
        code[jmp] = Instr::Jmp(end);
        return Some(());
    }
    if expr.starts_with("if ") { return None; }
    // атомный вызов call name(args), только если нет верхнеуровневого бинарника
    if let Some(rest) = expr.strip_prefix("call ") {
        if find_top_binop(expr).is_none() {
            return compile_call_reg(code, rest.trim(), reg_of);
        }
    }
    // сравнения
    for op in ["==", "!=", "<=", ">=", "<", ">"] {
        if let Some(p) = expr.find(op) {
            let (a, b) = (&expr[..p], &expr[p + op.len()..]);
            compile_expr_reg(code, a, reg_of)?;
            compile_expr_reg(code, b, reg_of)?;
            let instr = match op {
                "==" => Instr::Eq, "!=" => Instr::Ne,
                "<" => Instr::Lt, "<=" => Instr::Le,
                ">" => Instr::Gt, ">=" => Instr::Ge,
                _ => return None,
            };
            code.push(instr);
            return Some(());
        }
    }
    // бинарный сплит по верхнеуровневому оператору
    if let Some((p, opc)) = find_top_binop(expr) {
        let left = expr[..p].trim();
        let right = expr[p + 1..].trim();
        if left.is_empty() {
            if opc == '-' { compile_expr_reg(code, right, reg_of)?; code.push(Instr::Neg); return Some(()); }
            return None;
        }
        compile_expr_reg(code, left, reg_of)?;
        compile_expr_reg(code, right, reg_of)?;
        let instr = match opc {
            '+' => Instr::Add, '-' => Instr::Sub, '*' => Instr::Mul, '/' => Instr::Div,
            '%' => Instr::Mod, _ => return None,
        };
        code.push(instr);
        return Some(());
    }
    None
}

/// Регистровый вариант compile_call_to, аргументы тоже через compile_expr_reg.
fn compile_call_reg(code: &mut Vec<Instr>, rest: &str, reg_of: &HashMap<&str, u16>) -> Option<()> {
    let rest = rest.trim();
    if !rest.ends_with(')') { return None; }
    let open = rest.find('(')?;
    let name = rest[..open].trim();
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
        return None;
    }
    let mut depth = 0i32;
    let mut close = None;
    for (bi, bc) in rest.bytes().enumerate() {
        if bi < open { continue; }
        match bc {
            b'(' => depth += 1,
            b')' => { depth -= 1; if depth == 0 { close = Some(bi); break; } }
            _ => {}
        }
    }
    let close = close?;
    if rest[close + 1..].trim() != "" { return None; }
    let args_str = &rest[open + 1..close];
    if args_str.trim().is_empty() {
        code.push(Instr::Call(name.to_string(), 0));
        return Some(());
    }
    let args: Vec<String> = VM::split_top_level(args_str, ',').iter().map(|s| s.trim().to_string()).collect();
    if args.len() > 256 { return None; }
    for a in &args {
        compile_expr_reg(code, a, reg_of)?;
    }
    code.push(Instr::Call(name.to_string(), args.len()));
    Some(())
}

/// Компилирует тело функции как ВЫРАЖЕНИЕ (одна строка), if-expr или арифметика/call.
fn compile_body_expr(code: &mut Vec<Instr>, body: &str) -> bool {
    let b = body.trim();
    if b.starts_with("if ") && b.ends_with(" end") {
        // if <cond> then <A> [else <B>] end
        let inner = &b[3..b.len() - 4]; // без "if " и " end"
        let then_pos = match inner.find(" then ") { Some(p) => p, None => return false };
        let cond = inner[..then_pos].trim();
        let rest = inner[then_pos + 6..].trim();
        let (a, b_opt) = match find_top_else(rest) {
            Some(p) => (rest[..p].trim(), Some(rest[p + 6..].trim())),
            None => (rest, None),
        };
        if !compile_expr_to(code, cond) { return false; }
        let jz = code.len();
        code.push(Instr::JmpIfZero(0)); // fixup > else/end
        if !compile_body_expr(code, a) { return false; }
        let jmp = code.len();
        code.push(Instr::Jmp(0)); // fixup > end
        let else_start = code.len();
        if let Some(bb) = b_opt {
            if bb.is_empty() { return false; }
            if !compile_body_expr(code, bb) { return false; }
        }
        let end = code.len();
        code[jz] = Instr::JmpIfZero(else_start);
        code[jmp] = Instr::Jmp(end);
        return true;
    }
    if b.starts_with("if ") { return false; }
    compile_expr_to(code, b)
}

/// Позиция нижнего (depth 0) " else " в остатке if-выражения (вне вложенных if/end).
fn find_top_else(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut skip_next_then = false;
    let mut i = 0;
    while i < bytes.len() {
        if s[i..].starts_with(" then ") {
            if skip_next_then { skip_next_then = false; }
            else { depth += 1; }
            i += 6;
            continue;
        }
        if s[i..].starts_with(" end") {
            depth -= 1;
            i += 4;
            continue;
        }
        // `elif` любого уровня, следующий then, его собственный (sibling)
        // глубину не меняет. Без флага elif-цепочки травят счётчик и прячут
        // топ-level else (однострочник с elif веткой уходил в Slow/ломался).
        if s[i..].starts_with(" elif ") {
            skip_next_then = true;
            i += 6;
            continue;
        }
        if s[i..].starts_with(" else ") {
            if depth == 0 { return Some(i); }
            i += 6;
            continue;
        }
        if s[i..].starts_with("else ") || s[i..].starts_with(" end") || s[i..].starts_with(" then") {
            i += 1;
            continue;
        }
        i += 1;
    }
    None
}

impl Instr {
    fn from_exprop(op: &ExprOp) -> Option<Instr> {
        use crate::main::expr::ExprOp::*;
        match op {
            PushNum(n) => Some(Instr::Const(*n)),
            PushVar(n) => Some(Instr::Load(n.clone())),
            Add => Some(Instr::Add),
            Sub => Some(Instr::Sub),
            Mul => Some(Instr::Mul),
            Div => Some(Instr::Div),
            Mod => Some(Instr::Mod),
            Pow => Some(Instr::Pow),
            And | Or | Xor | Shl | Shr => None, // нет опкодов, вызывающий откатывается на Slow
            Not => None,
            Neg => Some(Instr::Neg),
        }
    }
}

/// Сплавляет регистровый while-цикл в нативный Rust-цикл над локальными переменными
/// счётчик (reg0) растёт на `step` пока < `limit`; на каждой итерации аккумуляторы из `lin`
/// получают `+= counter*coef`, из `add`, `+= const`. Без диспатча.
fn run_fused_loop(vm: &mut VM, step: f64, limit: f64, lin: &[(u16, f64)], add: &[(u16, f64)]) -> Result<(), String> {
    let mut counter = vm.fast_reg(0);
    // локальные копии аккумуляторов
    let mut lacc: Vec<(u16, f64)> = lin.iter().map(|(r, _)| (*r, vm.fast_reg(*r))).collect();
    let mut aacc: Vec<(u16, f64)> = add.iter().map(|(r, _)| (*r, vm.fast_reg(*r))).collect();
    let mut iters: u64 = 0;
    while counter < limit {
        // линейные аккумуляторы, acc += counter * coef
        for ((_, v), (_, coef)) in lacc.iter_mut().zip(lin) { *v += counter * coef; }
        // константные приращения
        for ((_, v), (_, d)) in aacc.iter_mut().zip(add) { *v += d; }
        counter += step;
        iters += 1;
        if iters > 200_000_000_000 { return Err("bytecode: instruction limit exceeded".to_string()); }
    }
    vm.set_fast_reg(0, counter);
    for (r, v) in lacc { vm.set_fast_reg(r, v); }
    for (r, v) in aacc { vm.set_fast_reg(r, v); }
    Ok(())
}

/// Сверхплотный пустой цикл, reg0 += 1 пока reg0 < limit. Гвард один раз до цикла.
fn run_fast_loop_counter(vm: &mut VM, limit: f64) -> Result<(), String> {
    let mut c = vm.fast_reg(0);
    let trip = ((limit - c) / 1.0).ceil().max(0.0) as u128;
    if trip > 200_000_000_000 { return Err("bytecode: instruction limit exceeded".to_string()); }
    while c < limit { c += 1.0; }
    vm.set_fast_reg(0, c);
    Ok(())
}

/// Сверхплотный цикл с аккумулятором, acc_reg += reg0; reg0 += 1; пока reg0 < limit.
fn run_fast_loop_acc(vm: &mut VM, limit: f64, acc_reg: u16) -> Result<(), String> {
    let mut c = vm.fast_reg(0);
    let mut acc = vm.fast_reg(acc_reg);
    let trip = ((limit - c) / 1.0).ceil().max(0.0) as u128;
    if trip > 200_000_000_000 { return Err("bytecode: instruction limit exceeded".to_string()); }
    while c < limit {
        acc += c;
        c += 1.0;
    }
    vm.set_fast_reg(0, c);
    vm.set_fast_reg(acc_reg, acc);
    Ok(())
}

/// Общий сплавленный цикл, локальные регистры + локальный стек, микрокоманды FuseOp.
/// Счётчик в reg[0], предел, `limit`. Опции выполняются на каждой итерации.
fn run_fused_loop_g(vm: &mut VM, limit: f64, ops: &[FuseOp], tail: &[FuseOp], nregs: u16) -> Result<(), String> {
    // локальные копии регистров
    let mut regs: Vec<f64> = (0..nregs as usize).map(|i| vm.fast_reg(i as u16)).collect();
    let mut stack: Vec<f64> = Vec::with_capacity(8);
    let mut iters: u64 = 0;
    while regs[0] < limit {
        for op in ops {
            exec_fuse_op(op, &mut regs, &mut stack)?;
        }
        iters += 1;
        if iters > 200_000_000_000 { return Err("bytecode: instruction limit exceeded".to_string()); }
    }
    // tail, вычисления после цикла (производные значения)
    for op in tail {
        exec_fuse_op(op, &mut regs, &mut stack)?;
    }
    for (i, v) in regs.iter().enumerate() {
        vm.set_fast_reg(i as u16, *v);
    }
    Ok(())
}

/// Разрешяет аргумент команды из байткода ровно как eval_value во время разрешения
/// `"..."`, литерал (обрезать кавычки), голое слово, переменная/число/текст.
#[inline]
fn resolve_cmd_arg(vm: &mut VM, v: &Value) -> Value {
    match v {
        Value::Text(t) => {
            let t = t.trim();
            if (t.starts_with('"') && t.ends_with('"') && t.len() >= 2)
                || (t.starts_with('\'') && t.ends_with('\'') && t.len() >= 2)
            {
                Value::Text(VM::tq(t))
            } else if t == "true" {
                Value::Bool(true)
            } else if t == "false" {
                Value::Bool(false)
            } else if t == "none" {
                Value::None
            } else if t.starts_with('[') && t.ends_with(']') {
                // список-литерал, как в slow-path eval_value
                vm.eval_value(t).unwrap_or_else(|_| v.clone())
            } else if t.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
                if let Some(varv) = vm.get_var(t) {
                    varv
                } else if let Ok(n) = t.parse::<f64>() {
                    Value::Number(n)
                } else {
                    Value::Text(t.to_string())
                }
            } else {
                v.clone()
            }
        }
        _ => v.clone(),
    }
}

#[inline]
fn exec_fuse_op(op: &FuseOp, regs: &mut [f64], stack: &mut Vec<f64>) -> Result<(), String> {
    match op {
        FuseOp::Const(n) => stack.push(*n),
        FuseOp::LoadR(r) => stack.push(regs[*r as usize]),
        FuseOp::StoreR(r) => { if let Some(v) = stack.pop() { regs[*r as usize] = v; } }
        FuseOp::Add => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a + b); }
        FuseOp::Sub => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a - b); }
        FuseOp::Mul => { let b = stack.pop().unwrap_or(0.0); let a = stack.pop().unwrap_or(0.0); stack.push(a * b); }
        FuseOp::Div => {
            let b = stack.pop().unwrap_or(0.0);
            let a = stack.pop().unwrap_or(0.0);
            if b == 0.0 { return Err("Division by zero".to_string()); }
            stack.push(a / b);
        }
        FuseOp::AddConstR(r, d) => regs[*r as usize] += *d,
        FuseOp::SubConstR(r, d) => regs[*r as usize] -= *d,
        FuseOp::MulConstR(r, d) => regs[*r as usize] *= *d,
        FuseOp::DivConstR(r, d) => { if *d == 0.0 { return Err("Division by zero".to_string()); } regs[*r as usize] /= *d; }
        FuseOp::AddRegR(a, b) => regs[*a as usize] += regs[*b as usize],
        FuseOp::SubRegR(a, b) => regs[*a as usize] -= regs[*b as usize],
    }
    Ok(())
}

/// Исполняет скомпилированный байткод (возвращает значение последнего присваивания/вызова).
pub fn run(vm: &mut VM, code: &[Instr]) -> Result<Value, String> {
    let r = run_impl(vm, code, false);
    // При аварийном выходе кадры for_stack протухают и портят СЛЕДУЮЩИЙ запуск
    // (ForNext читает чужой кадр). Успешный прогон всегда сбалансирован.
    // Заодно пишем last_error, иначе ~error/~backtrace в fast всегда "(no error)".
    if let Err(e) = &r {
        vm.for_stack.clear();
        vm.last_error = e.clone();
    }
    r
}

/// Как run(), но возвращает ВЕРХ стека (результат выражения), для тел функций.
pub fn run_func(vm: &mut VM, code: &[Instr]) -> Result<Value, String> {
    run_impl(vm, code, true)
}

fn run_impl(vm: &mut VM, code: &[Instr], ret_top: bool) -> Result<Value, String> {
    let mut stack: Vec<Value> = Vec::new();
    let mut ip = 0usize;
    let mut last = Value::None;
    let mut guard = 0usize;
    while ip < code.len() {
        guard += 1;
        if guard > 200_000_000_000 { return Err("iteration limit exceeded".to_string()); }
        match &code[ip] {
            Instr::Const(n) => stack.push(Value::Number(*n)),
            Instr::Str(s) => stack.push(Value::Text(s.clone())),
            Instr::Load(name) => {
                let v = vm.get_var(name).unwrap_or(Value::None);
                stack.push(v);
            }
            Instr::Store(name) => {
                let v = stack.pop().unwrap_or(Value::None);
                vm.set_var(name, v.clone());
                last = v;
            }
            Instr::Add => {
                // Правило (сохраняет все старые числовые кейсы + чинит конкатенацию)
                // оба as_number (текст-число коэрсится, "5" + 0 = 5) > арифметика;
                // иначе текст с любой стороны > конкатенация display (кавычки
                // литералов срезаем через VM::tq); иначе, type error как раньше.
                let b = stack.pop().unwrap_or(Value::None);
                let a = stack.pop().unwrap_or(Value::None);
                match (a.as_number(), b.as_number()) {
                    (Some(x), Some(y)) => stack.push(Value::Number(x + y)),
                    _ if matches!(a, Value::Text(_)) || matches!(b, Value::Text(_)) => {
                        let ds = |v: &Value| match v {
                            Value::Text(t) => VM::tq(t),
                            _ => v.display(),
                        };
                        stack.push(Value::Text(format!("{}{}", ds(&a), ds(&b))));
                    }
                    _ => return Err("bytecode: type error in arithmetic".to_string()),
                }
            }
            Instr::Sub => binop(&mut stack, |a, b| a - b)?,
            Instr::Mul => binop(&mut stack, |a, b| a * b)?,
            Instr::Div => binop_div(&mut stack)?,
            Instr::Mod => {
                let b = stack.pop().and_then(|v| v.as_number());
                let a = stack.pop().and_then(|v| v.as_number());
                match (a, b) {
                    (Some(a), Some(b)) => {
                        if b == 0.0 { return Err("Modulo by zero".to_string()); }
                        stack.push(Value::Number(a % b));
                    }
                    _ => return Err("bytecode: type error in arithmetic".to_string()),
                }
            },
            Instr::Pow => binop(&mut stack, |a, b| a.powf(b))?,
            Instr::Neg => {
                if let Some(Value::Number(n)) = stack.pop() { stack.push(Value::Number(-n)); }
            }
            Instr::Eq => binop_cmp(&mut stack, |a, b| a == b),
            Instr::Ne => binop_cmp(&mut stack, |a, b| a != b),
            Instr::Lt => binop_cmp(&mut stack, |a, b| a < b),
            Instr::Le => binop_cmp(&mut stack, |a, b| a <= b),
            Instr::Gt => binop_cmp(&mut stack, |a, b| a > b),
            Instr::Ge => binop_cmp(&mut stack, |a, b| a >= b),
            Instr::Call(name, argc) => {
                // guard против битого байткода, argc > глубины стека > паника в debug.
                if *argc > stack.len() {
                    return Err("bytecode: stack underflow in Call".to_string());
                }
                // JIT-быстрый путь, пользовательская функция (кэш скомпил. тела)
                // Аргументы читаем как срез вершины стека (без split_off, дешевле в рекурсии).
                if let Some(f) = vm.functions.get(name) {
                    if let Some(cache) = &f.code {
                        let params = f.params.clone();
                        let reg_cache = cache.code_reg.clone();
                        let map_cache = cache.code.clone();
                        let arg_start = stack.len() - *argc;
                        // РЕГИСТРОВЫЙ РЕЖИМ, чисто числовое тело, без scope/HashMap.
                        // Снапшот fast_regs вызывающего (рекурсия) > bind > run > restore.
                        if let Some((cr, n)) = &reg_cache {
                            let n = *n;
                            if n > 0 && *argc >= n && stack[arg_start..].iter().take(n).all(|v| v.as_number().is_some()) {
                                let cr = Rc::clone(cr);
                                let mut saved = [0f64; 8];
                                for i in 0..n { saved[i] = vm.fast_reg(i as u16); }
                                for (i, v) in stack[arg_start..].iter().enumerate().take(n) {
                                    vm.set_fast_reg(i as u16, v.as_number().unwrap());
                                }
                                vm.call_depth += 1;
                                if vm.call_depth > 1000 {
                                    vm.call_depth -= 1;
                                    return Err("recursion depth limit exceeded".to_string());
                                }
                                let res = run_func(vm, &cr);
                                vm.call_depth -= 1;
                                for i in 0..n { vm.set_fast_reg(i as u16, saved[i]); }
                                let v = res?;
                                last = v.clone();
                                stack.truncate(arg_start);
                                stack.push(v);
                                ip += 1;
                                continue;
                            }
                        }
                        // MAP/scope-режим (базовый, общий случай)
                        if let Some(code) = &map_cache {
                            let code = Rc::clone(code);
                            // Лимит, ДО push'ей, иначе Err на лимите течёт скоупом
                            // и стеком вызовов (каждый хит = +1 кадр навсегда).
                            // Сообщение как у slow (без префикса bytecode,).
                            if vm.call_depth >= 1000 {
                                return Err("recursion depth limit exceeded".to_string());
                            }
                            let owned_args: Vec<Value> = stack.split_off(arg_start);
                            vm.call_stack.push(name.clone());
                            vm.push_scope();
                            vm.bind_params(&params, &owned_args)?;
                            vm.call_depth += 1;
                            let res = run_func(vm, &code);
                            vm.call_depth -= 1;
                            vm.pop_scope();
                            vm.call_stack.pop();
                            let v = res?;
                            last = v.clone();
                            stack.push(v);
                            ip += 1;
                            continue;
                        }
                    }
                }
                let args: Vec<Value> = stack.split_off(stack.len() - *argc);
                // JIT-быстрый путь, list_push/add/push(name, item)
                if *argc == 2 {
                    if let Value::Text(varname) = &args[0] {
                        // чтение списка, list_get/get(name, idx)
                        // БЕЗ vm.get_var(), он клонирует ВЕСЬ список (Value::get_var > cloned()).
                        // Берём заимствование и клонируем только прочитанный элемент.
                        if *name == "list_get" || *name == "get" {
                            let idx = resolve_cmd_arg(vm, &args[1]).as_number().map(|n| n as isize).unwrap_or(isize::MAX);
                            if let Some(Value::List(list)) = vm.variables.get(varname) {
                                // Отрицательные, с конца, как l[-1] и slow ~get.
                                let i = if idx < 0 { list.len() as isize + idx } else { idx };
                                if i >= 0 && (i as usize) < list.len() {
                                    let v = list[i as usize].clone();
                                    // гвард, display() аллоцирует, в скриптах (noecho) не форматируем зря
                                    if !vm.noecho { vm.echo(v.display()); }
                                    last = v.clone();
                                    stack.push(v);
                                    ip += 1;
                                    continue;
                                }
                                return Err("Index out of range".to_string());
                            }
                            return Err(format!("\x1B[91mList '{}' not found\x1B[0m", varname));
                        }
                        // длина, len/list_len(name)
                        if *name == "list_len" || *name == "len" {
                            let n = match vm.variables.get(varname) {
                                Some(Value::List(list)) => Some(list.len() as f64),
                                Some(Value::Text(t)) => Some(t.chars().count() as f64),
                                _ => None,
                            };
                            if let Some(n) = n {
                                let v = Value::Number(n);
                                if !vm.noecho { vm.echo(Value::fmt_num(n)); }
                                last = v.clone();
                                stack.push(v);
                                ip += 1;
                                continue;
                            }
                        }
                        // чтение словаря, dict_get/dget(name, key)
                        // Тот же приём, borrow > clone только значения, не всего словаря.
                        if *name == "dict_get" || *name == "dget" {
                            let key = resolve_cmd_arg(vm, &args[1]).display();
                            if let Some(Value::Dict(dict)) = vm.variables.get(varname) {
                                if let Some(v) = dict.get(&key) {
                                    let v = v.clone();
                                    if !vm.noecho { vm.echo(v.display()); }
                                    last = v.clone();
                                    stack.push(v);
                                    ip += 1;
                                    continue;
                                }
                                return Err(format!("Key '{}' not found", key));
                            }
                            return Err(format!("\x1B[91mDict '{}' not found\x1B[0m", varname));
                        }
                        if *name == "list_push" || *name == "add" || *name == "push" {
                            let item = resolve_cmd_arg(vm, &args[1]);
                            if let Some(Value::List(list)) = vm.get_var_mut(varname) {
                                list.push(item);
                                last = Value::Bool(true);
                                stack.push(last.clone());
                                ip += 1;
                                continue;
                            }
                            return Err(format!("\x1B[91mList '{}' not found\x1B[0m", varname));
                        }
                    }
                }
                // JIT-быстрый путь, dict_set/dset(name, key, val)
                if *argc == 3 {
                    if let Value::Text(varname) = &args[0] {
                        if *name == "dict_set" || *name == "dset" {
                            let key = resolve_cmd_arg(vm, &args[1]).display();
                            let val = resolve_cmd_arg(vm, &args[2]);
                            if let Some(Value::Dict(dict)) = vm.get_var_mut(varname) {
                                dict.insert(key, val);
                                last = Value::Bool(true);
                                stack.push(last.clone());
                                ip += 1;
                                continue;
                            }
                            return Err(format!("\x1B[91mDict '{}' not found\x1B[0m", varname));
                        }
                    }
                }
                // generic fallback
                match crate::main::eval::eval_cmd_fast(vm, name, &args) {
                    Ok(v) => { last = v.clone(); stack.push(v); }
                    Err(e) => {
                        if e.starts_with("__break__") || e.starts_with("__continue__") { return Err(e); }
                        return Err(e);
                    }
                }
            }
            Instr::Label(_) => {}
            Instr::JmpBackIfZero(target) => {
                let v = stack.pop().unwrap_or(Value::None);
                if v.truthy() {
                    ip = *target;
                    continue;
                }
            }
            Instr::JmpIfZero(target) => {
                let v = stack.pop().unwrap_or(Value::None);
                if !v.truthy() { ip = *target; continue; }
            }
            Instr::Jmp(target) => { ip = *target; continue; }
            Instr::Nop => {}
            Instr::LitBool(b) => stack.push(Value::Bool(*b)),
            // FAST REGISTER ops
            Instr::SetConstR(idx, n) => vm.set_fast_reg(*idx, *n),
            Instr::CopyR(dst, src) => {
                let v = vm.fast_reg(*src);
                vm.set_fast_reg(*dst, v);
            }
            Instr::AddConstR(idx, n) => vm.add_fast_reg(*idx, *n),
            Instr::SubConstR(idx, n) => vm.add_fast_reg(*idx, -*n),
            Instr::MulConstR(idx, n) => vm.mul_fast_reg(*idx, *n),
            Instr::DivConstR(idx, n) => vm.div_fast_reg(*idx, *n),
            Instr::JrLtConst(idx, n, target) => {
                if vm.fast_reg(*idx) < *n { ip = *target; continue; }
            }
            Instr::JrGeConst(idx, n, target) => {
                if vm.fast_reg(*idx) >= *n { ip = *target; continue; }
            }
            Instr::SyncRegVar(name, idx) => {
                let v = vm.fast_reg(*idx);
                vm.set_var(name, Value::Number(v));
            }
            Instr::LoadRegR(idx) => {
                stack.push(Value::Number(vm.fast_reg(*idx)));
            }
            Instr::StoreRegR(idx) => {
                let v = stack.pop().unwrap_or(Value::Number(0.0));
                let n = match v { Value::Number(n) => n, _ => 0.0 };
                vm.set_fast_reg(*idx, n);
            }
            Instr::AddRegR(dst, src) => {
                let a = vm.fast_reg(*dst);
                let b = vm.fast_reg(*src);
                vm.set_fast_reg(*dst, a + b);
            }
            Instr::SubRegR(dst, src) => {
                let a = vm.fast_reg(*dst);
                let b = vm.fast_reg(*src);
                vm.set_fast_reg(*dst, a - b);
            }
            Instr::FusedLoop { step, limit, lin, add } => {
                run_fused_loop(vm, *step, *limit, lin, add)?;
            }
            Instr::FastLoopCounter { limit } => {
                run_fast_loop_counter(vm, *limit)?;
            }
            Instr::FastLoopAcc { limit, acc_reg } => {
                run_fast_loop_acc(vm, *limit, *acc_reg)?;
            }
            Instr::FusedLoopG { limit, ops, tail, nregs } => {
                run_fused_loop_g(vm, *limit, ops, tail, *nregs)?;
            }
            Instr::Slow(line) => {
                match crate::main::eval::evaluate(vm, line) {
                    Ok(v) => { last = v; }
                    Err(e) => { return Err(e); }
                }
            }
            Instr::SlowBlock(text) => {
                let blines: Vec<String> = text.lines().map(|l| l.trim_end_matches('\r').to_string()).collect();
                match vm.exec_lines(&blines, 0) {
                    Ok((_, v)) => { last = v; }
                    Err(e) => { return Err(e); }
                }
            }
            Instr::ForBegin { var, source, body, end } => {
                let src = vm.eval_for_source(source)?;
                match src {
                    ForSource::Range(a, b) => {
                        let step = if a <= b { 1 } else { -1 };
                        if (step > 0 && a > b) || (step < 0 && a < b) {
                            ip = *end;
                            continue;
                        } else {
                            vm.for_stack.push(ForIter::Range { i: a, step, to: b });
                            vm.set_var(var, Value::Number(a as f64));
                            ip = *body;
                            continue;
                        }
                    }
                    ForSource::List(items) => {
                        if items.is_empty() {
                            ip = *end;
                            continue;
                        } else {
                            vm.for_stack.push(ForIter::List { i: 0, items });
                            vm.set_var(var, vm.for_stack.last().map(|f| match f {
                                ForIter::List { items, .. } => items[0].clone(),
                                _ => Value::None,
                            }).unwrap_or(Value::None));
                            ip = *body;
                            continue;
                        }
                    }
                }
            }
            Instr::ForNext { var, body } => {
                let action = {
                    let fr = vm.for_stack.last_mut().ok_or("bytecode: for: stack underflow")?;
                    match fr {
                        ForIter::Range { i, step, to } => {
                            *i += *step;
                            if (*step > 0 && *i > *to) || (*step < 0 && *i < *to) { None }
                            else { Some(Value::Number(*i as f64)) }
                        }
                        ForIter::List { i, items } => {
                            *i += 1;
                            if *i < items.len() { Some(items[*i].clone()) } else { None }
                        }
                    }
                };
                match action {
                    Some(v) => { vm.set_var(var, v); ip = *body; continue; }
                    None => { vm.for_stack.pop(); }
                }
            }
            Instr::ForBreak { end } => {
                vm.for_stack.pop();
                ip = *end;
                continue;
            }
        }
        ip += 1;
    }
    Ok(if ret_top { stack.last().unwrap_or(&last).clone() } else { last })
}

fn binop(stack: &mut Vec<Value>, f: impl Fn(f64, f64) -> f64) -> Result<(), String> {
    let b = stack.pop().and_then(|v| v.as_number());
    let a = stack.pop().and_then(|v| v.as_number());
    match (a, b) {
        (Some(a), Some(b)) => { stack.push(Value::Number(f(a, b))); Ok(()) }
        _ => {
            #[cfg(debug_assertions)]
            if std::env::var("TREK_DUMP_BC").is_ok() { eprintln!("  binop type error a={:?} b={:?}", a, b); }
            Err("bytecode: type error in arithmetic".to_string())
        }
    }
}

fn binop_div(stack: &mut Vec<Value>) -> Result<(), String> {
    let b = stack.pop().and_then(|v| v.as_number());
    let a = stack.pop().and_then(|v| v.as_number());
    match (a, b) {
        (Some(a), Some(b)) if b != 0.0 => { stack.push(Value::Number(a / b)); Ok(()) }
        (Some(_), Some(_)) => Err("Division by zero".to_string()),
        _ => Err("bytecode: type error in arithmetic".to_string()),
    }
}

fn binop_cmp(stack: &mut Vec<Value>, f: impl Fn(f64, f64) -> bool) {
    let b = stack.pop().and_then(|v| v.as_number()).unwrap_or(0.0);
    let a = stack.pop().and_then(|v| v.as_number()).unwrap_or(0.0);
    stack.push(Value::Bool(f(a, b)));
}
