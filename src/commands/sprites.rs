use crate::main::vm::{VM, Value, AnimState, GameObject, MapObject, Particle};
use crate::main::screen::Screen;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

/// Модуль игровых систем: спрайты, анимации, карты, камера, коллизии, HUD, меню, частицы, физика
pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // СПРАЙТЫ

    if code.starts_with("~sprite(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let rows_str = VM::tq(&p[1]);
            let rows: Vec<String> = rows_str.split("\\n").map(|s| s.to_string()).collect();
            vm.sprites.insert(name, rows);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~sprite_load(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let path = VM::tq(&p[1]);
            match fs::read_to_string(&path) {
                Ok(content) => {
                    let rows: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                    vm.sprites.insert(name, rows);
                    return Some(Ok(Value::Bool(true)));
                }
                Err(e) => return Some(Err(e.to_string())),
            }
        }
    }

    if code.starts_with("~draw_sprite(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 3 {
            let name = VM::tq(&p[0]);
            let x = vm.eval_expr(&p[1]).unwrap_or(0.0) as usize;
            let y = vm.eval_expr(&p[2]).unwrap_or(0.0) as usize;
            if let Some(rows) = vm.sprites.get(&name).cloned() {
                if let Some(ref mut scr) = vm.screen {
                    for (dy, row) in rows.iter().enumerate() {
                        scr.draw_text(x, y + dy, row);
                    }
                } else {
                    for (dy, row) in rows.iter().enumerate() {
                        print!("\x1B[{};{}H{}", y + dy + 1, x + 1, row);
                    }
                    let _ = io::stdout().flush();
                }
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code.starts_with("~sprite_size(") && code.ends_with(')') {
        let name = VM::tq(&code[13..code.len() - 1]);
        if let Some(rows) = vm.sprites.get(&name) {
            let w = rows.iter().map(|r| r.len()).max().unwrap_or(0);
            let h = rows.len();
            let list = vec![Value::Number(w as f64), Value::Number(h as f64)];
            vm.echo(&format!("{}", Value::List(list.clone()).display()));
            return Some(Ok(Value::List(list)));
        }
        return Some(Ok(Value::List(vec![Value::Number(0.0), Value::Number(0.0)])));
    }

    if code.starts_with("~sprite_del(") && code.ends_with(')') {
        let name = VM::tq(&code[12..code.len() - 1]);
        vm.sprites.remove(&name);
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~sprite_copy(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let src = VM::tq(&p[0]);
            let dst = VM::tq(&p[1]);
            if let Some(rows) = vm.sprites.get(&src).cloned() {
                vm.sprites.insert(dst, rows);
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code.starts_with("~sprite_mirror(") && code.ends_with(')') {
        let name = VM::tq(&code[15..code.len() - 1]);
        if let Some(rows) = vm.sprites.get_mut(&name) {
            for row in rows.iter_mut() {
                *row = row.chars().rev().collect();
            }
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~sprite_rotate(") && code.ends_with(')') {
        let name = VM::tq(&code[15..code.len() - 1]);
        if let Some(rows) = vm.sprites.get(&name).cloned() {
            let h = rows.len();
            let w = rows.iter().map(|r| r.len()).max().unwrap_or(0);
            let mut new_rows = vec![String::new(); w];
            for x in 0..w {
                for y in (0..h).rev() {
                    new_rows[x].push(rows[y].chars().nth(x).unwrap_or(' '));
                }
            }
            vm.sprites.insert(name, new_rows);
            return Some(Ok(Value::Bool(true)));
        }
    }

    // АНИМАЦИИ

    if code.starts_with("~anim(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let frames_str = VM::tq(&p[1]);
            let frames: Vec<String> = frames_str.split(',').map(|s| s.trim().to_string()).collect();
            vm.animations.insert(name, AnimState {
                frames,
                fps: 10.0,
                current: 0,
                playing: false,
                looping: false,
                pingpong: false,
                pp_dir: 1,
                last_update: Instant::now(),
                done: false,
            });
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~anim_speed(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let fps = vm.eval_expr(&p[1]).unwrap_or(10.0);
            if let Some(a) = vm.animations.get_mut(&name) {
                a.fps = fps;
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code.starts_with("~play(") && code.ends_with(')') {
        let name = VM::tq(&code[6..code.len() - 1]);
        if let Some(a) = vm.animations.get_mut(&name) {
            a.playing = true;
            a.looping = false;
            a.current = 0;
            a.done = false;
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~loop_anim(") && code.ends_with(')') {
        let name = VM::tq(&code[11..code.len() - 1]);
        if let Some(a) = vm.animations.get_mut(&name) {
            a.playing = true;
            a.looping = true;
            a.current = 0;
            a.done = false;
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~stop_anim(") && code.ends_with(')') {
        let name = VM::tq(&code[11..code.len() - 1]);
        if let Some(a) = vm.animations.get_mut(&name) {
            a.playing = false;
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~anim_state(") && code.ends_with(')') {
        let name = VM::tq(&code[12..code.len() - 1]);
        vm.update_anim(&name);
        if let Some(a) = vm.animations.get(&name) {
            if !a.frames.is_empty() {
                let f = a.frames[a.current].clone();
                vm.echo(format!("{}", f));
                return Some(Ok(Value::Text(f)));
            }
        }
        return Some(Ok(Value::None));
    }

    if code.starts_with("~anim_done(") && code.ends_with(')') {
        let name = VM::tq(&code[11..code.len() - 1]);
        let ok = vm.animations.get(&name).map(|a| a.done).unwrap_or(true);
        vm.echo(&format!("{}", ok));
        return Some(Ok(Value::Bool(ok)));
    }

    if code.starts_with("~anim_pingpong(") && code.ends_with(')') {
        let name = VM::tq(&code[15..code.len() - 1]);
        if let Some(a) = vm.animations.get_mut(&name) {
            a.pingpong = true;
            a.playing = true;
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~anim_create(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let fps = vm.eval_expr(&p[1]).unwrap_or(10.0);
            vm.animations.insert(name, AnimState {
                frames: vec![],
                fps,
                current: 0,
                playing: false,
                looping: false,
                pingpong: false,
                pp_dir: 1,
                last_update: Instant::now(),
                done: false,
            });
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~anim_addframe(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let frame = VM::tq(&p[1]);
            if let Some(a) = vm.animations.get_mut(&name) {
                a.frames.push(frame);
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code.starts_with("~anim_pause(") && code.ends_with(')') {
        let name = VM::tq(&code[12..code.len() - 1]);
        if let Some(a) = vm.animations.get_mut(&name) {
            a.playing = false;
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~anim_resume(") && code.ends_with(')') {
        let name = VM::tq(&code[13..code.len() - 1]);
        if let Some(a) = vm.animations.get_mut(&name) {
            a.playing = true;
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~anim_loop(") && code.ends_with(')') {
        let name = VM::tq(&code[11..code.len() - 1]);
        if let Some(a) = vm.animations.get_mut(&name) {
            a.looping = true;
            a.playing = true;
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~anim_once(") && code.ends_with(')') {
        let name = VM::tq(&code[11..code.len() - 1]);
        if let Some(a) = vm.animations.get_mut(&name) {
            a.looping = false;
            a.playing = true;
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~anim_frame(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let n = vm.eval_expr(&p[1]).unwrap_or(0.0) as usize;
            if let Some(a) = vm.animations.get_mut(&name) {
                a.current = n.min(a.frames.len().saturating_sub(1));
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code.starts_with("~anim_frames(") && code.ends_with(')') {
        let name = VM::tq(&code[13..code.len() - 1]);
        let n = vm.animations.get(&name).map(|a| a.frames.len()).unwrap_or(0);
        vm.echo(format!("{}", n));
        return Some(Ok(Value::Number(n as f64)));
    }

    if code.starts_with("~anim_current(") && code.ends_with(')') {
        let name = VM::tq(&code[14..code.len() - 1]);
        let n = vm.animations.get(&name).map(|a| a.current).unwrap_or(0);
        vm.echo(format!("{}", n));
        return Some(Ok(Value::Number(n as f64)));
    }

    if code.starts_with("~anim_remove(") && code.ends_with(')') {
        let name = VM::tq(&code[13..code.len() - 1]);
        vm.animations.remove(&name);
        return Some(Ok(Value::Bool(true)));
    }

    // FPS / ДЕЛЬТА

    if code.starts_with("~fps(") && code.ends_with(')') {
        let target = vm.eval_expr(&code[5..code.len() - 1]).unwrap_or(30.0).max(1.0);
        let ft = Duration::from_secs_f64(1.0 / target);
        let elapsed = vm.last_frame.elapsed();
        if elapsed < ft {
            thread::sleep(ft - elapsed);
        }
        vm.last_frame = Instant::now();
        vm.frame_count += 1;
        // Физика
        vm.physics_step(1.0 / target);
        return Some(Ok(Value::None));
    }

    if code == "~fps_show()" || code == "~fps_show" {
        let fps = 1.0 / vm.last_frame.elapsed().as_secs_f64().max(0.001);
        vm.echo(&format!("FPS: {:.1}", fps));
        return Some(Ok(Value::Number(fps)));
    }

    if code == "~delta()" {
        let d = vm.last_frame.elapsed().as_secs_f64();
        vm.echo(format!("{}", d));
        return Some(Ok(Value::Number(d)));
    }

    if code == "~framecount()" {
        vm.echo(&format!("{}", vm.frame_count));
        return Some(Ok(Value::Number(vm.frame_count as f64)));
    }

    // КАРТЫ

    if code.starts_with("~loadmap(") && code.ends_with(')') {
        let path = VM::tq(&code[9..code.len() - 1]);
        match fs::read_to_string(&path) {
            Ok(content) => {
                let rows: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                let h = rows.len();
                let w = rows.iter().map(|r| r.len()).max().unwrap_or(0);
                let mut scr = Screen::new(w, h, '\0', '.');
                for (y, row) in rows.iter().enumerate() {
                    for (x, ch) in row.chars().enumerate() {
                        scr.set(x, y, ch);
                    }
                }
                vm.map = Some(scr);
                println!("Map loaded: {}x{}", w, h);
                return Some(Ok(Value::Bool(true)));
            }
            Err(e) => return Some(Err(e.to_string())),
        }
    }

    if code.starts_with("~map_new(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() >= 2 {
            let w = vm.eval_expr(&p[0]).unwrap_or(30.0) as usize;
            let h = vm.eval_expr(&p[1]).unwrap_or(15.0) as usize;
            let fill = if p.len() > 2 {
                VM::tq(&p[2]).chars().next().unwrap_or('.')
            } else {
                '.'
            };
            vm.map = Some(Screen::new(w, h, '\0', fill));
            println!("Map {}x{} created", w, h);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~tile(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 2 {
            let x = vm.eval_expr(&p[0]).unwrap_or(0.0) as usize;
            let y = vm.eval_expr(&p[1]).unwrap_or(0.0) as usize;
            if let Some(ref m) = vm.map {
                let ch = m.get(x, y);
                vm.echo(&format!("{}", ch));
                return Some(Ok(Value::Text(ch.to_string())));
            }
        }
    }

    if code.starts_with("~settile(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 3 {
            let x = vm.eval_expr(&p[0]).unwrap_or(0.0) as usize;
            let y = vm.eval_expr(&p[1]).unwrap_or(0.0) as usize;
            let ch = VM::tq(&p[2]).chars().next().unwrap_or('.');
            if let Some(ref mut m) = vm.map {
                m.set(x, y, ch);
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code == "~mapsize()" {
        if let Some(ref m) = vm.map {
            let list = vec![Value::Number(m.width as f64), Value::Number(m.height as f64)];
            vm.echo(&format!("{}", Value::List(list.clone()).display()));
            return Some(Ok(Value::List(list)));
        }
        return Some(Ok(Value::None));
    }

    if code == "~drawmap()" || code == "~showmap" {
        if let Some(ref m) = vm.map {
            m.show();
            return Some(Ok(Value::None));
        }
        return Some(Err("No map".to_string()));
    }

    if code.starts_with("~drawmap_rect(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len() - 1], ',');
        if p.len() == 4 {
            if let Some(ref m) = vm.map {
                let x = vm.eval_expr(&p[0]).unwrap_or(0.0) as usize;
                let y = vm.eval_expr(&p[1]).unwrap_or(0.0) as usize;
                let w = vm.eval_expr(&p[2]).unwrap_or(0.0) as usize;
                let h = vm.eval_expr(&p[3]).unwrap_or(0.0) as usize;
                for dy in 0..h {
                    let line: String = (0..w).map(|dx| m.get(x + dx, y + dy)).collect();
                    println!("{}", line);
                }
                return Some(Ok(Value::None));
            }
            return Some(Err("No map".to_string()));
        }
    }

    if code.starts_with("~map_save(") && code.ends_with(')') {
        let path = VM::tq(&code[10..code.len() - 1]);
        if let Some(ref m) = vm.map {
            match m.save_to_file(&path) {
                Ok(_) => return Some(Ok(Value::Bool(true))),
                Err(e) => return Some(Err(e)),
            }
        }
        return Some(Err("No map".to_string()));
    }

    if code.starts_with("~map_fill(") && code.ends_with(')') {
        let ch = VM::tq(&code[10..code.len() - 1]).chars().next().unwrap_or('.');
        if let Some(ref mut m) = vm.map {
            m.fill(ch);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~map_replace(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let old = VM::tq(&p[0]).chars().next().unwrap_or('.');
            let new = VM::tq(&p[1]).chars().next().unwrap_or('.');
            if let Some(ref mut m) = vm.map {
                for y in 0..m.height {
                    for x in 0..m.width {
                        if m.buffer[y][x] == old {
                            m.buffer[y][x] = new;
                        }
                    }
                }
                return Some(Ok(Value::Bool(true)));
            }
        }
    }

    if code.starts_with("~map_count(") && code.ends_with(')') {
        let ch = VM::tq(&code[11..code.len() - 1]).chars().next().unwrap_or('.');
        if let Some(ref m) = vm.map {
            let n = m.buffer.iter().flat_map(|r| r.iter()).filter(|&&c| c == ch).count();
            vm.echo(format!("{}", n));
            return Some(Ok(Value::Number(n as f64)));
        }
        return Some(Ok(Value::Number(0.0)));
    }

    if code.starts_with("~map ") {
        let parts: Vec<&str> = code[5..].trim().split_whitespace().collect();
        if parts.len() >= 2 {
            let w = parts[0].parse::<usize>().unwrap_or(30);
            let h = parts[1].parse::<usize>().unwrap_or(15);
            let fill = if parts.len() > 2 { parts[2].chars().next().unwrap_or('.') } else { '.' };
            vm.map = Some(Screen::new(w, h, '\0', fill));
            println!("Map {}x{} created", w, h);
            return Some(Ok(Value::Bool(true)));
        }
    }

    // Объекты на карте

    if code.starts_with("~obj ") {
        let parts: Vec<&str> = code[5..].trim().splitn(4, ' ').collect();
        if parts.len() >= 4 {
            let name = parts[0].to_string();
            let ch = parts[1].chars().next().unwrap_or('@');
            let x = parts[2].parse::<usize>().unwrap_or(0);
            let y = parts[3].parse::<usize>().unwrap_or(0);
            vm.objects.insert(name, MapObject { ch, x, y });
            if let Some(ref mut m) = vm.map {
                m.set(x, y, ch);
            }
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~objmove ") {
        let parts: Vec<&str> = code[9..].trim().splitn(3, ' ').collect();
        if parts.len() >= 3 {
            let name = parts[0].to_string();
            let nx = parts[1].parse::<usize>().unwrap_or(0);
            let ny = parts[2].parse::<usize>().unwrap_or(0);
            if let Some(obj) = vm.objects.get(&name) {
                let ch = obj.ch;
                let ox = obj.x;
                let oy = obj.y;
                if let Some(ref mut m) = vm.map {
                    m.set(ox, oy, m.empty);
                    m.set(nx, ny, ch);
                }
                if let Some(obj_mut) = vm.objects.get_mut(&name) {
                    obj_mut.x = nx;
                    obj_mut.y = ny;
                }
                return Some(Ok(Value::Bool(true)));
            }
            return Some(Err(format!("Object '{}' not found", name)));
        }
    }

    if code.starts_with("~objdel ") {
        let name = code[8..].trim().to_string();
        if let Some(obj) = vm.objects.remove(&name) {
            if let Some(ref mut m) = vm.map {
                m.set(obj.x, obj.y, m.empty);
            }
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~spawn ") {
        let parts: Vec<&str> = code[7..].trim().splitn(3, ' ').collect();
        if parts.len() >= 3 {
            let name = parts[0];
            let x = parts[1].parse::<usize>().unwrap_or(0);
            let y = parts[2].parse::<usize>().unwrap_or(0);
            let ch = vm.get_var(name).and_then(|v| v.as_text()).unwrap_or_else(|| "@".to_string()).chars().next().unwrap_or('@');
            vm.objects.insert(name.to_string(), MapObject { ch, x, y });
            if let Some(ref mut m) = vm.map {
                m.set(x, y, ch);
            }
            return Some(Ok(Value::Bool(true)));
        }
    }

    // КАМЕРА

    if code.starts_with("~camera(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            vm.cam_x = vm.eval_expr(&p[0]).unwrap_or(0.0);
            vm.cam_y = vm.eval_expr(&p[1]).unwrap_or(0.0);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~camera_follow(") && code.ends_with(')') {
        let name = code[15..code.len() - 1].trim();
        if let Some(obj) = vm.objects.get(name) {
            vm.cam_x = obj.x as f64;
            vm.cam_y = obj.y as f64;
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~viewport(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 2 {
            vm.viewport_w = vm.eval_expr(&p[0]).unwrap_or(80.0) as usize;
            vm.viewport_h = vm.eval_expr(&p[1]).unwrap_or(24.0) as usize;
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~world_to_screen(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len() - 1], ',');
        if p.len() == 2 {
            let wx = vm.eval_expr(&p[0]).unwrap_or(0.0);
            let wy = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let sx = wx - vm.cam_x + (vm.viewport_w as f64 / 2.0);
            let sy = wy - vm.cam_y + (vm.viewport_h as f64 / 2.0);
            let list = vec![Value::Number(sx), Value::Number(sy)];
            vm.echo(&format!("{}", Value::List(list.clone()).display()));
            return Some(Ok(Value::List(list)));
        }
    }

    if code.starts_with("~screen_to_world(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len() - 1], ',');
        if p.len() == 2 {
            let sx = vm.eval_expr(&p[0]).unwrap_or(0.0);
            let sy = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let wx = sx + vm.cam_x - (vm.viewport_w as f64 / 2.0);
            let wy = sy + vm.cam_y - (vm.viewport_h as f64 / 2.0);
            let list = vec![Value::Number(wx), Value::Number(wy)];
            vm.echo(&format!("{}", Value::List(list.clone()).display()));
            return Some(Ok(Value::List(list)));
        }
    }

    if code.starts_with("~shake(") && code.ends_with(')') {
        vm.shake_amount = vm.eval_expr(&code[7..code.len() - 1]).unwrap_or(0.0) as i32;
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~zoom(") && code.ends_with(')') {
        vm.zoom_level = vm.eval_expr(&code[6..code.len() - 1]).unwrap_or(1.0);
        return Some(Ok(Value::Bool(true)));
    }

    // ОБЪЕКТНАЯ СИСТЕМА (физика)

    if code.starts_with("~game_object(") && code.ends_with(')') {
        let name = VM::tq(&code[13..code.len() - 1]);
        vm.entities.insert(name, crate::main::vm::Entity::new());
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~game_sprite(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let sprite = VM::tq(&p[1]);
            if let Some(e) = vm.entities.get_mut(&name) {
                e.sprite = sprite;
                return Some(Ok(Value::Bool(true)));
            }
            return Some(Err(format!("Entity '{}' not found", name)));
        }
    }

    if code.starts_with("~game_pos(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 1 {
            let name = VM::tq(&p[0]);
            if let Some(e) = vm.entities.get(&name) {
                let list = vec![Value::Number(e.x), Value::Number(e.y)];
                vm.echo(&format!("{}", Value::List(list.clone()).display()));
                return Some(Ok(Value::List(list)));
            }
            return Some(Err(format!("Entity '{}' not found", name)));
        } else if p.len() == 3 {
            let name = VM::tq(&p[0]);
            let x = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let y = vm.eval_expr(&p[2]).unwrap_or(0.0);
            if let Some(e) = vm.entities.get_mut(&name) {
                e.x = x;
                e.y = y;
                return Some(Ok(Value::Bool(true)));
            }
            return Some(Err(format!("Entity '{}' not found", name)));
        }
    }

    if code.starts_with("~game_move(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 3 {
            let name = VM::tq(&p[0]);
            let dx = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let dy = vm.eval_expr(&p[2]).unwrap_or(0.0);
            if let Some(e) = vm.entities.get_mut(&name) {
                e.x += dx;
                e.y += dy;
                return Some(Ok(Value::Bool(true)));
            }
            return Some(Err(format!("Entity '{}' not found", name)));
        }
    }

    if code.starts_with("~game_rotate(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len() - 1], ',');
        if p.len() == 2 {
            let name = VM::tq(&p[0]);
            let deg = vm.eval_expr(&p[1]).unwrap_or(0.0);
            if let Some(e) = vm.entities.get_mut(&name) {
                e.rot = (e.rot + deg) % 360.0;
                let rot = e.rot;
                vm.echo(format!("{}", rot));
                return Some(Ok(Value::Number(rot)));
            }
            return Some(Err(format!("Entity '{}' not found", name)));
        }
    }

    if code.starts_with("~game_scale(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 3 {
            let name = VM::tq(&p[0]);
            let sx = vm.eval_expr(&p[1]).unwrap_or(1.0);
            let sy = vm.eval_expr(&p[2]).unwrap_or(1.0);
            if let Some(e) = vm.entities.get_mut(&name) {
                e.sx = sx;
                e.sy = sy;
                return Some(Ok(Value::Bool(true)));
            }
            return Some(Err(format!("Entity '{}' not found", name)));
        }
    }

    if code.starts_with("~game_velocity(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len() - 1], ',');
        if p.len() == 3 {
            let name = VM::tq(&p[0]);
            let vx = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let vy = vm.eval_expr(&p[2]).unwrap_or(0.0);
            if let Some(e) = vm.entities.get_mut(&name) {
                e.vx = vx;
                e.vy = vy;
                return Some(Ok(Value::Bool(true)));
            }
            return Some(Err(format!("Entity '{}' not found", name)));
        }
    }

    if code.starts_with("~game_acceleration(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[20..code.len() - 1], ',');
        if p.len() == 3 {
            let name = VM::tq(&p[0]);
            let ax = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let ay = vm.eval_expr(&p[2]).unwrap_or(0.0);
            if let Some(e) = vm.entities.get_mut(&name) {
                e.ax = ax;
                e.ay = ay;
                return Some(Ok(Value::Bool(true)));
            }
            return Some(Err(format!("Entity '{}' not found", name)));
        }
    }

    if code.starts_with("~game_gravity(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len() - 1], ',');
        if p.len() == 2 {
            vm.gravity_x = vm.eval_expr(&p[0]).unwrap_or(0.0);
            vm.gravity_y = vm.eval_expr(&p[1]).unwrap_or(0.0);
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~game_physics(") && code.ends_with(')') {
        let arg = code[14..code.len() - 1].trim();
        vm.physics_on = arg == "true" || arg == "on" || arg == "1";
        return Some(Ok(Value::Bool(vm.physics_on)));
    }

    if code.starts_with("~game_collision(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[16..code.len() - 1], ',');
        if p.len() == 2 {
            let a = VM::tq(&p[0]);
            let b = VM::tq(&p[1]);
            let ea = match vm.entities.get(&a) {
                Some(e) => e,
                None => return Some(Err(format!("Entity '{}' not found", a))),
            };
            let eb = match vm.entities.get(&b) {
                Some(e) => e,
                None => return Some(Err(format!("Entity '{}' not found", b))),
            };
            let ok = ea.x < eb.x + eb.sx && ea.x + ea.sx > eb.x
                && ea.y < eb.y + eb.sy && ea.y + ea.sy > eb.y;
            vm.echo(&format!("{}", ok));
            return Some(Ok(Value::Bool(ok)));
        }
    }

    if code == "~game_input()" {
        let key = crate::main::platform::win_key::key().map(|c| c.to_string()).unwrap_or_default();
        let d = Value::Dict(std::collections::HashMap::from([
            ("key".to_string(), Value::Text(key)),
            ("mouse_x".to_string(), Value::Number(0.0)),
            ("mouse_y".to_string(), Value::Number(0.0)),
        ]));
        vm.echo(format!("{}", d.display()));
        return Some(Ok(d));
    }

    // КОЛЛИЗИИ (геометрические)

    if code.starts_with("~collide(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 5 {
            let x1 = vm.eval_expr(&p[0]).unwrap_or(0.0);
            let y1 = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let x2 = vm.eval_expr(&p[2]).unwrap_or(0.0);
            let y2 = vm.eval_expr(&p[3]).unwrap_or(0.0);
            let d = vm.eval_expr(&p[4]).unwrap_or(0.0);
            let ok = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt() <= d;
            vm.echo(&format!("{}", ok));
            return Some(Ok(Value::Bool(ok)));
        }
    }

    if code.starts_with("~collide_rect(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len() - 1], ',');
        if p.len() == 8 {
            let x1 = vm.eval_expr(&p[0]).unwrap_or(0.0);
            let y1 = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let w1 = vm.eval_expr(&p[2]).unwrap_or(0.0);
            let h1 = vm.eval_expr(&p[3]).unwrap_or(0.0);
            let x2 = vm.eval_expr(&p[4]).unwrap_or(0.0);
            let y2 = vm.eval_expr(&p[5]).unwrap_or(0.0);
            let w2 = vm.eval_expr(&p[6]).unwrap_or(0.0);
            let h2 = vm.eval_expr(&p[7]).unwrap_or(0.0);
            let ok = x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2;
            vm.echo(&format!("{}", ok));
            return Some(Ok(Value::Bool(ok)));
        }
    }

    if code.starts_with("~collide_circ(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len() - 1], ',');
        if p.len() == 6 {
            let x1 = vm.eval_expr(&p[0]).unwrap_or(0.0);
            let y1 = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let r1 = vm.eval_expr(&p[2]).unwrap_or(0.0);
            let x2 = vm.eval_expr(&p[3]).unwrap_or(0.0);
            let y2 = vm.eval_expr(&p[4]).unwrap_or(0.0);
            let r2 = vm.eval_expr(&p[5]).unwrap_or(0.0);
            let ok = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt() <= r1 + r2;
            vm.echo(&format!("{}", ok));
            return Some(Ok(Value::Bool(ok)));
        }
    }

    if code.starts_with("~collide_line(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len() - 1], ',');
        if p.len() == 8 {
            let x1 = vm.eval_expr(&p[0]).unwrap_or(0.0);
            let y1 = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let x2 = vm.eval_expr(&p[2]).unwrap_or(0.0);
            let y2 = vm.eval_expr(&p[3]).unwrap_or(0.0);
            let x3 = vm.eval_expr(&p[4]).unwrap_or(0.0);
            let y3 = vm.eval_expr(&p[5]).unwrap_or(0.0);
            let x4 = vm.eval_expr(&p[6]).unwrap_or(0.0);
            let y4 = vm.eval_expr(&p[7]).unwrap_or(0.0);
            let d1 = (x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3);
            let d2 = (x4 - x3) * (y2 - y3) - (y4 - y3) * (x2 - x3);
            let d3 = (x2 - x1) * (y3 - y1) - (y2 - y1) * (x3 - x1);
            let d4 = (x2 - x1) * (y4 - y1) - (y2 - y1) * (x4 - x1);
            let ok = ((d1 < 0.0 && d2 > 0.0) || (d1 > 0.0 && d2 < 0.0))
                && ((d3 < 0.0 && d4 > 0.0) || (d3 > 0.0 && d4 < 0.0));
            vm.echo(&format!("{}", ok));
            return Some(Ok(Value::Bool(ok)));
        }
    }

    if code.starts_with("~collide_point_rect(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[20..code.len() - 1], ',');
        if p.len() == 6 {
            let px = vm.eval_expr(&p[0]).unwrap_or(0.0);
            let py = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let x = vm.eval_expr(&p[2]).unwrap_or(0.0);
            let y = vm.eval_expr(&p[3]).unwrap_or(0.0);
            let w = vm.eval_expr(&p[4]).unwrap_or(0.0);
            let h = vm.eval_expr(&p[5]).unwrap_or(0.0);
            let ok = px >= x && px <= x + w && py >= y && py <= y + h;
            vm.echo(&format!("{}", ok));
            return Some(Ok(Value::Bool(ok)));
        }
    }

    if code.starts_with("~collide_point_circ(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[20..code.len() - 1], ',');
        if p.len() == 5 {
            let px = vm.eval_expr(&p[0]).unwrap_or(0.0);
            let py = vm.eval_expr(&p[1]).unwrap_or(0.0);
            let cx = vm.eval_expr(&p[2]).unwrap_or(0.0);
            let cy = vm.eval_expr(&p[3]).unwrap_or(0.0);
            let r = vm.eval_expr(&p[4]).unwrap_or(0.0);
            let ok = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt() <= r;
            vm.echo(&format!("{}", ok));
            return Some(Ok(Value::Bool(ok)));
        }
    }

    if code.starts_with("~collide_grid(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len() - 1], ',');
        if p.len() == 3 {
            let x = vm.eval_expr(&p[0]).unwrap_or(0.0) as usize;
            let y = vm.eval_expr(&p[1]).unwrap_or(0.0) as usize;
            let wall = VM::tq(&p[2]).chars().next().unwrap_or('#');
            if let Some(ref m) = vm.map {
                let ch = m.get(x, y);
                let ok = ch == wall;
                vm.echo(&format!("{}", ok));
                return Some(Ok(Value::Bool(ok)));
            }
        }
    }

    // HUD / СЧЁТ / ЖИЗНИ / HP / XP

    if code.starts_with("~hud(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() >= 3 {
            let x = vm.eval_expr(&p[0]).unwrap_or(0.0) as usize;
            let y = vm.eval_expr(&p[1]).unwrap_or(0.0) as usize;
            let text = match vm.eval_value(&p[2]) {
                Ok(v) => vm.interp(&v.as_text().unwrap_or_default()),
                Err(e) => return Some(Err(e)),
            };
            print!("\x1B[{};{}H{}", y + 1, x + 1, text);
            let _ = io::stdout().flush();
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~score(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[7..code.len() - 1]).unwrap_or(0.0) as i64;
        vm.score = n;
        vm.set_var("score", Value::Number(n as f64));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~score_add(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[11..code.len() - 1]).unwrap_or(0.0) as i64;
        vm.score += n;
        vm.set_var("score", Value::Number(vm.score as f64));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~lives(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[7..code.len() - 1]).unwrap_or(0.0) as i64;
        vm.lives = n;
        vm.set_var("lives", Value::Number(n as f64));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~lives_add(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[11..code.len() - 1]).unwrap_or(0.0) as i64;
        vm.lives += n;
        vm.set_var("lives", Value::Number(vm.lives as f64));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~hp(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[4..code.len() - 1]).unwrap_or(0.0) as i64;
        vm.hp = n;
        vm.set_var("hp", Value::Number(n as f64));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~hp_add(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[8..code.len() - 1]).unwrap_or(0.0) as i64;
        vm.hp += n;
        vm.set_var("hp", Value::Number(vm.hp as f64));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~xp(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[4..code.len() - 1]).unwrap_or(0.0) as i64;
        vm.xp = n;
        vm.set_var("xp", Value::Number(n as f64));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~xp_add(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[8..code.len() - 1]).unwrap_or(0.0) as i64;
        vm.xp += n;
        vm.set_var("xp", Value::Number(vm.xp as f64));
        return Some(Ok(Value::Bool(true)));
    }

    // МЕНЮ / СОСТОЯНИЯ

    if code.starts_with("~menu(") && code.ends_with(')') {
        let items_str = VM::tq(&code[6..code.len() - 1]);
        vm.menu_items = items_str.split(',').map(|s| s.trim().to_string()).collect();
        vm.menu_selected = 0;
        for (i, item) in vm.menu_items.iter().enumerate() {
            println!("  {}{}", if i == vm.menu_selected { "▶ " } else { "  " }, item);
        }
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~menu_select()" || code == "~menu_select" {
        if !vm.menu_items.is_empty() {
            let idx = vm.menu_selected.min(vm.menu_items.len() - 1);
            let item = vm.menu_items[idx].clone();
            vm.echo(&format!("{}", item));
            return Some(Ok(Value::Text(item)));
        }
        return Some(Ok(Value::None));
    }

    if code == "~menu_index()" {
        vm.echo(&format!("{}", vm.menu_selected));
        return Some(Ok(Value::Number(vm.menu_selected as f64)));
    }

    if code == "~pause()" || code == "~pause" {
        vm.game_paused = true;
        println!("Game paused");
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~resume()" {
        vm.game_paused = false;
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~gamestate(") && code.ends_with(')') {
        let s = VM::tq(&code[11..code.len() - 1]);
        vm.game_state = s.clone();
        vm.set_var("gamestate", Value::Text(s));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~gamestate_is(") && code.ends_with(')') {
        let s = VM::tq(&code[14..code.len() - 1]);
        let ok = vm.game_state == s;
        vm.echo(&format!("{}", ok));
        return Some(Ok(Value::Bool(ok)));
    }

    if code.starts_with("~transition(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if !p.is_empty() {
            let duration = if p.len() > 1 {
                vm.eval_expr(&p[1]).unwrap_or(0.5)
            } else {
                0.5
            };
            println!("Transition: {} ({:.1}s)", VM::tq(&p[0]), duration);
            thread::sleep(Duration::from_secs_f64(duration));
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~dialog(") && code.ends_with(')') {
        let text = vm.interp(&VM::tq(&code[8..code.len() - 1]));
        let w = text.len() + 4;
        println!("\n  ┌{}┐", "─".repeat(w));
        println!("  │ {} │", text);
        println!("  └{}┘", "─".repeat(w));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~savegame(") && code.ends_with(')') {
        let slot = VM::tq(&code[10..code.len() - 1]);
        match crate::main::dtam::save_state(vm, &format!("save_{}.dtam", slot)) {
            Ok(_) => {
                println!("Saved slot '{}'", slot);
                return Some(Ok(Value::Bool(true)));
            }
            Err(e) => return Some(Err(e)),
        }
    }

    if code.starts_with("~loadgame(") && code.ends_with(')') {
        let slot = VM::tq(&code[10..code.len() - 1]);
        match crate::main::dtam::load_state(vm, &format!("save_{}.dtam", slot)) {
            Ok(_) => {
                println!("Loaded slot '{}'", slot);
                return Some(Ok(Value::Bool(true)));
            }
            Err(e) => return Some(Err(e)),
        }
    }

    if code.starts_with("~deletegame(") && code.ends_with(')') {
        let slot = VM::tq(&code[12..code.len() - 1]);
        let _ = fs::remove_file(format!("save_{}.dtam", slot));
        println!("Deleted slot '{}'", slot);
        return Some(Ok(Value::Bool(true)));
    }

    // ЭФФЕКТЫ / ЧАСТИЦЫ

    if code.starts_with("~particle(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() >= 3 {
            let x = vm.eval_expr(&p[0]).unwrap_or(0.0) as f32;
            let y = vm.eval_expr(&p[1]).unwrap_or(0.0) as f32;
            let ch = VM::tq(&p[2]).chars().next().unwrap_or('*');
            let life = if p.len() > 3 { vm.eval_expr(&p[3]).unwrap_or(10.0) as i32 } else { 10 };
            use rand::Rng;
            vm.particles.push(Particle {
                x,
                y,
                vx: rand::thread_rng().gen_range(-1.0..1.0),
                vy: rand::thread_rng().gen_range(-1.0..0.0),
                ch,
                life,
            });
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~particle_burst(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[16..code.len() - 1], ',');
        if p.len() >= 2 {
            let x = vm.eval_expr(&p[0]).unwrap_or(0.0) as f32;
            let y = vm.eval_expr(&p[1]).unwrap_or(0.0) as f32;
            let n = if p.len() > 2 { vm.eval_expr(&p[2]).unwrap_or(5.0) as usize } else { 5 };
            let chs = ['*', '.', '+', 'o', 'x', '#'];
            use rand::Rng;
            for _ in 0..n {
                vm.particles.push(Particle {
                    x,
                    y,
                    vx: rand::thread_rng().gen_range(-2.0..2.0),
                    vy: rand::thread_rng().gen_range(-2.0..0.5),
                    ch: chs[rand::thread_rng().gen_range(0..chs.len())],
                    life: rand::thread_rng().gen_range(5..15),
                });
            }
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~flash(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() >= 2 {
            let color = VM::tq(&p[0]);
            let duration = vm.eval_expr(&p[1]).unwrap_or(0.5);
            print!("{}", crate::main::vm::VM::ansi_bg(&color));
            let _ = io::stdout().flush();
            thread::sleep(Duration::from_secs_f64(duration));
            print!("\x1B[0m");
            let _ = io::stdout().flush();
            return Some(Ok(Value::Bool(true)));
        }
    }

    if code.starts_with("~fade_in(") && code.ends_with(')') {
        let duration = vm.eval_expr(&code[9..code.len() - 1]).unwrap_or(0.5);
        thread::sleep(Duration::from_secs_f64(duration));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~fade_out(") && code.ends_with(')') {
        let duration = vm.eval_expr(&code[10..code.len() - 1]).unwrap_or(0.5);
        thread::sleep(Duration::from_secs_f64(duration));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~screen_color(") && code.ends_with(')') {
        let color = VM::tq(&code[14..code.len() - 1]);
        println!("{}", crate::main::vm::VM::ansi_bg(&color));
        return Some(Ok(Value::Bool(true)));
    }

    // ИГРОВОЙ ДВИЖОК (алиасы game_)

    if code.starts_with("~game_init(") && code.ends_with(')') {
        vm.game_mode = true;
        println!("Game ready (console mode)");
        return Some(Ok(Value::Bool(true)));
    }

    if code == "~game_quit()" {
        vm.game_mode = false;
        vm.game_objects.clear();
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~game_fps(") && code.ends_with(')') {
        return resolve(vm, &format!("~fps{}", &code[10..]));
    }

    if code == "~game_delta()" {
        return resolve(vm, "~delta()");
    }

    if code.starts_with("~game_scene(") && code.ends_with(')') {
        return resolve(vm, &format!("~gamestate{}", &code[12..]));
    }

    if code.starts_with("~game_switch(") && code.ends_with(')') {
        return resolve(vm, &format!("~gamestate{}", &code[13..]));
    }

    if code.starts_with("~game_camera(") && code.ends_with(')') {
        return resolve(vm, &format!("~camera{}", &code[13..]));
    }

    if code.starts_with("~game_camerazoom(") && code.ends_with(')') {
        return resolve(vm, &format!("~zoom{}", &code[17..]));
    }

    if code.starts_with("~game_camerashake(") && code.ends_with(')') {
        return resolve(vm, &format!("~shake{}", &code[18..]));
    }

    if code.starts_with("~game_score(") && code.ends_with(')') {
        return resolve(vm, &format!("~score{}", &code[12..]));
    }

    if code.starts_with("~game_lives(") && code.ends_with(')') {
        return resolve(vm, &format!("~lives{}", &code[12..]));
    }

    if code.starts_with("~game_level(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[12..code.len() - 1]).unwrap_or(1.0);
        vm.set_var("level", Value::Number(n));
        return Some(Ok(Value::Bool(true)));
    }

    if code.starts_with("~game_save(") && code.ends_with(')') {
        return resolve(vm, &format!("~savegame{}", &code[11..]));
    }

    if code.starts_with("~game_load(") && code.ends_with(')') {
        return resolve(vm, &format!("~loadgame{}", &code[11..]));
    }

    if code == "~game_reset()" {
        return resolve(vm, "~reload");
    }

    if code == "~game_run()" {
        println!("(используйте loop {{ ... ~fps(30) ... }} для главного цикла)");
        return Some(Ok(Value::None));
    }

    None
}