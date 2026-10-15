use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! p1 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let n = vm.eval_expr(&code[$p.len()..code.len()-1]).ok()?; let r = ($f)(n); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } }; }
    macro_rules! p2 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_expr(&p[0]).ok()?; let b = vm.eval_expr(&p[1]).ok()?; let r = ($f)(a, b); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } } }; }
    macro_rules! p3 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 3 { let a = vm.eval_expr(&p[0]).ok()?; let b = vm.eval_expr(&p[1]).ok()?; let c = vm.eval_expr(&p[2]).ok()?; let r = ($f)(a, b, c); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } } }; }

    p1!("~clamp_01(", 10, |x: f64| x.clamp(0.0, 1.0));
    p1!("~ease_in_quad(", 14, |x: f64| x * x);
    p1!("~ease_out_quad(", 15, |x: f64| 1.0 - (1.0 - x) * (1.0 - x));
    p1!("~ease_in_out_quad(", 18, |x: f64| if x < 0.5 { 2.0 * x * x } else { 1.0 - (2.0 * (1.0 - x)).powi(2) / 2.0 });
    p1!("~ease_in_cubic(", 15, |x: f64| x * x * x);
    p1!("~ease_out_cubic(", 16, |x: f64| 1.0 - (1.0 - x).powi(3));
    p1!("~ease_in_out_cubic(", 19, |x: f64| if x < 0.5 { 4.0 * x * x * x } else { 1.0 - (2.0 * (1.0 - x)).powi(3) / 2.0 });
    p1!("~smoothstep01(", 14, |x: f64| { let t = x.clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) });
    p1!("~frame_decay(", 13, |x: f64| x * 0.98);
    p1!("~invert_vel(", 12, |x: f64| -x);
    p1!("~speed_abs(", 11, |x: f64| x.abs());
    p1!("~g_force_val(", 13, |x: f64| x * 9.80665);
    p1!("~damp_factor(", 13, |x: f64| (1.0 - x).max(0.0));
    p1!("~osc_sin(", 9, |x: f64| x.sin());
    p1!("~osc_cos(", 9, |x: f64| x.cos());
    p1!("~frac_noise(", 12, |x: f64| { (x.sin() * 43758.5453).fract().abs() });
    p1!("~pulse_wave(", 12, |x: f64| if x.sin() >= 0.0 { 1.0 } else { 0.0 });
    p1!("~pingpong(", 10, |x: f64| 1.0 - ((x % 2.0).abs() - 1.0).abs());
    p1!("~reflect_vx(", 12, |x: f64| -x);
    p1!("~reflect_vy(", 12, |x: f64| -x);

    p2!("~game_dist2(", 12, |a: f64, b: f64| (a * a + b * b).sqrt());
    p2!("~angle_from_vel2(", 17, |a: f64, b: f64| a.atan2(b).to_degrees());
    p2!("~rand_between(", 14, |a: f64, b: f64| { use rand::Rng; let lo = a.min(b); let hi = a.max(b); rand::thread_rng().gen_range(lo..hi) });
    p2!("~rand_int_between(", 18, |a: f64, b: f64| { use rand::Rng; let lo = a.min(b).round() as i64; let hi = a.max(b).round() as i64; rand::thread_rng().gen_range(lo..=hi) as f64 });
    p2!("~angle_delta(", 13, |a: f64, b: f64| { let d = (b - a) % 360.0; if d > 180.0 { d - 360.0 } else if d < -180.0 { d + 360.0 } else { d } });
    p2!("~angle_delta_abs(", 17, |a: f64, b: f64| { let d = (b - a).abs() % 360.0; if d > 180.0 { 360.0 - d } else { d } });
    p2!("~jump_velocity(", 15, |h: f64, g: f64| if g <= 0.0 { f64::NAN } else { (2.0 * h * g).sqrt() });
    p2!("~fall_time(", 11, |h: f64, g: f64| if g <= 0.0 { f64::NAN } else { (2.0 * h / g).sqrt() });
    p2!("~time_from_speed(", 17, |d: f64, v: f64| if v == 0.0 { f64::NAN } else { d / v });
    p2!("~speed_from_time(", 17, |d: f64, t: f64| if t == 0.0 { f64::NAN } else { d / t });
    p2!("~dist_from_speed(", 17, |v: f64, t: f64| v * t);
    p3!("~accel_from_speed(", 18, |v0: f64, v1: f64, t: f64| if t == 0.0 { f64::NAN } else { (v1 - v0) / t });
    p2!("~vel_from_accel(", 16, |a: f64, t: f64| a * t);
    p2!("~avg_velocity(", 14, |v0: f64, v1: f64| (v0 + v1) / 2.0);
    p2!("~stop_distance(", 15, |v: f64, dec: f64| if dec == 0.0 { f64::INFINITY } else { v * v / (2.0 * dec.abs()) });
    p2!("~reaction_distance(", 19, |v: f64, t: f64| v * t);
    p2!("~follow_ratio(", 14, |cur: f64, tgt: f64| if tgt == 0.0 { f64::NAN } else { cur / tgt });
    p2!("~health_ratio(", 14, |hp: f64, max: f64| if max == 0.0 { 0.0 } else { (hp / max).clamp(0.0, 1.0) });
    p2!("~screen_ratio(", 14, |x: f64, size: f64| if size == 0.0 { 0.0 } else { x / size });
    p2!("~aspect_ratio(", 14, |w: f64, h: f64| if h == 0.0 { f64::NAN } else { w / h });
    p2!("~pixel_area(", 12, |w: f64, h: f64| w * h);
    p2!("~bounce_off_floor(", 18, |vy: f64, rest: f64| (vy.abs() * rest).max(0.0));
    p2!("~bounce_off_wall(", 17, |vx: f64, rest: f64| (vx.abs() * rest).max(0.0));
    p2!("~fall_speed(", 12, |t: f64, g: f64| g * t);
    p2!("~reach_time(", 12, |v: f64, a: f64| if a == 0.0 { f64::INFINITY } else { v / a });
    p2!("~climb_height(", 14, |deg: f64, dist: f64| dist * deg.to_radians().sin());
    p2!("~run_distance(", 14, |deg: f64, dist: f64| dist * deg.to_radians().cos());
    p2!("~slope_descent(", 15, |h: f64, len: f64| if len == 0.0 { f64::NAN } else { (h / len).atan().to_degrees() });
    p2!("~normalize_speed(", 17, |v: f64, max: f64| if max == 0.0 { 0.0 } else { (v / max).clamp(0.0, 1.0) });
    p2!("~weight_move(", 13, |x: f64, t: f64| x * t);
    p2!("~round_trip_time(", 17, |d: f64, v: f64| if v == 0.0 { f64::INFINITY } else { 2.0 * d / v });
    p2!("~combo_multiplier(", 18, |n: f64, base: f64| base * n.min(100.0));
    p2!("~score_multiplier(", 18, |s: f64, m: f64| s * m);
    p2!("~damage_reduction(", 18, |armor: f64, max: f64| if max == 0.0 { 0.0 } else { (armor / (armor + max)).clamp(0.0, 1.0) });
    p2!("~critical_chance(", 17, |luck: f64, base: f64| (base + luck * 0.01).clamp(0.0, 1.0));
    p2!("~hit_chance(", 12, |acc: f64, eva: f64| { let d = acc - eva; if d >= 90.0 { 1.0 } else if d <= -90.0 { 0.0 } else { (d + 90.0) / 180.0 } });

    p3!("~clamp_value(", 13, |x: f64, lo: f64, hi: f64| x.clamp(lo, hi));
    p3!("~wrap_value(", 12, |x: f64, lo: f64, hi: f64| { let r = hi - lo; if r <= 0.0 { lo } else { ((x - lo) % r + r) % r + lo } });
    p3!("~move_towards(", 14, |cur: f64, tgt: f64, d: f64| { if (tgt - cur).abs() <= d.abs() { tgt } else { cur + d.signum() * d.abs() } });
    p3!("~lerp_angle(", 12, |a: f64, b: f64, t: f64| { let d = (b - a) % 360.0; let d = if d > 180.0 { d - 360.0 } else if d < -180.0 { d + 360.0 } else { d }; ((a + d * t) % 360.0 + 360.0) % 360.0 });
    p3!("~bounce_v(", 10, |v: f64, floor: f64, rest: f64| if v > floor { v * rest } else { floor });
    p3!("~accelerate(", 12, |v: f64, a: f64, dt: f64| v + a * dt);
    p3!("~drag_velocity(", 15, |v: f64, drag: f64, dt: f64| v * (1.0 - drag * dt).max(0.0));
    p3!("~position_update(", 17, |x: f64, v: f64, dt: f64| x + v * dt);
    p3!("~velocity_update(", 17, |v: f64, a: f64, dt: f64| v + a * dt);
    p3!("~angle_turn(", 12, |cur: f64, tgt: f64, step: f64| { let d = (tgt - cur) % 360.0; let d = if d > 180.0 { d - 360.0 } else if d < -180.0 { d + 360.0 } else { d }; if d.abs() <= step { tgt } else { cur + d.signum() * step } });
    p3!("~timer_countdown(", 17, |t: f64, dt: f64, _: f64| (t - dt).max(0.0));
    p3!("~repeat_cycle(", 14, |t: f64, interval: f64, _: f64| if interval <= 0.0 { 0.0 } else { t % interval });
    p3!("~ease_tween(", 12, |start: f64, end: f64, t: f64| start + (end - start) * t.clamp(0.0, 1.0));
    p3!("~smooth_tween(", 14, |start: f64, end: f64, t: f64| { let u = t.clamp(0.0, 1.0); let s = u * u * (3.0 - 2.0 * u); start + (end - start) * s });
    p3!("~gravity_step(", 14, |vy: f64, g: f64, dt: f64| vy + g * dt);
    p3!("~friction_step(", 15, |vx: f64, f: f64, dt: f64| { let dec = f * dt; if vx.abs() <= dec { 0.0 } else { vx - vx.signum() * dec } });
    p3!("~cooldown_tick(", 15, |t: f64, dt: f64, reset: f64| if t <= 0.0 { reset } else { (t - dt).max(0.0) });
    p3!("~health_after_damage(", 21, |hp: f64, dmg: f64, min: f64| (hp - dmg).max(min));
    p3!("~random_walk(", 13, |x: f64, range: f64, _: f64| { use rand::Rng; x + rand::thread_rng().gen_range(-range..range) });
    p3!("~angular_vel(", 13, |ang: f64, omega: f64, dt: f64| { ((ang + omega * dt) % 360.0 + 360.0) % 360.0 });
    p3!("~exponential_moving_avg(", 24, |prev: f64, cur: f64, alpha: f64| prev + (cur - prev) * alpha.clamp(0.0, 1.0));
    p3!("~target_lerp(", 13, |prev: f64, target: f64, alpha: f64| prev + (target - prev) * alpha.clamp(0.0, 1.0));
    p3!("~spin_update(", 13, |angle: f64, omega: f64, dt: f64| angle + omega * dt);
    p3!("~projectile_pos_x(", 18, |x: f64, vx: f64, dt: f64| x + vx * dt);
    p3!("~xp_curve(", 10, |level: f64, base: f64, growth: f64| base * level.powf(growth));
    p3!("~level_threshold(", 17, |level: f64, base: f64, growth: f64| base * level * growth);

    if code == "~fps" {
        let r = vm.game_fps.unwrap_or(60.0); vm.echo(format!("{}", Value::fmt_num(r))); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~fps(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[5..code.len()-1]).ok()?; vm.game_fps = Some(n); vm.echo(format!("{}", Value::fmt_num(n))); Some(Ok(Value::Number(n)))
    } else if code.starts_with("~collide_rect(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len()-1], ','); if p.len() >= 8 {
            let (x1,y1,w1,h1,x2,y2,w2,h2) = (vm.eval_value(&p[0]).ok()?.as_number()?, vm.eval_value(&p[1]).ok()?.as_number()?, vm.eval_value(&p[2]).ok()?.as_number()?, vm.eval_value(&p[3]).ok()?.as_number()?, vm.eval_value(&p[4]).ok()?.as_number()?, vm.eval_value(&p[5]).ok()?.as_number()?, vm.eval_value(&p[6]).ok()?.as_number()?, vm.eval_value(&p[7]).ok()?.as_number()?);
            let r = x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2;
            vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r))) } else { None }
    } else if code.starts_with("~collide_circ(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len()-1], ','); if p.len() >= 6 {
            let (x1,y1,r1,x2,y2,r2) = (vm.eval_value(&p[0]).ok()?.as_number()?, vm.eval_value(&p[1]).ok()?.as_number()?, vm.eval_value(&p[2]).ok()?.as_number()?, vm.eval_value(&p[3]).ok()?.as_number()?, vm.eval_value(&p[4]).ok()?.as_number()?, vm.eval_value(&p[5]).ok()?.as_number()?);
            let r = ((x1-x2).powi(2) + (y1-y2).powi(2)).sqrt() <= r1 + r2;
            vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r))) } else { None }
    } else if code.starts_with("~collide_point(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ','); if p.len() >= 4 {
            let (px,py,rx,ry,rw,rh) = if p.len() >= 6 { (vm.eval_value(&p[0]).ok()?.as_number()?, vm.eval_value(&p[1]).ok()?.as_number()?, vm.eval_value(&p[2]).ok()?.as_number()?, vm.eval_value(&p[3]).ok()?.as_number()?, vm.eval_value(&p[4]).ok()?.as_number()?, vm.eval_value(&p[5]).ok()?.as_number()?) } else { (0.0,0.0,0.0,0.0,0.0,0.0) };
            let r = px >= rx && px <= rx + rw && py >= ry && py <= ry + rh; vm.echo(format!("{}", r)); Some(Ok(Value::Bool(r))) } else { None }
    } else if code.starts_with("~gravity_x(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[11..code.len()-1]).ok()?; vm.gravity_x = n; vm.physics_on = true; vm.echo(format!("{}", Value::fmt_num(n))); Some(Ok(Value::Number(n)))
    } else if code.starts_with("~gravity_y(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[11..code.len()-1]).ok()?; vm.gravity_y = n; vm.physics_on = true; vm.echo(format!("{}", Value::fmt_num(n))); Some(Ok(Value::Number(n)))
    } else if code.starts_with("~entity_add(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 3 {
            let name = p[0].trim().to_string(); let x = vm.eval_expr(&p[1]).ok()?; let y = vm.eval_expr(&p[2]).ok()?;
            vm.entities.insert(name, crate::main::vm::Entity { x, y, ..crate::main::vm::Entity::new() });
            Some(Ok(Value::None)) } else { None }
    } else if code.starts_with("~entity_move(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ','); if p.len() >= 3 {
            let name = p[0].trim().to_string(); let dx = vm.eval_expr(&p[1]).ok()?; let dy = vm.eval_expr(&p[2]).ok()?;
            if let Some(e) = vm.entities.get_mut(&name) { e.x += dx; e.y += dy; } Some(Ok(Value::None)) } else { None }
    } else if code.starts_with("~physics_step(") && code.ends_with(')') {
        let dt = if code.len() > 14 { vm.eval_expr(&code[14..code.len()-1]).ok().unwrap_or(0.016) } else { 0.016 };
        vm.physics_step(dt); Some(Ok(Value::None))
    } else if code.starts_with("~camera(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ','); if p.len() >= 2 { vm.cam_x = vm.eval_expr(&p[0]).ok()?; vm.cam_y = vm.eval_expr(&p[1]).ok()?; } Some(Ok(Value::None))
    } else if code.starts_with("~shake(") && code.ends_with(')') {
        vm.shake_amount = vm.eval_expr(&code[7..code.len()-1]).ok()? as i32; Some(Ok(Value::None))
    } else if code.starts_with("~zoom(") && code.ends_with(')') {
        vm.zoom_level = vm.eval_expr(&code[6..code.len()-1]).ok()?; Some(Ok(Value::None))
    } else {
        None
    }
}
