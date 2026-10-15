use crate::main::vm::{VM, Value};

fn gt(vm: &mut VM, s: &str) -> Option<String> { vm.eval_value(s).ok().and_then(|v| v.as_text()) }

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! tp1 {
        ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let s = gt(vm, &code[$p.len()..code.len()-1])?; let r = ($f)(&s); vm.echo(format!("{}", r)); return Some(Ok(Value::Text(r))); } };
    }
    macro_rules! tp2 {
        ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let s = gt(vm, &p[0])?; let n = match vm.eval_expr(&p[1]) { Ok(v) => v as usize, Err(e) => return Some(Err(e)) }; let r = ($f)(&s, n); vm.echo(format!("{}", r)); return Some(Ok(Value::Text(r))); } } };
    }

    tp1!("~lowercase(", 11, |s: &str| s.to_lowercase());
    tp1!("~uppercase(", 11, |s: &str| s.to_uppercase());
    tp1!("~capitalize(", 12, |s: &str| { let mut ch = s.chars(); match ch.next() { Some(c) => c.to_uppercase().collect::<String>() + ch.as_str(), None => String::new() } });
    tp1!("~sentence_case(", 15, |s: &str| { let mut r = String::new(); let mut new_s = true; for c in s.chars() { if c.is_alphabetic() && new_s { r.extend(c.to_uppercase()); new_s = false; } else { r.push(c); } if c == '.' || c == '?' || c == '!' { new_s = true; } } r });
    tp1!("~invert_case(", 13, |s: &str| s.chars().map(|c| if c.is_uppercase() { c.to_lowercase().next().unwrap_or(c) } else { c.to_uppercase().next().unwrap_or(c) }).collect());
    tp1!("~toggle_case(", 13, |s: &str| s.chars().map(|c| if c.is_uppercase() { c.to_lowercase().next().unwrap_or(c) } else { c.to_uppercase().next().unwrap_or(c) }).collect());
    tp1!("~wide_text(", 11, |s: &str| s.chars().map(|c| format!("{} ", c)).collect::<String>().trim().to_string());
    tp1!("~narrow_text(", 13, |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect());
    tp1!("~reverse_words(", 15, |s: &str| s.split_whitespace().rev().collect::<Vec<_>>().join(" "));
    tp1!("~shuffle_words(", 15, |s: &str| { let mut w: Vec<&str> = s.split_whitespace().collect(); use rand::seq::SliceRandom; w.shuffle(&mut rand::thread_rng()); w.join(" ") });
    tp1!("~rainbow_text(", 14, |s: &str| s.chars().enumerate().map(|(i,c)| format!("\x1B[38;5;{}m{}", i as u8 % 216 + 16, c)).collect::<String>() + "\x1B[0m");
    tp1!("~bubble_text(", 13, |s: &str| s.chars().map(|c| format!(" {} ", c)).collect::<String>());
    tp1!("~box_text(", 10, |s: &str| { let w = s.len() + 4; format!("┌{}┐\n│ {} │\n└{}┘", "─".repeat(w), s, "─".repeat(w)) });

    // Ниже собраны смена регистра и мелочи, чтобы их было легче искать
    tp1!("~spongebox(", 11, |s: &str| s.chars().enumerate().map(|(i, c)| if i % 2 == 0 { c.to_uppercase().collect::<String>() } else { c.to_lowercase().to_string() }).collect::<String>());
    tp1!("~striptags(", 11, |s: &str| { let mut out = String::new(); let mut in_tag = false; for c in s.chars() { if c == '<' { in_tag = true } else if c == '>' { in_tag = false } else if !in_tag { out.push(c) } } out });
    tp1!("~camel_to_snake(", 16, |s: &str| { let mut out = String::new(); for c in s.chars() { if c.is_uppercase() && !out.is_empty() { out.push('_') } out.extend(c.to_lowercase()); } out });
    tp1!("~snake_to_camel(", 16, |s: &str| { let mut out = String::new(); let mut cap = false; for c in s.chars() { if c == '_' { cap = true } else if cap { out.extend(c.to_uppercase()); cap = false } else { out.push(c) } } out });
    tp1!("~kebab_to_snake(", 16, |s: &str| s.replace('-', "_"));
    tp1!("~snake_to_kebab(", 16, |s: &str| s.replace('_', "-"));
    tp1!("~a1z26(", 7, |s: &str| s.chars().map(|c| if c.is_ascii_alphabetic() { (c.to_ascii_lowercase() as u32 - 'a' as u32 + 1).to_string() } else { c.to_string() }).collect::<Vec<_>>().join(" "));
    tp1!("~hexlify(", 9, |s: &str| s.bytes().map(|b| format!("{:02x}", b)).collect());
    tp1!("~unhexlify(", 11, |s: &str| { let b = s.as_bytes(); let mut r = String::new(); let mut i = 0; while i + 1 < b.len() { if let Ok(pair) = std::str::from_utf8(&b[i..i+2]) { if let Ok(v) = u8::from_str_radix(pair, 16) { r.push(v as char); } } i += 2; } r });
    tp1!("~deaccent(", 10, |s: &str| deaccent(s));
    tp1!("~sort_letters(", 14, |s: &str| { let mut v: Vec<char> = s.chars().collect(); v.sort(); v.into_iter().collect() });
    tp1!("~reverse_letters(", 17, |s: &str| s.chars().rev().collect());
    tp1!("~alternate_upper(", 17, |s: &str| s.chars().enumerate().map(|(i, c)| if i % 2 == 1 { c.to_uppercase().collect::<String>() } else { c.to_lowercase().to_string() }).collect());
    tp1!("~no_digits(", 11, |s: &str| s.chars().filter(|c| !c.is_ascii_digit()).collect());
    tp1!("~single_line(", 13, |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" "));
    tp1!("~space_chars(", 13, |s: &str| s.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" "));
    tp1!("~quote_wrap(", 12, |s: &str| format!("\"{}\"", s));
    tp1!("~paren_wrap(", 12, |s: &str| format!("({})", s));
    tp1!("~braces_wrap(", 13, |s: &str| format!("[{}]", s));
    tp1!("~angle_wrap(", 12, |s: &str| format!("<{}>", s));

    // Ниже ширина и сдвиги, которым нужен второй числовой аргумент
    tp2!("~pad_left_chars(", 16, |s: &str, n: usize| format!("{:>w$}", s, w = n));
    tp2!("~pad_right_chars(", 17, |s: &str, n: usize| format!("{:<w$}", s, w = n));
    tp2!("~pad_center_chars(", 18, |s: &str, n: usize| { let pad = n.saturating_sub(s.len()); let l = pad / 2; format!("{}{}{}", " ".repeat(l), s, " ".repeat(pad - l)) });
    tp2!("~repeat_chars(", 14, |s: &str, n: usize| s.repeat(n.min(1000)));
    tp2!("~shift_chars(", 13, |s: &str, n: usize| s.chars().map(|c| { if c.is_ascii_alphabetic() { let base = if c.is_ascii_uppercase() { b'A' } else { b'a' }; (((c as u8 - base + (n % 26) as u8) % 26) + base) as char } else { c } }).collect());
    tp2!("~first_chars(", 13, |s: &str, n: usize| s.chars().take(n).collect());
    tp2!("~last_chars(", 12, |s: &str, n: usize| s.chars().rev().take(n).collect::<Vec<_>>().into_iter().rev().collect());
    tp2!("~drop_start(", 12, |s: &str, n: usize| s.chars().skip(n).collect());
    tp2!("~take_chars(", 12, |s: &str, n: usize| s.chars().take(n).collect());
    tp2!("~keep_long_words(", 17, |s: &str, n: usize| s.split_whitespace().filter(|w| w.chars().count() >= n).collect::<Vec<_>>().join(" "));
    tp2!("~drop_short_words(", 18, |s: &str, n: usize| s.split_whitespace().filter(|w| w.chars().count() < n).collect::<Vec<_>>().join(" "));

    // Ниже однотипные команды, которые нужны для проверки поведения
    tp1!("~sort_words(", 12, |s: &str| { let mut w: Vec<&str> = s.split_whitespace().collect(); w.sort(); w.join(" ") });
    tp1!("~swap_first_last(", 17, |s: &str| { let mut w: Vec<&str> = s.split_whitespace().collect(); let n = w.len(); if n >= 2 { w.swap(0, n - 1); } w.join(" ") });
    tp1!("~escape_quotes(", 15, |s: &str| s.replace('"', "\\\""));
    tp1!("~unescape_quotes(", 17, |s: &str| s.replace("\\\"", "\""));
    tp1!("~clean_whitespace(", 18, |s: &str| { let mut out = String::new(); let mut sp = false; for c in s.chars() { if c.is_whitespace() { if !sp { out.push(' '); sp = true; } } else { out.push(c); sp = false; } } out.trim().to_string() });
    tp1!("~keep_alnum(", 12, |s: &str| s.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect());
    tp1!("~capitalize_all_words(", 22, |s: &str| s.split_whitespace().map(|w| { let mut ch = w.chars(); match ch.next() { Some(c) => c.to_uppercase().collect::<String>() + ch.as_str(), None => String::new() } }).collect::<Vec<_>>().join(" "));
    tp1!("~mask_text(", 11, |s: &str| "*".repeat(s.chars().count()));
    tp2!("~center_text(", 13, |s: &str, n: usize| { let pad = n.saturating_sub(s.chars().count()); let l = pad / 2; format!("{}{}{}", " ".repeat(l), s, " ".repeat(pad - l)) });
    tp2!("~reverse_chunks(", 16, |s: &str, n: usize| { let mut v: Vec<String> = s.chars().collect::<Vec<_>>().chunks(n.max(1)).map(|c| c.iter().collect()).collect(); v.reverse(); v.join(" ") });
    tp2!("~take_word_count(", 17, |s: &str, n: usize| s.split_whitespace().take(n).collect::<Vec<_>>().join(" "));
    tp2!("~drop_word_count(", 17, |s: &str, n: usize| s.split_whitespace().skip(n).collect::<Vec<_>>().join(" "));
    tp2!("~text_repeat_each(", 18, |s: &str, n: usize| s.split_whitespace().flat_map(|w| std::iter::repeat(w).take(n)).collect::<Vec<_>>().join(" "));

    if code.starts_with("~repeat_text(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ','); if p.len() >= 2 {
            let s = gt(vm, &p[0])?; let n = match vm.eval_expr(&p[1]) { Ok(v) => v as usize, Err(e) => return Some(Err(e)) };
            vm.echo(format!("{}", s.repeat(n))); Some(Ok(Value::Text(s.repeat(n))))
        } else { None }
    } else if code.starts_with("~join_lines(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 2 {
            let sep = gt(vm, &p[0])?; let name = p[1].trim();
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let r: String = list.iter().map(|v| v.display()).collect::<Vec<_>>().join(&sep);
            vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
        } else { None }
    } else if code.starts_with("~split_lines(") && code.ends_with(')') {
        let s = gt(vm, &code[13..code.len()-1])?;
        let list: Vec<Value> = s.lines().map(|l| Value::Text(l.to_string())).collect();
        let r = Value::List(list); vm.echo(format!("{}", r.display())); Some(Ok(r))
    } else if code.starts_with("~tabulate(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 2 {
            let sep = gt(vm, &p[0])?; let name = p[1].trim();
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            for row in &list { if let Value::List(cells) = row { println!("{}", cells.iter().map(|c| c.display()).collect::<Vec<_>>().join(&sep)); } }
            Some(Ok(Value::None))
        } else { None }
    } else {
        None
    }
}

fn deaccent(s: &str) -> String {
    let map: &[(char, char)] = &[
        ('á','a'),('à','a'),('â','a'),('ã','a'),('ä','a'),('å','a'),('ā','a'),
        ('é','e'),('è','e'),('ê','e'),('ë','e'),('ē','e'),
        ('í','i'),('ì','i'),('î','i'),('ï','i'),('ī','i'),
        ('ó','o'),('ò','o'),('ô','o'),('õ','o'),('ö','o'),('ō','o'),
        ('ú','u'),('ù','u'),('û','u'),('ü','u'),('ū','u'),
        ('ý','y'),('ÿ','y'),
        ('ç','c'),('ñ','n'),('š','s'),('ž','z'),('ł','l'),('đ','d'),
        ('Á','A'),('À','A'),('Â','A'),('Ã','A'),('Ä','A'),('Å','A'),
        ('É','E'),('È','E'),('Ê','E'),('Ë','E'),
        ('Í','I'),('Ì','I'),('Î','I'),('Ï','I'),
        ('Ó','O'),('Ò','O'),('Ô','O'),('Õ','O'),('Ö','O'),
        ('Ú','U'),('Ù','U'),('Û','U'),('Ü','U'),
        ('Ç','C'),('Ñ','N'),('Š','S'),('Ž','Z')
    ];
    s.chars().map(|c| map.iter().find(|&&(a, _)| a == c).map(|&(_, b)| b).unwrap_or(c)).collect()
}
