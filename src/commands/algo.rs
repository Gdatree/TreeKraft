use crate::main::vm::{VM, Value};
use std::collections::{HashMap, VecDeque};

/// Идем по уже готовому списку смежности чтобы найти путь между вершинами
/// Флаг use_stack выбирает очередь для поиска в ширину или стек для поиска в глубину с обратным порядком соседей чтобы один код обслуживал обе стратегии
/// Возвращаем путь от старта до цели или пустоту если пути нет
fn walk_graph(adj: &HashMap<i64, Vec<i64>>, start: i64, target: i64, use_stack: bool) -> Option<Vec<Value>> {
    let mut deque = VecDeque::new();
    let mut visited = HashMap::new();
    deque.push_back(start);
    visited.insert(start, -1i64);
    while let Some(node) = if use_stack { deque.pop_back() } else { deque.pop_front() } {
        if node == target {
            let mut path = Vec::new();
            let mut cur = target;
            while cur != -1 {
                path.push(Value::Number(cur as f64));
                cur = *visited.get(&cur)?;
            }
            path.reverse();
            return Some(path);
        }
        if let Some(neighbors) = adj.get(&node) {
            if use_stack {
                for &next in neighbors.iter().rev() {
                    if !visited.contains_key(&next) { visited.insert(next, node); deque.push_back(next); }
                }
            } else if !neighbors.is_empty() {
                for &next in neighbors {
                    if !visited.contains_key(&next) { visited.insert(next, node); deque.push_back(next); }
                }
            }
        }
    }
    None
}

/// Превращаем словарь из переменной в таблицу смежности чтобы поиск работал по числам а не по строкам
fn read_adj(vm: &mut VM, graph_name: &str) -> Option<HashMap<i64, Vec<i64>>> {
    let graph = match vm.get_var(graph_name) { Some(Value::Dict(d)) => d, _ => return None };
    let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
    for (k, v) in &graph {
        let from = k.parse::<i64>().ok()?;
        let to_list = match v { Value::List(l) => l.iter().filter_map(|x| x.as_number().map(|n| n as i64)).collect(), _ => vec![] };
        adj.entry(from).or_default().extend(to_list);
    }
    Some(adj)
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~bfs(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len()-1], ',');
        if p.len() == 3 {
            let graph_name = p[0].trim();
            let start = vm.eval_expr(&p[1]).ok()? as i64;
            let target = vm.eval_expr(&p[2]).ok()? as i64;
            let adj = match read_adj(vm, graph_name) { Some(a) => a, None => return Some(Err("Graph not found (must be dict)".to_string())) };
            match walk_graph(&adj, start, target, false) {
                Some(path) => { let r = Value::List(path); vm.echo(r.display()); return Some(Ok(r)); }
                None => return Some(Err("Path not found".to_string())),
            }
        } else { None }
    } else if code.starts_with("~dfs(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len()-1], ',');
        if p.len() == 3 {
            let graph_name = p[0].trim();
            let start = vm.eval_expr(&p[1]).ok()? as i64;
            let target = vm.eval_expr(&p[2]).ok()? as i64;
            let adj = match read_adj(vm, graph_name) { Some(a) => a, None => return Some(Err("Graph not found (must be dict)".to_string())) };
            match walk_graph(&adj, start, target, true) {
                Some(path) => { let r = Value::List(path); vm.echo(r.display()); return Some(Ok(r)); }
                None => return Some(Err("Path not found".to_string())),
            }
        } else { None }
    } else if code.starts_with("~factorial(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[11..code.len()-1]).ok()? as u64;
        let r = (1..=n.min(20)).product::<u64>() as f64;
        vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~fibonacci(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[11..code.len()-1]).ok()? as u64;
        let (mut a, mut b) = (0u64, 1u64);
        for _ in 0..n { let t = a + b; a = b; b = t; }
        vm.echo(a); Some(Ok(Value::Number(a as f64)))
    } else if code.starts_with("~collatz(") && code.ends_with(')') {
        let mut n = vm.eval_expr(&code[9..code.len()-1]).ok()? as u64;
        let mut seq = Vec::new();
        seq.push(Value::Number(n as f64));
        while n != 1 { n = if n % 2 == 0 { n / 2 } else { n * 3 + 1 }; seq.push(Value::Number(n as f64)); }
        let r = Value::List(seq); vm.echo(r.display()); Some(Ok(r))
    } else if code.starts_with("~isqrt(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[7..code.len()-1]).ok()? as f64;
        let r = n.sqrt().floor();
        vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~is_perfect_square(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[19..code.len()-1]).ok()?;
        let s = n.sqrt().floor();
        let r = n > 0.0 && (s * s - n).abs() < 1e-9;
        vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~digital_root(") && code.ends_with(')') {
        let mut n = vm.eval_expr(&code[14..code.len()-1]).ok()? as i64;
        if n < 0 { n = -n; }
        while n >= 10 { n = n.abs().to_string().bytes().map(|b| (b - b'0') as i64).sum(); }
        let r = n as f64; vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~euler_totient(") && code.ends_with(')') {
        let mut n = vm.eval_expr(&code[15..code.len()-1]).ok()? as i64;
        let mut result = n; let mut x = n;
        let mut p = 2i64;
        while p * p <= x {
            if x % p == 0 { while x % p == 0 { x /= p; } result -= result / p; }
            p += 1;
        }
        if x > 1 { result -= result / x; }
        let r = result as f64; vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
    } else {
        None
    }
}
