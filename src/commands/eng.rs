use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! e1 {
        ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let n = match vm.eval_expr(&code[$p.len()..code.len()-1]) { Ok(v) => v, Err(e) => return Some(Err(e)) }; let r = ($f)(n); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } };
    }
    macro_rules! e2 {
        ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let a = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) }; let b = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) }; let r = ($f)(a, b); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } }; };
    }
    macro_rules! e3 {
        ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 3 { let a = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) }; let b = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) }; let c = match vm.eval_expr(&p[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) }; let r = ($f)(a, b, c); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } }; };
    }

    // Модуль стали 200 ГПа зашит внутрь чтобы прогиб балки считался только по двум числам без лишних вводов
    e2!("~beam_deflection(", 17, |f: f64, l: f64| f * l.powi(3) / (48.0 * 200e9));
    e2!("~beam_stress(", 13, |m: f64, z: f64| m / z);
    e2!("~moment_force(", 14, |f: f64, d: f64| f * d);
    e2!("~shear_stress(", 14, |v: f64, a: f64| v / a);
    e2!("~torsion_stress(", 16, |t: f64, r: f64| t * r / (std::f64::consts::PI * r.powi(4) / 2.0));
    e3!("~column_buckling(", 17, |e: f64, i: f64, l: f64| std::f64::consts::PI.powi(2) * e * i / l.powi(2));

    // Плотность воздуха и вязкость зашиты внутрь поэтому расход и силы считаем только от скорости и площади
    e2!("~reynolds(", 10, |v: f64, d: f64| v * d / 1.516e-5);
    e2!("~bernoulli(", 11, |p: f64, v: f64| p + 0.5 * 1.225 * v.powi(2));
    e2!("~drag_force(", 12, |v: f64, a: f64| 0.5 * 1.225 * v.powi(2) * a * 0.47);
    e2!("~lift_force(", 12, |v: f64, a: f64| 0.5 * 1.225 * v.powi(2) * a * 1.0);
    e2!("~flow_rate(", 11, |v: f64, a: f64| v * a);
    e3!("~pipe_loss(", 11, |f: f64, l: f64, d: f64| f * l / d);

    // Закон Ома и соединения резисторов считаем готовыми формулами чтобы не выводить их вручную в скрипте
    e2!("~power_law(", 11, |v: f64, i: f64| v * i);
    e2!("~resistor_parallel(", 19, |r1: f64, r2: f64| 1.0 / (1.0/r1 + 1.0/r2));
    e2!("~resistor_series(", 17, |r1: f64, r2: f64| r1 + r2);
    e2!("~capacitor_parallel(", 20, |c1: f64, c2: f64| c1 + c2);
    e2!("~capacitor_series(", 18, |c1: f64, c2: f64| 1.0 / (1.0/c1 + 1.0/c2));
    e2!("~inductor_series(", 17, |l1: f64, l2: f64| l1 + l2);
    e2!("~time_constant(", 15, |r: f64, c: f64| r * c);
    e2!("~resonant_freq(", 15, |l: f64, c: f64| 1.0 / (2.0 * std::f64::consts::PI * (l * c).sqrt()));
    e1!("~skin_depth(", 12, |f: f64| 1.0 / (std::f64::consts::PI * f * 4e-7 * 5.8e7).sqrt());

    // Теплоемкость воздуха и теплоту парообразования зашили внутрь поэтому хватает только массы и разницы температур
    e2!("~sensible_heat(", 15, |m: f64, dt: f64| m * 1005.0 * dt);
    e2!("~latent_heat(", 13, |m: f64, _: f64| m * 2.5e6);
    e2!("~enthalpy(", 10, |t: f64, h: f64| 1.006 * t + h * (2501.0 + 1.86 * t));

    // Напряжение деформацию и модуль считаем делением двух чисел поэтому порядок аргументов ниже важен для знака
    e2!("~stress(", 8, |f: f64, a: f64| f / a);
    e2!("~strain(", 8, |dl: f64, l: f64| dl / l);
    e2!("~modulus(", 9, |s: f64, st: f64| s / st);
    e2!("~poisson(", 9, |lat: f64, ax: f64| -lat / ax);

    // Уровень шума считаем отношением двух давлений поэтому ниже просим оба числа
    e2!("~decibel(", 9, |p1: f64, p2: f64| 10.0 * (p1 / p2).log10());
    // Расхождение считаем точной формулой с логарифмом а поглощение воздухом считаем упрощенно линейно чтобы прикидка оставалась быстрой
    // Поглощение растет с частотой и дистанцией и падает с влажностью поэтому это не строгий стандарт а быстрая оценка
    // Вблизи источника формула точна а вдали и на высоких частотах дает лишь порядок величины поэтому применяем ее для прикидок
    e3!("~sound_attenuation(", 19, |d: f64, f: f64, rh: f64| {
        let spread = 20.0 * d.max(1.0).log10();
        let air = d.max(0.0) * (f / 1000.0) * 0.005 * (1.2 - (rh / 100.0).clamp(0.0, 1.0));
        spread + air.max(0.0)
    });

    // Показатель стекла 1.5 зашит внутрь чтобы линзы и увеличение считались простыми отношениями по двум числам
    e2!("~lens_maker(", 12, |r1: f64, r2: f64| (1.5 - 1.0) * (1.0/r1 - 1.0/r2));
    e2!("~magnification(", 15, |hi: f64, ho: f64| hi / ho);
    e2!("~f_number(", 10, |f: f64, d: f64| f / d);
    e2!("~snell(", 7, |n1: f64, n2: f64| (n1 / n2).asin());

    // Ускорение 9.81 зашито внутрь чтобы энергия импульс работа и давление считались прямыми формулами без лишних вводов
    e2!("~rot_energy(", 12, |i: f64, w: f64| 0.5 * i * w.powi(2));
    e2!("~potential_energy(", 18, |m: f64, h: f64| m * 9.81 * h);
    e2!("~momentum(", 10, |m: f64, v: f64| m * v);
    e2!("~impulse(", 9, |f: f64, t: f64| f * t);
    e2!("~work_force(", 12, |f: f64, d: f64| f * d);
    e2!("~power_work(", 12, |w: f64, t: f64| w / t);
    e2!("~torque_force(", 14, |f: f64, r: f64| f * r);
    e2!("~acceleration_force(", 20, |f: f64, m: f64| f / m);
    e3!("~centripetal_force(", 19, |m: f64, v: f64, r: f64| m * v.powi(2) / r);
    e2!("~mass_density(", 14, |m: f64, v: f64| m / v);
    e2!("~pressure_force(", 16, |f: f64, a: f64| f / a);
    e2!("~hydrostatic_pressure(", 22, |depth: f64, density: f64| depth * density * 9.81);
    e2!("~buoyancy(", 10, |v: f64, density: f64| v * density * 9.81);
    e2!("~doppler_freq(", 14, |f0: f64, v: f64| f0 * (343.0 / (343.0 - v)));
    e3!("~thermal_expansion(", 19, |l0: f64, alpha: f64, dt: f64| l0 * alpha * dt);
    e3!("~heat_energy(", 13, |m: f64, c: f64, dt: f64| m * c * dt);
    e2!("~gear_ratio(", 12, |t1: f64, t2: f64| t1 / t2);
    e2!("~mechanical_advantage(", 22, |load: f64, effort: f64| load / effort);

    // Эти формулы повторяют базовые но при делении на ноль отдают NaN чтобы скрипт не падал а получал понятный признак ошибки
    e2!("~resistor_power(", 16, |i: f64, r: f64| i.powi(2) * r);
    e3!("~voltage_divider(", 17, |v: f64, r1: f64, r2: f64| v * r2 / (r1 + r2));
    e2!("~capacitor_energy(", 18, |v: f64, c: f64| 0.5 * c * v.powi(2));
    e2!("~inductor_energy(", 17, |i: f64, l: f64| 0.5 * l * i.powi(2));
    e1!("~rms_voltage(", 13, |v: f64| v / 2.0_f64.sqrt());
    e1!("~frequency_period(", 18, |f: f64| 1.0 / f);
    e1!("~wavelength(", 12, |f: f64| 3e8 / f);
    e2!("~electric_field(", 16, |v: f64, d: f64| v / d);

    e1!("~blackbody_peak2(", 17, |t: f64| if t <= 0.0 { f64::NAN } else { 2.897771955e-3 / t });
    e1!("~photon_energy2(", 16, |f: f64| 6.62607015e-34 * f);
    e1!("~photon_momentum(", 17, |l: f64| if l == 0.0 { f64::INFINITY } else { 6.62607015e-34 / l });
    e1!("~kepler_period(", 15, |a: f64| 2.0 * std::f64::consts::PI * (a.powi(3) / 1.32712440018e20).sqrt());
    e1!("~pH_to_H(", 9, |ph: f64| 10.0f64.powf(-ph));
    e1!("~H_to_pH(", 9, |h: f64| if h <= 0.0 { f64::NAN } else { -h.log10() });

    e2!("~work_force_d(", 14, |f: f64, d: f64| f * d);
    e2!("~power_rat2(", 12, |w: f64, t: f64| if t == 0.0 { f64::NAN } else { w / t });
    e2!("~potential_energy_grav(", 23, |m: f64, h: f64| m * 9.80665 * h);
    e2!("~momentum_mass(", 15, |m: f64, v: f64| m * v);
    e2!("~impulse_force(", 15, |f: f64, t: f64| f * t);
    e2!("~force_pressure(", 16, |p: f64, a: f64| p * a);
    e2!("~pressure_force2(", 17, |f: f64, a: f64| if a == 0.0 { f64::NAN } else { f / a });
    e2!("~torque_lever(", 14, |f: f64, r: f64| f * r);
    e2!("~centripetal_accel(", 19, |v: f64, r: f64| if r == 0.0 { f64::NAN } else { v * v / r });
    e2!("~angular_velocity_lin(", 22, |v: f64, r: f64| if r == 0.0 { f64::NAN } else { v / r });
    e2!("~linear_velocity_ang(", 21, |w: f64, r: f64| w * r);
    e2!("~power_angular(", 15, |tq: f64, w: f64| tq * w);
    e2!("~kinetic_rot(", 13, |i: f64, w: f64| 0.5 * i * w * w);
    e2!("~moment_inertia_ring(", 21, |m: f64, r: f64| m * r * r);
    e2!("~moment_inertia_disk(", 21, |m: f64, r: f64| 0.5 * m * r * r);
    e2!("~moment_inertia_rod(", 20, |m: f64, l: f64| m * l * l / 12.0);
    e2!("~spring_force(", 14, |k: f64, x: f64| k * x);
    e2!("~spring_potential(", 18, |k: f64, x: f64| 0.5 * k * x * x);
    e2!("~spring_frequency(", 18, |k: f64, m: f64| if m <= 0.0 { f64::NAN } else { (k / m).sqrt() / (2.0 * std::f64::consts::PI) });
    e2!("~pendulum_period(", 17, |l: f64, g: f64| if g <= 0.0 { f64::NAN } else { 2.0 * std::f64::consts::PI * (l / g).sqrt() });
    e2!("~pendulum_frequency(", 20, |l: f64, g: f64| if l <= 0.0 { f64::NAN } else { (g / l).sqrt() / (2.0 * std::f64::consts::PI) });
    e2!("~wave_speed(", 12, |f: f64, l: f64| f * l);
    e2!("~wave_frequency(", 16, |v: f64, l: f64| if l == 0.0 { f64::NAN } else { v / l });
    e2!("~electric_power(", 16, |v: f64, i: f64| v * i);
    e2!("~electric_voltage(", 18, |i: f64, r: f64| i * r);
    e2!("~electric_current(", 18, |v: f64, r: f64| if r == 0.0 { f64::NAN } else { v / r });
    e2!("~electric_charge2(", 18, |i: f64, t: f64| i * t);
    e2!("~electric_energy(", 17, |p: f64, t: f64| p * t);
    e2!("~ohm_resistance(", 16, |v: f64, i: f64| if i == 0.0 { f64::NAN } else { v / i });
    e2!("~parallel_resistance(", 21, |r1: f64, r2: f64| if r1 + r2 == 0.0 { 0.0 } else { r1 * r2 / (r1 + r2) });
    e2!("~series_resistance(", 19, |r1: f64, r2: f64| r1 + r2);
    e2!("~capacitance_charge(", 20, |c: f64, v: f64| c * v);
    e2!("~capacitor_energy2(", 19, |c: f64, v: f64| 0.5 * c * v * v);
    e2!("~inductor_energy2(", 18, |l: f64, i: f64| 0.5 * l * i * i);
    e2!("~rc_time_constant(", 18, |r: f64, c: f64| r * c);
    e2!("~power_dissipated(", 18, |i: f64, r: f64| i * i * r);
    e2!("~density2(", 10, |m: f64, v: f64| if v == 0.0 { f64::NAN } else { m / v });
    e2!("~weight_mass(", 13, |m: f64, g: f64| m * g);
    e2!("~flow_rate2(", 12, |v: f64, t: f64| if t == 0.0 { f64::NAN } else { v / t });
    e2!("~mach_number(", 13, |v: f64, c: f64| if c == 0.0 { f64::NAN } else { v / c });
    e2!("~orbital_speed(", 15, |m: f64, r: f64| if r <= 0.0 { f64::NAN } else { (6.67430e-11 * m / r).sqrt() });
    e2!("~escape_velocity(", 17, |m: f64, r: f64| if r <= 0.0 { f64::NAN } else { (2.0 * 6.67430e-11 * m / r).sqrt() });
    e2!("~g_field(", 9, |m: f64, r: f64| if r <= 0.0 { f64::NAN } else { 6.67430e-11 * m / (r * r) });
    e2!("~luminosity(", 12, |r: f64, t: f64| r * r * t.powi(4));
    e2!("~de_broglie_wavelength(", 23, |p: f64, m: f64| if m == 0.0 { f64::INFINITY } else { p / m });
    e2!("~relativistic_factor(", 21, |v: f64, c: f64| { let b = v / c; if b.abs() >= 1.0 { f64::INFINITY } else { 1.0 / (1.0 - b * b).sqrt() } });
    e2!("~engine_efficiency(", 19, |w: f64, q: f64| if q == 0.0 { f64::NAN } else { w / q });
    e2!("~carnot_efficiency(", 19, |th: f64, tc: f64| if th <= 0.0 { f64::NAN } else { 1.0 - tc / th });
    e2!("~entropy_change(", 16, |q: f64, t: f64| if t == 0.0 { f64::NAN } else { q / t });
    e2!("~latent_heat2(", 14, |m: f64, l: f64| m * l);
    e2!("~gas_work(", 10, |p: f64, dv: f64| p * dv);
    e2!("~molar_mass_moles(", 18, |m: f64, mm: f64| if mm == 0.0 { f64::NAN } else { m / mm });

    e3!("~velocity_final(", 16, |v0: f64, a: f64, t: f64| v0 + a * t);
    e3!("~displacement_accel(", 20, |v0: f64, t: f64, a: f64| v0 * t + 0.5 * a * t * t);
    e3!("~centripetal_f2(", 16, |m: f64, v: f64, r: f64| if r == 0.0 { f64::NAN } else { m * v * v / r });
    e3!("~thermal_heat2(", 15, |m: f64, c: f64, dt: f64| m * c * dt);
    e3!("~ideal_gas_pressure(", 20, |n: f64, t: f64, v: f64| if v <= 0.0 { f64::NAN } else { n * 8.314462618 * t / v });
    e3!("~buoyant_force(", 15, |rho: f64, v: f64, g: f64| rho * v * g);
    e3!("~projectile_range(", 18, |v: f64, th: f64, g: f64| if g == 0.0 { f64::NAN } else { v * v * (2.0 * th.to_radians()).sin() / g });
    e3!("~projectile_height(", 19, |v: f64, th: f64, g: f64| if g == 0.0 { f64::NAN } else { v * v * th.to_radians().sin().powi(2) / (2.0 * g) });
    e3!("~projectile_time(", 17, |v: f64, th: f64, g: f64| if g == 0.0 { f64::NAN } else { 2.0 * v * th.to_radians().sin() / g });
    e3!("~gravity_force(", 15, |m1: f64, m2: f64, r: f64| if r <= 0.0 { f64::NAN } else { 6.67430e-11 * m1 * m2 / (r * r) });
    e3!("~time_dilation(", 15, |t: f64, v: f64, c: f64| { let b = v / c; if b.abs() >= 1.0 { f64::INFINITY } else { t / (1.0 - b * b).sqrt() } });
    e3!("~length_contraction(", 20, |l: f64, v: f64, c: f64| { let b = v / c; if b.abs() >= 1.0 { 0.0 } else { l * (1.0 - b * b).sqrt() } });
    e3!("~specific_heat_cap(", 19, |q: f64, m: f64, dt: f64| if m * dt == 0.0 { f64::NAN } else { q / (m * dt) });
    e3!("~thermal_exp2(", 14, |l0: f64, a: f64, dt: f64| l0 * (1.0 + a * dt));
    e3!("~ideal_gas_vol2(", 16, |n: f64, t: f64, p: f64| if p <= 0.0 { f64::NAN } else { n * 8.314462618 * t / p });
    e3!("~voltage_divider2(", 18, |vin: f64, r1: f64, r2: f64| if r1 + r2 == 0.0 { f64::NAN } else { vin * r2 / (r1 + r2) });
    e3!("~pressure_fluid(", 16, |rho: f64, h: f64, g: f64| rho * h * g);

    None
}
