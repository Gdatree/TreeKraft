use crate::main::vm::{VM, Value};
use crate::main::screen::Screen;

/// Экранная сетка для символьной графики чтобы рисовать точками и фигурами
pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // Карта команд файла чтобы быстро найти нужную группу
    // Создание экрана и точки через ldsc и ld а линии через ldm
    // Прямоугольники и текст через ldbox и ldtext а заливка и рамка через ldfill и ldborder
    // Чтение и очистка клетки через ldc и ldg а цвет спрайты и файлы через ldcolor ldsave ldload и ldsprite
    // Закрашенные фигуры и круги через ldrect_fill ldcirc и ldcirc_fill а прокрутка через ldscroll
    // Окружности дуги и эффекты через ldcircle ldarc ldfade ldshake ldstars ldsnow и ldrain

    if code.starts_with("~ldsc(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() >= 2 {
            let w = match vm.eval_value(&p[0]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(80.0) as usize;
            let h = match vm.eval_value(&p[1]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(24.0) as usize;
            let border = if p.len() > 2 {
                VM::tq(&p[2]).chars().next().unwrap_or('\0')
            } else {
                '\0'
            };
            let fill = if p.len() > 3 {
                VM::tq(&p[3]).chars().next().unwrap_or(' ')
            } else {
                ' '
            };
            vm.screen = Some(Screen::new(w, h, border, fill));
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~ld(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[4..code.len() - 1], ',');
        if p.len() >= 3 {
            let x = match vm.eval_value(&p[0]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            let y = match vm.eval_value(&p[1]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            let ch = VM::tq(&p[2]).chars().next().unwrap_or(' ');
            if let Some(ref mut scr) = vm.screen {
                scr.set(x, y, ch);
            }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~ldtext(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() >= 3 {
            let x = match vm.eval_value(&p[0]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            let y = match vm.eval_value(&p[1]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            let text = match vm.eval_value(&p[2]) { Ok(v) => v.as_text(), Err(_) => None }.unwrap_or_default();
            if let Some(ref mut scr) = vm.screen {
                scr.draw_text(x, y, &text);
            }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~ldbox(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() >= 5 {
            let x = match vm.eval_value(&p[0]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            let y = match vm.eval_value(&p[1]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            let w = match vm.eval_value(&p[2]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            let h = match vm.eval_value(&p[3]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            let ch = VM::tq(&p[4]).chars().next().unwrap_or('#');
            if let Some(ref mut scr) = vm.screen {
                scr.draw_box(x, y, w, h, ch);
            }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~ldfill(") && code.ends_with(')') {
        let ch = VM::tq(&code[8..code.len() - 1]).chars().next().unwrap_or(' ');
        if let Some(ref mut scr) = vm.screen {
            scr.fill(ch);
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~ldc(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() >= 2 {
            let x = match vm.eval_value(&p[0]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            let y = match vm.eval_value(&p[1]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            if let Some(ref mut scr) = vm.screen {
                scr.set(x, y, scr.empty);
            }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~ldg(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() >= 2 {
            let x = match vm.eval_value(&p[0]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            let y = match vm.eval_value(&p[1]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as usize;
            if let Some(ref scr) = vm.screen {
                if y < scr.height && x < scr.width {
                    let ch = scr.buffer[y][x];
                    vm.echo(ch);
                    return Some(Ok(Value::Text(ch.to_string())));
                }
            }
        }
        return Some(Ok(Value::Text(" ".to_string())));
    }

    if code == "~ldshow" || code == "~ldshow()" {
        if let Some(ref scr) = vm.screen {
            scr.render();
        }
        return Some(Ok(Value::None));
    }

    if code == "~ldclear" || code == "~ldclear()" {
        if let Some(ref mut scr) = vm.screen {
            scr.clear();
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~ldm(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() >= 5 {
            let x1 = match vm.eval_value(&p[0]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as i32;
            let y1 = match vm.eval_value(&p[1]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as i32;
            let x2 = match vm.eval_value(&p[2]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as i32;
            let y2 = match vm.eval_value(&p[3]) { Ok(v) => v.as_number(), Err(_) => None }.unwrap_or(0.0) as i32;
            let ch = VM::tq(&p[4]).chars().next().unwrap_or('*');

            if let Some(ref mut scr) = vm.screen {
                // Шаги делаем только целыми без дробей по методу Брезенхэма чтобы линия оставалась ровной и быстрой даже на большом экране
                let mut x = x1;
                let mut y = y1;
                let dx = (x2 - x1).abs();
                let dy = (y2 - y1).abs();
                let sx = if x1 < x2 { 1 } else { -1 };
                let sy = if y1 < y2 { 1 } else { -1 };
                let mut err = dx - dy;

                loop {
                    if x >= 0 && y >= 0 {
                        scr.set(x as usize, y as usize, ch);
                    }
                    if x == x2 && y == y2 { break; }
                    let e2 = 2 * err;
                    if e2 > -dy { err -= dy; x += sx; }
                    if e2 < dx { err += dx; y += sy; }
                }
            }
            return Some(Ok(Value::None));
        }
    }

    // Фигуры прокрутку рамку и файлы держим отдельно от базы чтобы базу выше было легко читать

    if code.starts_with("~ldrect(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ',');
        if p.len() >= 5 {
            let x = vm.eval_value(&p[0]).ok()?.as_number()? as usize;
            let y = vm.eval_value(&p[1]).ok()?.as_number()? as usize;
            let w = vm.eval_value(&p[2]).ok()?.as_number()? as usize;
            let h = vm.eval_value(&p[3]).ok()?.as_number()? as usize;
            let ch = VM::tq(&p[4]).chars().next().unwrap_or('#');
            if let Some(ref mut scr) = vm.screen { scr.fill_rect(x, y, w, h, ch); }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~ldcircle(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 4 {
            let cx = vm.eval_value(&p[0]).ok()?.as_number()? as usize;
            let cy = vm.eval_value(&p[1]).ok()?.as_number()? as usize;
            let r = vm.eval_value(&p[2]).ok()?.as_number()? as usize;
            let ch = VM::tq(&p[3]).chars().next().unwrap_or('*');
            if let Some(ref mut scr) = vm.screen { scr.draw_circle(cx, cy, r, ch); }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~ldfill_circle(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ',');
        if p.len() >= 4 {
            let cx = vm.eval_value(&p[0]).ok()?.as_number()? as usize;
            let cy = vm.eval_value(&p[1]).ok()?.as_number()? as usize;
            let r = vm.eval_value(&p[2]).ok()?.as_number()? as usize;
            let ch = VM::tq(&p[3]).chars().next().unwrap_or('*');
            if let Some(ref mut scr) = vm.screen { scr.fill_circle(cx, cy, r, ch); }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~ldscroll(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 2 {
            let dx = vm.eval_value(&p[0]).ok()?.as_number()? as i32;
            let dy = vm.eval_value(&p[1]).ok()?.as_number()? as i32;
            if let Some(ref mut scr) = vm.screen { scr.scroll(dx, dy); }
            return Some(Ok(Value::None));
        }
    }

    if code.starts_with("~ldborder(") && code.ends_with(')') {
        let ch = VM::tq(&code[10..code.len()-1]).chars().next().unwrap_or('#');
        if let Some(ref mut scr) = vm.screen { scr.border = ch; }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~ldsave(") && code.ends_with(')') {
        let path = VM::tq(&code[8..code.len()-1]);
        if let Some(ref scr) = vm.screen { return Some(scr.save_to_file(&path).map(|_| Value::None).map_err(|e| e)); }
        return Some(Err("No screen".to_string()));
    }

    if code.starts_with("~ldload(") && code.ends_with(')') {
        let path = VM::tq(&code[8..code.len()-1]);
        if let Some(ref mut scr) = vm.screen { return Some(scr.load_from_file(&path).map(|_| Value::None).map_err(|e| e)); }
        return Some(Err("No screen".to_string()));
    }

    None
}
