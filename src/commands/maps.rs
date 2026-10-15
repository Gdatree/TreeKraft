use crate::main::vm::{VM, Value, MapObject};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~map_new(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() >= 3 {
            let name = p[0].trim().trim_matches('"'); let w = vm.eval_value(&p[1]).ok()?.as_number()? as usize; let h = vm.eval_value(&p[2]).ok()?.as_number()? as usize;
            let fill = if p.len() > 3 { p[3].trim().trim_matches('"').chars().next().unwrap_or('.') } else { '.' };
            vm.maps.insert(name.to_string(), vec![vec![fill; w]; h]);
            vm.echo(&format!("Map '{}' created ({}x{})", name, w, h)); Some(Ok(Value::None)) } else { None }
    } else if code.starts_with("~tile(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len()-1], ','); if p.len() >= 3 {
            let name = p[0].trim(); let x = vm.eval_value(&p[1]).ok()?.as_number()? as usize; let y = vm.eval_value(&p[2]).ok()?.as_number()? as usize;
            if let Some(map) = vm.maps.get(name) { if y < map.len() && x < map[0].len() { let r = map[y][x]; vm.echo(r); Some(Ok(Value::Text(r.to_string()))) } else { Some(Err("Out of bounds".to_string())) } } else { Some(Err("Map not found".to_string())) }
        } else { None }
    } else if code.starts_with("~settile(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() >= 4 {
            let name = p[0].trim().to_string(); let x = vm.eval_value(&p[1]).ok()?.as_number()? as usize; let y = vm.eval_value(&p[2]).ok()?.as_number()? as usize;
            let ch = VM::tq(&p[3]).chars().next().unwrap_or(' ');
            if let Some(map) = vm.maps.get_mut(&name) { if y < map.len() && x < map[0].len() { map[y][x] = ch; Some(Ok(Value::None)) } else { Some(Err("Out of bounds".to_string())) } } else { Some(Err("Map not found".to_string())) }
        } else { None }
    } else if code.starts_with("~drawmap(") && code.ends_with(')') {
        let name = code[9..code.len()-1].trim(); self::draw_map(vm, name); Some(Ok(Value::None))
    } else if code.starts_with("~map_size(") && code.ends_with(')') {
        let name = code[10..code.len()-1].trim();
        if let Some(map) = vm.maps.get(name) { let r = Value::List(vec![Value::Number(map[0].len() as f64), Value::Number(map.len() as f64)]); vm.echo(r.display()); Some(Ok(r)) } else { Some(Err("Map not found".to_string())) }
    } else if code.starts_with("~map_fill(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim().to_string(); let ch = VM::tq(&p[1]).chars().next().unwrap_or('.');
            if let Some(map) = vm.maps.get_mut(&name) { for row in map.iter_mut() { for c in row.iter_mut() { *c = ch; } } Some(Ok(Value::None)) } else { Some(Err("Map not found".to_string())) }
        } else { None }
    } else if code.starts_with("~obj(") && code.ends_with(')') || code.starts_with("~spawn(") && code.ends_with(')') {
        let prefix = if code.starts_with("~obj") { "~obj" } else { "~spawn" };
        let p = VM::split_top_level(&code[prefix.len()+1..code.len()-1], ','); if p.len() >= 4 {
            let name = p[0].trim().to_string(); let x = vm.eval_value(&p[1]).ok()?.as_number()? as usize; let y = vm.eval_value(&p[2]).ok()?.as_number()? as usize;
            let ch = VM::tq(&p[3]).chars().next().unwrap_or('?');
            vm.objects.insert(name, MapObject { ch, x, y }); Some(Ok(Value::None)) } else { None }
    } else if code.starts_with("~objmove(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() >= 3 {
            let name = p[0].trim(); let dx = vm.eval_value(&p[1]).ok()?.as_number()? as i32; let dy = vm.eval_value(&p[2]).ok()?.as_number()? as i32;
            if let Some(obj) = vm.objects.get_mut(name) { obj.x = (obj.x as i32 + dx) as usize; obj.y = (obj.y as i32 + dy) as usize; Some(Ok(Value::None)) } else { Some(Err("Object not found".to_string())) }
        } else { None }
    } else if code.starts_with("~objdel(") && code.ends_with(')') {
        let name = code[8..code.len()-1].trim(); vm.objects.remove(name); Some(Ok(Value::None))
    } else if code.starts_with("~showmap") && (code.len() == 8 || (code.starts_with("~showmap(") && code.ends_with(')'))) {
        let name = if code.len() > 9 { code[9..code.len()-1].trim() } else { "_main" };
        self::draw_map(vm, name); Some(Ok(Value::None))
    } else if code.starts_with("~loadmap(") && code.ends_with(')') {
        let path = VM::tq(&code[9..code.len()-1]);
        match std::fs::read_to_string(&path) { Ok(c) => { let name = std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "map".to_string()); let map: Vec<Vec<char>> = c.lines().map(|l| l.chars().collect()).collect(); vm.maps.insert(name.clone(), map); println!("Map '{}' loaded", name); Some(Ok(Value::None)) } Err(e) => Some(Err(e.to_string())) }
    } else {
        None
    }
}

fn draw_map(vm: &VM, name: &str) {
    if let Some(map) = vm.maps.get(name) {
        for row in map { let line: String = row.iter().collect(); println!("{}", line); }
    }
}
