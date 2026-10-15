use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if VM::matches_cmd_prefix(code, "rot13(") {
        let a = VM::extract_cmd_args(code, "rot13(")?; let s = vm.eval_value(&a).ok()?.as_text()?;
        let r: String = s.chars().map(|c| match c { 'a'..='m'|'A'..='M'=> ((c as u8)+13) as char, 'n'..='z'|'N'..='Z'=> ((c as u8)-13) as char, _=>c }).collect();
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if VM::matches_cmd_prefix(code, "caesar_cipher(") {
        let a = VM::extract_cmd_args(code, "caesar_cipher(")?; let p = VM::split_top_level(&a, ',');
        if p.len() >= 2 { let s = vm.eval_value(&p[0]).ok()?.as_text()?; let n = vm.eval_expr(&p[1]).ok()? as i32;
            let r: String = s.chars().map(|c| if c.is_ascii_alphabetic() { let base = if c.is_ascii_lowercase() { b'a' } else { b'A' }; ((((c as u8).wrapping_sub(base)).wrapping_add(n as u8)) % 26 + base) as char } else { c }).collect();
            vm.echo(&r); Some(Ok(Value::Text(r))) } else { None }
    } else if VM::matches_cmd_prefix(code, "base64enc(") || VM::matches_cmd_prefix(code, "base64_encode(") {
        let prefix = if code.contains("base64enc") { "base64enc" } else { "base64_encode" };
        let a = VM::extract_cmd_args(code, prefix)?; let s = vm.eval_value(&a).ok()?.as_text()?;
        use base64::Engine; let r = base64::engine::general_purpose::STANDARD.encode(&s.as_bytes());
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if VM::matches_cmd_prefix(code, "base64dec(") || VM::matches_cmd_prefix(code, "base64_decode(") {
        let prefix = if code.contains("base64dec") { "base64dec" } else { "base64_decode" };
        let a = VM::extract_cmd_args(code, prefix)?; let s = vm.eval_value(&a).ok()?.as_text()?;
        use base64::Engine; match base64::engine::general_purpose::STANDARD.decode(&s) { Ok(b) => { let r = String::from_utf8_lossy(&b).to_string(); vm.echo(&r); Some(Ok(Value::Text(r))) } Err(e) => Some(Err(e.to_string())) }
    } else if code.starts_with("~md5(") && code.ends_with(')') {
        let s = vm.eval_value(&code[5..code.len()-1]).ok()?.as_text()?;
        let mut h: u32 = 0x67452301u32.wrapping_add(0xEFCDAB89).wrapping_add(0x98BADCFE).wrapping_add(0x10325476);
        for b in s.bytes() { h = h.wrapping_mul(0x01000193u32).wrapping_add(b as u32); }
        let r = format!("{:08x}", h);
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~sha256(") && code.ends_with(')') {
        let s = vm.eval_value(&code[8..code.len()-1]).ok()?.as_text()?;
        let mut h: u64 = 0x6A09E667F3BCC908;
        for b in s.bytes() { h = h.wrapping_mul(0x100000001B3).wrapping_add(b as u64); }
        let r = format!("{:016x}", h);
        vm.echo(&r); Some(Ok(Value::Text(r)))
    } else if code.starts_with("~encrypt(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() >= 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?; let key = vm.eval_value(&p[1]).ok()?.as_text()?;
            if key.is_empty() { return Some(Err("encrypt: empty key".to_string())); }
            let r: String = s.bytes().enumerate().map(|(i, b)| (b ^ key.as_bytes()[i % key.len()]) as char).collect();
            use base64::Engine; let enc = base64::engine::general_purpose::STANDARD.encode(r.as_bytes()); vm.echo(&enc); Some(Ok(Value::Text(enc))) } else { None }
    } else if code.starts_with("~decrypt(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() >= 2 {
            let s = vm.eval_value(&p[0]).ok()?.as_text()?; let key = vm.eval_value(&p[1]).ok()?.as_text()?;
            if key.is_empty() { return Some(Err("decrypt: empty key".to_string())); }
            use base64::Engine; let bytes = base64::engine::general_purpose::STANDARD.decode(&s).ok()?;
            let r: String = bytes.iter().enumerate().map(|(i, &b)| (b ^ key.as_bytes()[i % key.len()]) as char).collect();
            vm.echo(&r); Some(Ok(Value::Text(r))) } else { None }
    } else if code.starts_with("~random_bytes(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[14..code.len()-1]).ok()? as usize;
        use base64::Engine; let bytes: Vec<u8> = (0..n).map(|_| rand::random::<u8>()).collect();
        let r = base64::engine::general_purpose::STANDARD.encode(&bytes); vm.echo(&r); Some(Ok(Value::Text(r)))
    } else {
        None
    }
}
