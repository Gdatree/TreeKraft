use crate::main::vm::VM;

#[derive(Clone, Debug)]
pub enum ExprOp {
    PushNum(f64),
    PushVar(String),
    Add, Sub, Mul, Div, Mod, Pow,
    And, Or, Xor, Shl, Shr, Not,
    Neg,
}

#[derive(Clone, Copy, PartialEq)]
enum Tok { Plus, Minus, Mul, Div, Mod, Pow, Shl, Shr, And, Or, Xor, Not, Neg, Lparen }

impl Tok {
    fn prec(self) -> i32 {
        match self {
            Tok::Not | Tok::Neg => 6,
            Tok::Pow => 5,
            Tok::Mul | Tok::Div | Tok::Mod => 4,
            Tok::Plus | Tok::Minus => 3,
            Tok::Shl | Tok::Shr => 2,
            Tok::And => 1,
            Tok::Or | Tok::Xor => 0,
            Tok::Lparen => -1,
        }
    }
    fn to_op(self) -> Option<ExprOp> {
        match self {
            Tok::Plus => Some(ExprOp::Add), Tok::Minus => Some(ExprOp::Sub),
            Tok::Mul => Some(ExprOp::Mul), Tok::Div => Some(ExprOp::Div),
            Tok::Mod => Some(ExprOp::Mod), Tok::Pow => Some(ExprOp::Pow),
            Tok::Shl => Some(ExprOp::Shl), Tok::Shr => Some(ExprOp::Shr),
            Tok::And => Some(ExprOp::And), Tok::Or => Some(ExprOp::Or),
            Tok::Xor => Some(ExprOp::Xor), Tok::Not => Some(ExprOp::Not),
            Tok::Neg => Some(ExprOp::Neg),
            _ => None,
        }
    }
}

impl VM {
    pub fn compile_expr(&self, expr: &str) -> Result<Vec<ExprOp>, String> {
        let expr = expr.trim();
        if expr.is_empty() { return Err("Empty expression".to_string()); }

        let mut out: Vec<ExprOp> = Vec::new();
        let mut ops: Vec<Tok> = Vec::new();
        let chars: Vec<char> = expr.chars().collect();
        let mut i = 0usize;
        let mut xop = true;

        while i < chars.len() {
            let c = chars[i];
            if c.is_whitespace() { i += 1; continue; }
            if c == '(' { ops.push(Tok::Lparen); xop = true; i += 1; continue; }
            if c == ')' {
                while let Some(op) = ops.pop() { if op == Tok::Lparen { break; } if let Some(o) = op.to_op() { out.push(o); } }
                xop = false; i += 1; continue;
            }
            if c.is_ascii_digit() || c == '.' || (c == '-' && xop && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') { i += 1; }
                let ns: String = chars[start..i].iter().collect();
                out.push(ExprOp::PushNum(ns.parse::<f64>().map_err(|_| format!("Bad number: {}", ns))?));
                xop = false; continue;
            }
            if c.is_ascii_alphabetic() || c == '_' {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || (chars[i] == '.' && i+1 < chars.len() && chars[i+1].is_ascii_alphabetic())) { i += 1; }
                let name: String = chars[start..i].iter().collect();
                out.push(ExprOp::PushVar(name));
                xop = false; continue;
            }
            // унарный минус на переменной/скобке, -x, -(a+b), a * -b, -.5
            // (числовые литералы -5/-5.5 уже обработаны веткой выше как PushNum(-5))
            if c == '-' && xop && i + 1 < chars.len() && !chars[i + 1].is_ascii_digit() {
                ops.push(Tok::Neg); xop = true; i += 1; continue;
            }
            // двух-символьные операторы, <<, >>, ^^
            let two = if i + 1 < chars.len() { format!("{}{}", c, chars[i+1]) } else { String::new() };
            let two_tok = match two.as_str() { "<<" => Some(Tok::Shl), ">>" => Some(Tok::Shr), "^^" => Some(Tok::Xor), _ => None };
            if let Some(t) = two_tok {
                while let Some(&top) = ops.last() { if top != Tok::Lparen && top.prec() >= t.prec() { ops.pop().unwrap().to_op().map(|o| out.push(o)); } else { break; } }
                ops.push(t); xop = true; i += 2; continue;
            }
            // унарный битовый NOT
            if c == '~' && xop {
                while let Some(&top) = ops.last() { if top != Tok::Lparen && top.prec() > Tok::Not.prec() { ops.pop().unwrap().to_op().map(|o| out.push(o)); } else { break; } }
                ops.push(Tok::Not); xop = true; i += 1; continue;
            }
            let one_tok = match c {
                '+' => Some(Tok::Plus), '-' => Some(Tok::Minus),
                '*' => Some(Tok::Mul), '/' => Some(Tok::Div),
                '%' => Some(Tok::Mod), '^' => Some(Tok::Pow),
                '&' => Some(Tok::And), '|' => Some(Tok::Or),
                _ => None,
            };
            if let Some(t) = one_tok {
                while let Some(&top) = ops.last() { if top != Tok::Lparen && top.prec() >= t.prec() { ops.pop().unwrap().to_op().map(|o| out.push(o)); } else { break; } }
                ops.push(t); xop = true; i += 1; continue;
            }
            return Err(format!("Unknown token in expression: '{}'", c));
        }
        while let Some(op) = ops.pop() { if op != Tok::Lparen { if let Some(o) = op.to_op() { out.push(o); } } }
        if out.is_empty() { return Err("Empty expression".to_string()); }
        Ok(out)
    }

    pub fn exec_expr_ops(&self, ops: &[ExprOp]) -> Result<f64, String> {
        let mut stack: Vec<f64> = Vec::with_capacity(ops.len().min(32));
        for op in ops {
            match op {
                ExprOp::PushNum(n) => stack.push(*n),
                ExprOp::PushVar(name) => {
                    let v = self.get_var(name).ok_or_else(|| format!("Unknown variable: {}", name))?;
                    stack.push(v.as_number().ok_or_else(|| format!("'{}' not numeric", name))?);
                }
                ExprOp::Add => { let b = stack.pop().ok_or("Missing operand")?; let a = stack.pop().ok_or("Missing operand")?; stack.push(a + b); }
                ExprOp::Sub => { let b = stack.pop().ok_or("Missing operand")?; let a = stack.pop().ok_or("Missing operand")?; stack.push(a - b); }
                ExprOp::Mul => { let b = stack.pop().ok_or("Missing operand")?; let a = stack.pop().ok_or("Missing operand")?; stack.push(a * b); }
                ExprOp::Div => { let b = stack.pop().ok_or("Missing operand")?; let a = stack.pop().ok_or("Missing operand")?; if b == 0.0 { return Err("Division by zero".to_string()); } stack.push(a / b); }
                ExprOp::Mod => { let b = stack.pop().ok_or("Missing operand")?; let a = stack.pop().ok_or("Missing operand")?; if b == 0.0 { return Err("Modulo by zero".to_string()); } stack.push(a % b); }
                ExprOp::Pow => { let b = stack.pop().ok_or("Missing operand")?; let a = stack.pop().ok_or("Missing operand")?; stack.push(a.powf(b)); }
                ExprOp::And => { let b = stack.pop().ok_or("Missing operand")? as i64; let a = stack.pop().ok_or("Missing operand")? as i64; stack.push((a & b) as f64); }
                ExprOp::Or => { let b = stack.pop().ok_or("Missing operand")? as i64; let a = stack.pop().ok_or("Missing operand")? as i64; stack.push((a | b) as f64); }
                ExprOp::Xor => { let b = stack.pop().ok_or("Missing operand")? as i64; let a = stack.pop().ok_or("Missing operand")? as i64; stack.push((a ^ b) as f64); }
                ExprOp::Shl => {
                    let b = stack.pop().ok_or("Missing operand")? as i64;
                    let a = stack.pop().ok_or("Missing operand")? as i64;
                    if !(0..64).contains(&b) { return Err(format!("Shift out of range: {} << {}", a, b)); }
                    stack.push((a << b) as f64);
                }
                ExprOp::Shr => {
                    let b = stack.pop().ok_or("Missing operand")? as i64;
                    let a = stack.pop().ok_or("Missing operand")? as i64;
                    if !(0..64).contains(&b) { return Err(format!("Shift out of range: {} >> {}", a, b)); }
                    stack.push((a >> b) as f64);
                }
                ExprOp::Not => { let a = stack.pop().ok_or("Missing operand")? as i64; stack.push((!a) as f64); }
                ExprOp::Neg => { let a = stack.pop().ok_or("Missing operand")?; stack.push(-a); }
            }
        }
        stack.pop().ok_or_else(|| "No result".to_string())
    }
}