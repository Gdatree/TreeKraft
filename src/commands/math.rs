use crate::main::vm::{VM, Value};
use rand::Rng;
use rand::seq::SliceRandom;

/// Математика живет тут чтобы скрипты считали без своих формул и циклов
pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! m1 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let n = vm.eval_expr(&code[$p.len()..code.len()-1]).ok()?; let r = ($f)(n); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } }; }
    macro_rules! m2 { ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let a = vm.eval_expr(&p[0]).ok()?; let b = vm.eval_expr(&p[1]).ok()?; let r = ($f)(a, b); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } } }; }

    m1!("~gamma_func(", 12, |x: f64| gamma_l(x));
    m1!("~lgamma_ln(", 11, |x: f64| gamma_l(x.abs()).ln());
    m1!("~digamma(", 9, |x: f64| { let d = 1e-6; (gamma_l(x + d) - gamma_l(x - d)) / (2.0 * d) });
    m1!("~erf_x(", 7, |x: f64| erf_approx(x));
    m1!("~erfc_x(", 8, |x: f64| 1.0 - erf_approx(x));
    m1!("~erf_inv(", 9, |x: f64| erf_inv_approx(x));
    m1!("~zeta_val(", 10, |x: f64| { if x <= 1.0 { f64::NAN } else { (1..=200000).map(|k| (k as f64).powf(-x)).sum() } });
    m1!("~lambert_w(", 11, |x: f64| { if x < -0.367879 { f64::NAN } else { let mut w = if x < 1.0 { x } else { x.ln() }; for _ in 0..40 { let ew = (-w).exp(); let f = w * ew - x; let d = (w + 1.0) * ew; if d.abs() < 1e-14 { break } w -= f / d; } w } });
    m1!("~sigmoid_math(", 14, |x: f64| 1.0 / (1.0 + (-x).exp()));
    m1!("~sigmoid_prime(", 15, |x: f64| { let s = 1.0 / (1.0 + (-x).exp()); s * (1.0 - s) });
    m1!("~softplus_math(", 15, |x: f64| (1.0 + x.exp()).ln());
    m1!("~logit_math(", 12, |x: f64| { if x <= 0.0 || x >= 1.0 { f64::NAN } else { (x / (1.0 - x)).ln() } });
    m1!("~softsign(", 10, |x: f64| x / (1.0 + x.abs()));
    m1!("~relu_math(", 11, |x: f64| x.max(0.0));
    m1!("~leaky_relu(", 12, |x: f64| if x < 0.0 { 0.01 * x } else { x });
    m1!("~elu_math(", 10, |x: f64| if x < 0.0 { x.exp() - 1.0 } else { x });
    m1!("~gelu_math(", 11, |x: f64| 0.5 * x * (1.0 + erf_approx(x / std::f64::consts::SQRT_2)));
    m1!("~swish_math(", 12, |x: f64| x / (1.0 + (-x).exp()));
    m1!("~sinc(", 6, |x: f64| if x.abs() < 1e-12 { 1.0 } else { x.sin() / x });
    m1!("~j0_bessel(", 11, |x: f64| bessel_j0(x));
    m1!("~j1_bessel(", 11, |x: f64| bessel_j1(x));
    m1!("~y0_bessel(", 11, |x: f64| bessel_y0(x));
    m1!("~y1_bessel(", 11, |x: f64| bessel_y1(x));
    m1!("~sech(", 6, |x: f64| 1.0 / x.cosh());
    m1!("~csch(", 6, |x: f64| if x.abs() < 1e-12 { f64::NAN } else { 1.0 / x.sinh() });
    m1!("~coth(", 6, |x: f64| if x.abs() < 1e-12 { f64::NAN } else { 1.0 / x.tanh() });
    m1!("~csc_deg(", 9, |x: f64| { let r = x.to_radians().sin(); if r.abs() < 1e-12 { f64::NAN } else { 1.0 / r } });
    m1!("~sec_deg(", 9, |x: f64| { let r = x.to_radians().cos(); if r.abs() < 1e-12 { f64::NAN } else { 1.0 / r } });
    m1!("~cot_deg(", 9, |x: f64| { let r = x.to_radians().tan(); if r.abs() < 1e-12 { f64::NAN } else { 1.0 / r } });
    m1!("~asec(", 6, |x: f64| if x.abs() < 1.0 { f64::NAN } else { (1.0 / x).acos() });
    m1!("~acsc(", 6, |x: f64| if x.abs() < 1.0 { f64::NAN } else { (1.0 / x).asin() });
    m1!("~acot(", 6, |x: f64| (1.0 / x).atan());
    m1!("~acoth(", 7, |x: f64| if x.abs() <= 1.0 { f64::NAN } else { 0.5 * ((x + 1.0) / (x - 1.0)).ln() });
    m1!("~normalize_angle(", 17, |x: f64| { ((x % 360.0) + 360.0) % 360.0 });
    m1!("~wrap_rad(", 10, |x: f64| { ((x % (2.0 * std::f64::consts::PI)) + 2.0 * std::f64::consts::PI) % (2.0 * std::f64::consts::PI) });
    m1!("~next_float(", 12, |x: f64| x + x.abs().max(1.0) * 1e-15);
    m1!("~prev_float(", 12, |x: f64| x - x.abs().max(1.0) * 1e-15);
    m1!("~log1p_math(", 12, |x: f64| x.ln_1p());
    m1!("~expm1_math(", 12, |x: f64| x.exp_m1());
    m1!("~db_to_amp(", 11, |x: f64| 10.0f64.powf(x / 20.0));
    m1!("~amp_to_db(", 11, |x: f64| if x <= 0.0 { f64::NEG_INFINITY } else { 20.0 * x.log10() });
    m1!("~db_to_power(", 13, |x: f64| 10.0f64.powf(x / 10.0));
    m1!("~power_to_db(", 13, |x: f64| if x <= 0.0 { f64::NEG_INFINITY } else { 10.0 * x.log10() });
    m1!("~unit_step(", 11, |x: f64| if x >= 0.0 { 1.0 } else { 0.0 });
    m1!("~square_wave_math(", 18, |x: f64| if x.sin() >= 0.0 { 1.0 } else { -1.0 });
    m1!("~triangle_wave_math(", 20, |x: f64| (2.0 / std::f64::consts::PI) * x.sin().asin());
    m1!("~sawtooth_math(", 15, |x: f64| x - x.floor());
    m1!("~gaussian_bell(", 15, |x: f64| (-x * x / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt());
    m1!("~cauchy_pdf(", 12, |x: f64| 1.0 / (std::f64::consts::PI * (1.0 + x * x)));
    m1!("~logistic_cdf(", 14, |x: f64| 1.0 / (1.0 + (-x).exp()));
    m1!("~gumbel_pdf(", 12, |x: f64| (-(x + (-x).exp())).exp());
    m1!("~weibull_pdf(", 13, |x: f64| if x < 0.0 { 0.0 } else { (-x).exp() });
    m1!("~rayleigh_pdf(", 14, |x: f64| if x < 0.0 { 0.0 } else { x * (-x * x / 2.0).exp() });
    m1!("~factorial2(", 12, |x: f64| { let n = x as i64; if n < 0 { f64::NAN } else { let mut r = 1.0f64; let mut k = n; while k > 0 { r *= k as f64; k -= 2 } r } });
    m1!("~subfactorial(", 14, |x: f64| { let n = x as i64; if n < 0 { f64::NAN } else { (gamma_l(n as f64 + 1.0) / std::f64::consts::E).round() } });
    m1!("~hyperfactorial(", 16, |x: f64| { let n = x as i64; if n < 0 { f64::NAN } else { (1..=n).map(|k| (k as f64).powf(k as f64)).product() } });
    m1!("~superfactorial2(", 17, |x: f64| { let n = x as i64; if n < 0 { f64::NAN } else { let mut r = 1.0f64; let mut f = 1.0f64; for k in 1..=n { f *= k as f64; r *= f } r } });
    m1!("~partition_num(", 15, |x: f64| { let n = x as i64; if n < 0 { 0.0 } else { let mut p = vec![0.0f64; (n + 1) as usize]; p[0] = 1.0; for i in 1..=n { let mut s = 0.0f64; let mut k = 1i64; loop { let g1 = i - k * (3 * k - 1) / 2; let g2 = i - k * (3 * k + 1) / 2; let sg = if k % 2 == 1 { 1.0 } else { -1.0 }; if g1 < 0 && g2 < 0 { break } if g1 >= 0 { s += sg * p[g1 as usize] } if g2 >= 0 { s += sg * p[g2 as usize] } k += 1 } p[i as usize] = s } p[n as usize] } });
    m1!("~digit_sum_math(", 16, |x: f64| { let mut n = x.abs().round() as i64; let mut s = 0i64; while n > 0 { s += n % 10; n /= 10 } s as f64 });
    m1!("~digital_root2(", 15, |x: f64| { let n = x.abs().round() as i64; if n == 0 { 0.0 } else { let r = n % 9; (if r == 0 { 9 } else { r }) as f64 } });
    m1!("~totient_phi(", 13, |x: f64| { let n = x.round() as i64; if n <= 1 { n as f64 } else { let mut r = n; let mut m = n; let mut k = 2i64; while k * k <= m { if m % k == 0 { while m % k == 0 { m /= k } r -= r / k } k += 1 } if m > 1 { r -= r / m } r as f64 } });
    m1!("~aliquot_sum2(", 14, |x: f64| { let n = x.round() as i64; if n <= 1 { 0.0 } else { let mut s = 1i64; let mut k = 2i64; while k * k <= n { if n % k == 0 { s += k; if k != n / k { s += n / k } } k += 1 } s as f64 } });
    m1!("~primorial(", 11, |x: f64| { let n = x.round() as i64; let mut r = 1.0f64; let mut c = 0i64; let mut k = 2i64; while c < n { if is_prime(k) { r *= k as f64; c += 1 } k += 1 } r });
    m1!("~fib_index(", 11, |x: f64| { let mut a = 0f64; let mut b = 1f64; let mut i = 0f64; while a < x { let t = a + b; a = b; b = t; i += 1.0 } i });
    m1!("~lucas_n(", 9, |x: f64| { let n = x.round() as i64; if n < 0 { f64::NAN } else { let mut a = 2f64; let mut b = 1f64; for _ in 0..n { let t = a + b; a = b; b = t } a } });
    m1!("~pentagonal_n(", 14, |x: f64| { let n = x.round() as f64; n * (3.0 * n - 1.0) / 2.0 });
    m1!("~hexagonal_n(", 13, |x: f64| { let n = x.round() as f64; n * (2.0 * n - 1.0) });
    m1!("~star_number(", 13, |x: f64| { let n = x.round() as f64; 6.0 * n * (n - 1.0) + 1.0 });
    m1!("~centered_square(", 17, |x: f64| { let n = x.round() as f64; n * n + (n - 1.0) * (n - 1.0) });
    m1!("~harm_num_math(", 15, |x: f64| { let n = x.round() as i64; if n <= 0 { 0.0 } else { (1..=n).map(|k| 1.0 / k as f64).sum() } });

    m2!("~mean2(", 7, |a: f64, b: f64| (a + b) / 2.0);
    m2!("~midpoint_math(", 15, |a: f64, b: f64| (a + b) / 2.0);
    m2!("~atan2d_math(", 13, |a: f64, b: f64| a.atan2(b).to_degrees());
    m2!("~polar_angle(", 13, |a: f64, b: f64| a.atan2(b));
    m2!("~mod_int(", 9, |a: f64, b: f64| { let m = b.abs(); let r = a - ((a / m).floor() * m); if r < 0.0 { r + m } else { r } });
    m2!("~mod_float(", 11, |a: f64, b: f64| a % b);
    m2!("~round_dp(", 10, |x: f64, n: f64| { let p = 10.0f64.powf(n.round()); (x * p).round() / p });
    m2!("~floor_dp(", 10, |x: f64, n: f64| { let p = 10.0f64.powf(n.round()); (x * p).floor() / p });
    m2!("~ceil_dp(", 9, |x: f64, n: f64| { let p = 10.0f64.powf(n.round()); (x * p).ceil() / p });
    m2!("~percent_of(", 12, |a: f64, b: f64| a * b / 100.0);
    m2!("~percent_change(", 16, |a: f64, b: f64| if a == 0.0 { f64::NAN } else { (b - a) / a * 100.0 });
    m2!("~percent_diff(", 14, |a: f64, b: f64| { let m = (a + b) / 2.0; if m == 0.0 { f64::NAN } else { (a - b).abs() / m * 100.0 } });
    m2!("~add_pct(", 9, |a: f64, b: f64| a * (1.0 + b / 100.0));
    m2!("~sub_pct(", 9, |a: f64, b: f64| a * (1.0 - b / 100.0));
    m2!("~compound_growth(", 17, |a: f64, r: f64| a * (r / 100.0).exp());
    m2!("~log_base(", 10, |x: f64, base: f64| if base <= 0.0 || base == 1.0 { f64::NAN } else { x.ln() / base.ln() });
    m2!("~log2_ratio(", 12, |a: f64, b: f64| (a / b).log2());
    m2!("~decibels_ratio(", 16, |a: f64, b: f64| 20.0 * (a / b).abs().log10());
    m2!("~power_ratio_db(", 16, |a: f64, b: f64| 10.0 * (a / b).abs().log10());
    m2!("~sqrt_sum_sq(", 13, |a: f64, b: f64| (a * a + b * b).sqrt());
    m2!("~dist_manhattan(", 16, |a: f64, b: f64| a.abs() + b.abs());
    m2!("~dist_chebyshev(", 16, |a: f64, b: f64| a.abs().max(b.abs()));
    m2!("~harmonic_pair(", 15, |a: f64, b: f64| if a == 0.0 || b == 0.0 { 0.0 } else { 2.0 / (1.0 / a + 1.0 / b) });
    m2!("~safe_div(", 10, |a: f64, b: f64| if b == 0.0 { 0.0 } else { a / b });
    m2!("~safe_pow(", 10, |a: f64, b: f64| if a < 0.0 && b.fract() != 0.0 { f64::NAN } else { a.powf(b) });
    m2!("~safe_sqrt(", 11, |a: f64, _: f64| if a < 0.0 { 0.0 } else { a.sqrt() });
    m2!("~mod_neg(", 9, |a: f64, b: f64| { ((a % b) + b) % b });
    m2!("~growth_rate(", 13, |a: f64, b: f64| if a == 0.0 { f64::NAN } else { (b - a) / a });
    m2!("~decay_rate(", 12, |a: f64, b: f64| if a == 0.0 { f64::NAN } else { (a - b) / a });
    m2!("~doubling_time(", 15, |_: f64, r: f64| if r == 0.0 { f64::INFINITY } else { 70.0 / r.abs() });
    m2!("~half_life_time(", 16, |_: f64, r: f64| if r == 0.0 { f64::INFINITY } else { 69.3 / r.abs() });
    m2!("~tip_amount(", 12, |a: f64, b: f64| a * b / 100.0);
    m2!("~tax_amount(", 12, |a: f64, b: f64| a * b / 100.0);
    m2!("~gross_from_net(", 16, |a: f64, b: f64| if b >= 100.0 { f64::NAN } else { a / (1.0 - b / 100.0) });
    m2!("~net_from_gross(", 16, |a: f64, b: f64| a * (1.0 - b / 100.0));
    m2!("~a_percent_of_b(", 16, |a: f64, b: f64| if b == 0.0 { f64::NAN } else { a / b * 100.0 });
    m2!("~nth_root(", 10, |a: f64, b: f64| if b == 0.0 { f64::NAN } else { a.powf(1.0 / b) });
    m2!("~slope_deg(", 11, |a: f64, b: f64| a.atan2(b).to_degrees());
    m2!("~bearing_deg(", 13, |a: f64, b: f64| { ((a.atan2(b).to_degrees() + 360.0) % 360.0) });
    m2!("~remain_floor(", 14, |a: f64, b: f64| a - (a / b).floor() * b);
    m2!("~round_half_up(", 15, |a: f64, b: f64| if b == 0.0 { a } else { (a / b).round() * b });
    m2!("~round_half_even(", 17, |a: f64, b: f64| if b == 0.0 { a } else { (a / b).round_ties_even() * b });
    m2!("~step_round(", 12, |a: f64, b: f64| if b == 0.0 { a } else { (a / b).round() * b });
    m2!("~floor_multiple(", 16, |a: f64, b: f64| if b == 0.0 { a } else { (a / b).floor() * b });
    m2!("~ceil_multiple(", 15, |a: f64, b: f64| if b == 0.0 { a } else { (a / b).ceil() * b });
    m2!("~quad_mean(", 11, |a: f64, b: f64| ((a * a + b * b) / 2.0).sqrt());
    m2!("~pow_int(", 9, |a: f64, b: f64| a.powi(b.round() as i32));
    m2!("~int_root_floor(", 16, |a: f64, b: f64| { let n = a.round(); let r = b.round() as i64; if r <= 0 { f64::NAN } else { let mut lo = 0.0f64; let mut hi = n.max(1.0); while hi - lo > 1.0 { let mid = ((lo + hi) / 2.0).floor(); if mid.powi(r as i32) <= n { lo = mid } else { hi = mid } } lo } });
    m2!("~kelly_fraction(", 16, |a: f64, p: f64| if a == 0.0 { f64::NAN } else { (a * p - (1.0 - p)) / a });
    m2!("~prob_odds(", 11, |a: f64, _: f64| if a <= 0.0 || a >= 1.0 { f64::NAN } else { a / (1.0 - a) });
    m1!("~tau_num(", 9, |_: f64| 2.0 * std::f64::consts::PI);
    m1!("~e_const(", 9, |_: f64| std::f64::consts::E);
    m1!("~euler_gamma_num(", 17, |_: f64| 0.5772156649015329);
    m1!("~plastic_number(", 16, |_: f64| 1.324717957244746);
    m1!("~feigenbaum(", 12, |_: f64| 4.66920160910299);
    m1!("~apery_const(", 13, |_: f64| 1.2020569031595942);
    m1!("~khinchin_const(", 16, |_: f64| 2.6854520010);
    m1!("~gauss_const(", 13, |_: f64| 0.8346268416740732);
    m1!("~omega_const(", 13, |_: f64| 0.5671432904097838);
    m1!("~catalan_const2(", 16, |_: f64| 0.915965594177219);
    m1!("~glaisher_const(", 16, |_: f64| 1.2824271291006226);
    m1!("~brun_const(", 12, |_: f64| 1.90216058);
    m1!("~silver_ratio2(", 15, |_: f64| 2.414213562373095);
    m1!("~conway_const(", 14, |_: f64| 1.303577269034296);
    m1!("~sind(", 6, |x: f64| x.to_radians().sin());
    m1!("~cosd(", 6, |x: f64| x.to_radians().cos());
    m1!("~tand(", 6, |x: f64| x.to_radians().tan());
    m1!("~asind(", 7, |x: f64| x.asin().to_degrees());
    m1!("~acosd(", 7, |x: f64| x.acos().to_degrees());
    m1!("~atand(", 7, |x: f64| x.atan().to_degrees());
    m1!("~csc_rad(", 9, |x: f64| if x.sin().abs() < 1e-12 { f64::NAN } else { 1.0 / x.sin() });
    m1!("~sec_rad(", 9, |x: f64| if x.cos().abs() < 1e-12 { f64::NAN } else { 1.0 / x.cos() });
    m1!("~cot_rad(", 9, |x: f64| if x.tan().abs() < 1e-12 { f64::NAN } else { 1.0 / x.tan() });
    m1!("~asec_rad(", 10, |x: f64| if x.abs() < 1.0 { f64::NAN } else { (1.0 / x).acos() });
    m1!("~acsc_rad(", 10, |x: f64| if x.abs() < 1.0 { f64::NAN } else { (1.0 / x).asin() });
    m1!("~acot_rad(", 10, |x: f64| (1.0 / x).atan());
    m1!("~asec_deg(", 10, |x: f64| if x.abs() < 1.0 { f64::NAN } else { (1.0 / x).acos().to_degrees() });
    m1!("~acsc_deg(", 10, |x: f64| if x.abs() < 1.0 { f64::NAN } else { (1.0 / x).asin().to_degrees() });
    m1!("~acot_deg(", 10, |x: f64| (1.0 / x).atan().to_degrees());
    m1!("~versin(", 8, |x: f64| 1.0 - x.cos());
    m1!("~coversin(", 10, |x: f64| 1.0 - x.sin());
    m1!("~haversin(", 10, |x: f64| (x / 2.0).sin().powi(2));
    m1!("~exsec(", 7, |x: f64| if x.cos().abs() < 1e-12 { f64::NAN } else { 1.0 / x.cos() - 1.0 });
    m1!("~excsc(", 7, |x: f64| if x.sin().abs() < 1e-12 { f64::NAN } else { 1.0 / x.sin() - 1.0 });
    m1!("~next_pow2(", 11, |x: f64| { let n = x.round() as i64; if n <= 0 { 1.0 } else { 2.0f64.powi(64 - (n as u64).leading_zeros() as i32) } });
    m1!("~prev_pow2(", 11, |x: f64| { let n = x.round() as i64; if n <= 0 { 1.0 } else { 2.0f64.powi(63 - (n as u64).leading_zeros() as i32) } });
    m1!("~is_power_of_two(", 17, |x: f64| { let n = x.round() as i64; (n > 0 && (n & (n - 1)) == 0) as i64 as f64 });
    m1!("~popcount(", 10, |x: f64| { (x.round() as u64).count_ones() as f64 });
    m1!("~trail_zeros(", 13, |x: f64| { let n = x.round() as i64; if n == 0 { 0.0 } else { let mut k = n; let mut c = 0i64; while k % 2 == 0 { k /= 2; c += 1 } c as f64 } });
    m1!("~leading_zeros(", 15, |x: f64| { (x.round() as u64).leading_zeros() as f64 });
    m1!("~bit_reverse(", 13, |x: f64| { let mut v = x.round() as u64; let mut r = 0u64; for _ in 0..64 { r = (r << 1) | (v & 1); v >>= 1 } r as f64 });
    m1!("~gray_code(", 11, |x: f64| { let n = x.round() as u64; (n ^ (n >> 1)) as f64 });
    m1!("~bit_parity(", 12, |x: f64| if (x.round() as u64).count_ones() % 2 == 0 { 1.0 } else { 0.0 });
    m1!("~collatz_next(", 14, |x: f64| { let n = x.round() as i64; if n % 2 == 0 { (n / 2) as f64 } else { (3 * n + 1) as f64 } });
    m1!("~collatz_len(", 13, |x: f64| { let mut n = x.round() as i64; let mut c = 0i64; while n != 1 { if n % 2 == 0 { n /= 2 } else { n = 3 * n + 1 } c += 1 } c as f64 });
    m1!("~collatz_peak(", 14, |x: f64| { let mut n = x.round() as i64; let mut m = n; while n != 1 { if n % 2 == 0 { n /= 2 } else { n = 3 * n + 1 } m = m.max(n) } m as f64 });
    m1!("~reverse_number(", 16, |x: f64| { let s = x.round().abs().to_string(); s.chars().rev().collect::<String>().parse::<f64>().unwrap_or(0.0) });
    m1!("~digit_num_count(", 17, |x: f64| x.round().abs().to_string().len() as f64);
    m1!("~digits_prod(", 13, |x: f64| x.round().abs().to_string().chars().filter_map(|c| c.to_digit(10)).fold(1i64, |a, d| a * d as i64) as f64);
    m1!("~narcissistic_check(", 20, |x: f64| { let s = x.round().abs().to_string(); let n = s.len() as u32; let sum: i64 = s.chars().filter_map(|c| c.to_digit(10)).map(|d| (d as i64).pow(n)).sum(); (sum == x.round().abs() as i64) as i64 as f64 });
    m1!("~perfect_check(", 15, |x: f64| { let n = x.round() as i64; if n <= 1 { 0.0 } else { let mut s = 1i64; let mut k = 2i64; while k * k <= n { if n % k == 0 { s += k; if k != n / k { s += n / k } } k += 1 } (s == n) as i64 as f64 } });
    m1!("~abundant_check(", 16, |x: f64| { let n = x.round() as i64; if n <= 1 { 0.0 } else { let mut s = 1i64; let mut k = 2i64; while k * k <= n { if n % k == 0 { s += k; if k != n / k { s += n / k } } k += 1 } (s > n) as i64 as f64 } });
    m1!("~deficient_check(", 17, |x: f64| { let n = x.round() as i64; if n <= 1 { 0.0 } else { let mut s = 1i64; let mut k = 2i64; while k * k <= n { if n % k == 0 { s += k; if k != n / k { s += n / k } } k += 1 } (s < n) as i64 as f64 } });
    m1!("~happy_check(", 13, |x: f64| { let mut n = x.round().abs() as i64; let mut seen = std::collections::HashSet::new(); while n != 1 && !seen.contains(&n) { seen.insert(n); n = n.to_string().chars().filter_map(|c| c.to_digit(10)).map(|d| (d * d) as i64).sum() } (n == 1) as i64 as f64 });
    m1!("~kaprekar_check(", 16, |x: f64| { let n = x.round() as i64; if n <= 0 { 0.0 } else { let sq = n * n; let s = sq.to_string(); let d = s.len() / 2; let (a, b) = s.split_at(s.len().saturating_sub(d)); let a = a.parse::<i64>().unwrap_or(0); let b = b.parse::<i64>().unwrap_or(0); (a + b == n) as i64 as f64 } });
    m1!("~palindrome_num_check(", 22, |x: f64| { let s = x.round().abs().to_string(); (s.chars().eq(s.chars().rev())) as i64 as f64 });
    m1!("~automorphic_check(", 19, |x: f64| { let n = x.round().abs() as i64; let sq = (n * n).to_string(); (sq.ends_with(&n.to_string())) as i64 as f64 });
    m1!("~trailing_zero_fact(", 20, |x: f64| { let n = x.round() as i64; if n < 0 { 0.0 } else { let mut c = 0i64; let mut k = 5i64; while k <= n { c += n / k; k *= 5 } c as f64 } });
    m1!("~tribonacci_n(", 14, |x: f64| { let n = x.round() as i64; if n <= 0 { 0.0 } else { let mut a = 0.0; let mut b = 0.0; let mut c = 1.0; for _ in 1..n { let t = a + b + c; a = b; b = c; c = t } c } });
    m1!("~tetranacci_n(", 14, |x: f64| { let n = x.round() as i64; if n <= 0 { 0.0 } else { let mut a = 0.0; let mut b = 0.0; let mut c = 0.0; let mut d = 1.0; for _ in 1..n { let t = a + b + c + d; a = b; b = c; c = d; d = t } d } });
    m1!("~jacobsthal_n(", 14, |x: f64| { let n = x.round() as i64; if n < 0 { f64::NAN } else { (2.0f64.powi(n as i32) - (-1.0f64).powi(n as i32)) / 3.0 } });
    m1!("~padovan_n(", 11, |x: f64| { let n = x.round() as i64; if n <= 0 { 1.0 } else { let mut a = 1.0; let mut b = 1.0; let mut c = 1.0; for _ in 1..n { let t = a + b; a = b; b = c; c = t } c } });
    m1!("~perrin_n(", 10, |x: f64| { let n = x.round() as i64; if n <= 0 { 3.0 } else { let mut a = 3.0; let mut b = 0.0; let mut c = 2.0; for _ in 1..n { let t = a + b; a = b; b = c; c = t } c } });
    m1!("~motzkin_n(", 11, |x: f64| { let n = x.round() as i64; if n <= 0 { 1.0 } else { let mut m = vec![0.0f64; (n + 1) as usize]; m[0] = 1.0; m[1] = 1.0; for i in 2..=n { m[i as usize] = m[(i - 1) as usize] + m[(i - 2) as usize]; } m[n as usize] } });
    m1!("~schroeder_n(", 13, |x: f64| { let n = x.round() as i64; if n <= 0 { 1.0 } else { let mut s = vec![0.0f64; (n + 1) as usize]; s[0] = 1.0; for i in 1..=n { s[i as usize] = s[(i - 1) as usize] + (0..i).map(|k| s[k as usize] * s[(i - 1 - k) as usize]).sum::<f64>() } s[n as usize] } });
    m1!("~central_binomial(", 18, |x: f64| { let n = x.round() as usize; if n == 0 { 1.0 } else { let mut row = vec![0u64; 2 * n + 1]; row[0] = 1; for _ in 1..=(2 * n) { for j in (1..=(2 * n)).rev() { row[j] += row[j - 1]; } } row[n] as f64 } });
    m1!("~fib_golden_ratio(", 18, |_: f64| std::f64::consts::FRAC_1_SQRT_2 * 0.0 + 0.6180339887498949);
    m1!("~phi_inv(", 9, |_: f64| 0.6180339887498949);
    m1!("~sqrt_phi(", 10, |_: f64| 1.272019649514069);
    m1!("~binet_n(", 9, |x: f64| { let n = x.round() as i64; if n < 0 { f64::NAN } else { let phi: f64 = 1.618033988749895; let psi: f64 = -0.6180339887498949; ((phi.powi(n as i32) - psi.powi(n as i32)) / 5.0f64.sqrt()).round() } });
    m1!("~catalan_number_n(", 18, |x: f64| { let n = x.round() as usize; if n == 0 { 1.0 } else { let mut row = vec![0u64; 2 * n + 1]; row[0] = 1; for _ in 1..=(2 * n) { for j in (1..=(2 * n)).rev() { row[j] += row[j - 1]; } } row[n] as f64 / (n + 1) as f64 } });
    m1!("~avogadro_number(", 17, |_: f64| 6.02214076e23);
    m1!("~boltzmann_const(", 17, |_: f64| 1.380649e-23);
    m1!("~planck_const2(", 15, |_: f64| 6.62607015e-34);
    m1!("~planck_reduced(", 16, |_: f64| 1.054571817e-34);
    m1!("~stefan_bc(", 11, |_: f64| 5.670374419e-8);
    m1!("~coulomb_const2(", 16, |_: f64| 8.9875517923e9);
    m1!("~faraday_const(", 15, |_: f64| 96485.33212);
    m1!("~gas_const2(", 12, |_: f64| 8.314462618);
    m1!("~rydberg_const(", 15, |_: f64| 10973731.56816);
    m1!("~fine_struct2(", 14, |_: f64| 0.0072973525693);
    m1!("~electron_mass2(", 16, |_: f64| 9.1093837015e-31);
    m1!("~proton_mass2(", 14, |_: f64| 1.67262192369e-27);
    m1!("~neutron_mass2(", 15, |_: f64| 1.67492749804e-27);
    m1!("~atomic_mass_unit(", 18, |_: f64| 1.66053906660e-27);
    m1!("~bohr_radius2(", 14, |_: f64| 5.29177210903e-11);
    m1!("~classical_electron_radius(", 27, |_: f64| 2.8179403262e-15);
    m1!("~magnetic_constant(", 19, |_: f64| 1.25663706212e-6);
    m1!("~electric_constant(", 19, |_: f64| 8.8541878128e-12);
    m1!("~sum_n2(", 8, |x: f64| { let n = x.round() as i64; if n < 0 { 0.0 } else { (n * (n + 1) * (2 * n + 1) / 6) as f64 } });
    m1!("~sum_n3(", 8, |x: f64| { let n = x.round() as i64; if n < 0 { 0.0 } else { let s = n * (n + 1) / 2; (s * s) as f64 } });
    m1!("~cube_volume2(", 14, |s: f64| s.powi(3));
    m1!("~sphere_diameter(", 17, |r: f64| 2.0 * r);
    m1!("~circle_diameter(", 17, |r: f64| 2.0 * r);
    m1!("~circle_diameter_from_area(", 27, |a: f64| 2.0 * (a / std::f64::consts::PI).sqrt());
    m1!("~rising_factorial(", 18, |x: f64| { let n = x.round() as i64; (0..n).fold(1.0f64, |acc, k| acc * (1.0 + k as f64)) });
    m1!("~falling_factorial(", 19, |x: f64| { let n = x.round() as i64; (0..n).fold(1.0f64, |acc, k| acc * (x - k as f64)) });
    m1!("~double_fact2(", 14, |x: f64| { let n = x.round() as i64; if n < 0 { f64::NAN } else { let mut r = 1.0f64; let mut k = n; while k > 0 { r *= k as f64; k -= 2 } r } });
    m1!("~zeta2_const(", 13, |_: f64| std::f64::consts::PI * std::f64::consts::PI / 6.0);

    m2!("~mean_geometric2(", 17, |a: f64, b: f64| (a * b).sqrt());
    m2!("~mean_contraharmonic(", 21, |a: f64, b: f64| if a + b == 0.0 { f64::NAN } else { (a * a + b * b) / (a + b) });
    m2!("~rect_perimeter(", 16, |w: f64, h: f64| 2.0 * (w + h));
    m2!("~square_perim2(", 15, |s: f64, _: f64| 4.0 * s);
    m2!("~triangle_area2(", 16, |b: f64, h: f64| b * h / 2.0);
    m2!("~para_area2(", 12, |b: f64, h: f64| b * h);
    m2!("~geom_series_sum(", 17, |a: f64, r: f64| if r.abs() >= 1.0 { if r == 1.0 { a * 1e308 } else { f64::NAN } } else { a / (1.0 - r) });
    m2!("~db_add(", 8, |a: f64, b: f64| 10.0 * (10.0f64.powf(a / 10.0) + 10.0f64.powf(b / 10.0)).log10());
    m2!("~spl_db(", 8, |a: f64, b: f64| 10.0 * (10.0f64.powf(a / 10.0) + 10.0f64.powf(b / 10.0)).log10());
    m2!("~fov_deg(", 9, |f: f64, sensor: f64| if f == 0.0 { f64::NAN } else { 2.0 * (sensor / (2.0 * f)).atan().to_degrees() });
    m2!("~deg_to_gon2(", 13, |x: f64, _: f64| x * 400.0 / 360.0);
    m2!("~gon_to_deg2(", 13, |x: f64, _: f64| x * 360.0 / 400.0);
    m2!("~odds_to_prob(", 14, |p: f64, _: f64| p / (1.0 + p));
    m2!("~decimal_odds(", 14, |p: f64, _: f64| if p <= 0.0 || p >= 1.0 { f64::NAN } else { 1.0 / p });
    m2!("~american_odds(", 15, |p: f64, _: f64| if p <= 0.0 || p >= 1.0 { f64::NAN } else if p >= 0.5 { (p / (1.0 - p) * 100.0).round() } else { (-(1.0 - p) / p * 100.0).round() });
    m2!("~implied_prob(", 14, |o: f64, _: f64| if o <= 1.0 { f64::NAN } else { 1.0 / o });
    m2!("~fractional_odds(", 17, |p: f64, _: f64| if p <= 0.0 { f64::NAN } else { (p - 1.0).round() });
    m2!("~ev_calc(", 9, |p: f64, v: f64| p * v);
    m2!("~expected_value(", 16, |w: f64, p: f64| w * p);
    m2!("~variance_of_pair(", 18, |a: f64, b: f64| { let m = (a + b) / 2.0; ((a - m).powi(2) + (b - m).powi(2)) / 2.0 });
    m2!("~ellipse_eccentricity(", 22, |a: f64, b: f64| if a <= 0.0 { f64::NAN } else { (1.0 - (b * b) / (a * a)).max(0.0).sqrt() });
    m2!("~synodic_period(", 16, |p1: f64, p2: f64| if p1 == 0.0 || p2 == 0.0 || p1 == p2 { f64::NAN } else { (p1 * p2).abs() / (p1 - p2).abs() });
    m2!("~mean_anomaly(", 14, |t: f64, p: f64| if p == 0.0 { f64::NAN } else { (t / p * 360.0) % 360.0 });
    m2!("~eccentric_anomaly(", 19, |m: f64, e: f64| { let m = m.to_radians(); let mut e2 = m; for _ in 0..10 { e2 = m + e * e2.sin() } e2.to_degrees() });
    m2!("~rising_diagonal(", 17, |a: f64, b: f64| (a * a + b * b).sqrt());

    // inc/dec меняют переменную на месте чтобы не писать присваивание вручную

    if code.starts_with("~inc ") {
        let name = code[5..].trim();
        let n = match vm.get_var(name) {
            Some(v) => match v.as_number() {
                Some(num) => num + 1.0,
                None => return Some(Err(format!("'{}' not numeric", name))),
            },
            None => return Some(Err(format!("'{}' not found", name))),
        };
        match vm.try_set_var(name, Value::Number(n)) {
            Ok(_) => {
                vm.echo(format!("{}", n));
                return Some(Ok(Value::Number(n)));
            }
            Err(e) => return Some(Err(e)),
        }
    }

    if code.starts_with("~dec ") {
        let name = code[5..].trim();
        let n = match vm.get_var(name) {
            Some(v) => match v.as_number() {
                Some(num) => num - 1.0,
                None => return Some(Err(format!("'{}' not numeric", name))),
            },
            None => return Some(Err(format!("'{}' not found", name))),
        };
        match vm.try_set_var(name, Value::Number(n)) {
            Ok(_) => {
                vm.echo(format!("{}", n));
                return Some(Ok(Value::Number(n)));
            }
            Err(e) => return Some(Err(e)),
        }
    }

    if code.starts_with("~inc_by(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let delta = match vm.eval_expr(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let current = vm.get_var(name).and_then(|v| v.as_number()).unwrap_or(0.0);
            let new_val = current + delta;
            match vm.try_set_var(name, Value::Number(new_val)) {
                Ok(_) => {
                    vm.echo(&format!("{}", new_val));
                    return Some(Ok(Value::Number(new_val)));
                }
                Err(e) => return Some(Err(e)),
            }
        }
        return Some(Err("~inc_by(var, delta)".to_string()));
    }

    if code.starts_with("~dec_by(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let delta = match vm.eval_expr(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let current = vm.get_var(name).and_then(|v| v.as_number()).unwrap_or(0.0);
            let new_val = current - delta;
            match vm.try_set_var(name, Value::Number(new_val)) {
                Ok(_) => {
                    vm.echo(&format!("{}", new_val));
                    return Some(Ok(Value::Number(new_val)));
                }
                Err(e) => return Some(Err(e)),
            }
        }
        return Some(Err("~dec_by(var, delta)".to_string()));
    }

    if code.starts_with("~mul_var(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let factor = match vm.eval_expr(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let current = vm.get_var(name).and_then(|v| v.as_number()).unwrap_or(0.0);
            let new_val = current * factor;
            match vm.try_set_var(name, Value::Number(new_val)) {
                Ok(_) => {
                    vm.echo(&format!("{}", new_val));
                    return Some(Ok(Value::Number(new_val)));
                }
                Err(e) => return Some(Err(e)),
            }
        }
        return Some(Err("~mul_var(var, factor)".to_string()));
    }

    if code.starts_with("~div_var(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let divisor = match vm.eval_expr(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            if divisor == 0.0 {
                return Some(Err("Division by zero".to_string()));
            }
            let current = vm.get_var(name).and_then(|v| v.as_number()).unwrap_or(0.0);
            let new_val = current / divisor;
            match vm.try_set_var(name, Value::Number(new_val)) {
                Ok(_) => {
                    vm.echo(&format!("{}", new_val));
                    return Some(Ok(Value::Number(new_val)));
                }
                Err(e) => return Some(Err(e)),
            }
        }
        return Some(Err("~div_var(var, divisor)".to_string()));
    }

    // Функции одного числа считаем тут чтобы скрипт оставался коротким

    m1!("~abs(", 5, f64::abs);
    m1!("~sqrt(", 6, f64::sqrt);
    m1!("~cbrt(", 6, |x: f64| x.cbrt());
    m1!("~sin(", 5, f64::sin);
    m1!("~cos(", 5, f64::cos);
    m1!("~tan(", 5, f64::tan);
    m1!("~asin(", 6, f64::asin);
    m1!("~acos(", 6, f64::acos);
    m1!("~sinh(", 6, f64::sinh);
    m1!("~cosh(", 6, f64::cosh);
    m1!("~tanh(", 6, f64::tanh);
    m1!("~floor(", 7, f64::floor);
    m1!("~ceil(", 6, f64::ceil);
    m1!("~round(", 7, f64::round);
    m1!("~trunc(", 7, f64::trunc);
    m1!("~fract(", 7, f64::fract);
    m1!("~log(", 5, f64::ln);
    m1!("~log2(", 6, |x: f64| x.log2());
    m1!("~log10(", 7, |x: f64| x.log10());
    m1!("~exp(", 5, f64::exp);
    m1!("~deg(", 5, f64::to_degrees);
    m1!("~rad(", 5, f64::to_radians);
    m1!("~sign(", 6, |x: f64| if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 });

    // Функции двух чисел разбирают аргументы тут чтобы не ошибиться с запятыми

    macro_rules! math2 {
        ($p:expr, $l:expr, $f:expr) => {
            if code.starts_with($p) && code.ends_with(')') {
                let parts = VM::split_top_level(&code[$l..code.len() - 1], ',');
                if parts.len() == 2 {
                    let a = match vm.eval_expr(&parts[0]) {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
                    };
                    let b = match vm.eval_expr(&parts[1]) {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
                    };
                    let result = ($f)(a, b);
                    vm.echo(&format!("{}", result));
                    return Some(Ok(Value::Number(result)));
                }
            }
        };
    }

    math2!("~atan2(", 7, |y: f64, x: f64| y.atan2(x));
    math2!("~min(", 5, f64::min);
    math2!("~max(", 5, f64::max);
    math2!("~hypot(", 7, |a: f64, b: f64| (a.powi(2) + b.powi(2)).sqrt());
    math2!("~gcd(", 5, |a: f64, b: f64| {
        let gcd_fn = |mut a: i64, mut b: i64| {
            while b != 0 { let t = b; b = a % b; a = t; }
            a.abs()
        };
        gcd_fn(a as i64, b as i64) as f64
    });
    math2!("~lcm(", 5, |a: f64, b: f64| {
        let gcd_fn = |mut a: i64, mut b: i64| {
            while b != 0 { let t = b; b = a % b; a = t; }
            a.abs()
        };
        let (a, b) = (a as i64, b as i64);
        if a == 0 || b == 0 { 0.0 } else { (a / gcd_fn(a, b) * b).abs() as f64 }
    });
    math2!("~pow(", 5, f64::powf);
    math2!("~root(", 6, |x: f64, n: f64| x.powf(1.0 / n));

    // clamp/lerp/between держат число в границах чтобы оно не улетело за допустимое

    if code.starts_with("~clamp(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() == 3 {
            let x = match vm.eval_expr(&p[0]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let lo = match vm.eval_expr(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let hi = match vm.eval_expr(&p[2]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let result = x.max(lo).min(hi);
            vm.echo(&format!("{}", result));
            return Some(Ok(Value::Number(result)));
        }
    }

    if code.starts_with("~lerp(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 3 {
            let a = match vm.eval_expr(&p[0]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let b = match vm.eval_expr(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let t = match vm.eval_expr(&p[2]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let result = a + (b - a) * t;
            vm.echo(&format!("{}", result));
            return Some(Ok(Value::Number(result)));
        }
    }

    if code.starts_with("~between(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 3 {
            let x = match vm.eval_expr(&p[0]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let lo = match vm.eval_expr(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let hi = match vm.eval_expr(&p[2]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let ok = x >= lo && x <= hi;
            vm.echo(&format!("{}", ok));
            return Some(Ok(Value::Bool(ok)));
        }
    }

    if code.starts_with("~map(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() == 5 {
            let x = match vm.eval_expr(&p[0]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let il = match vm.eval_expr(&p[1]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let ih = match vm.eval_expr(&p[2]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let ol = match vm.eval_expr(&p[3]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let oh = match vm.eval_expr(&p[4]) {
                Ok(v) => v,
                Err(e) => return Some(Err(e)),
            };
            let result = ol + (x - il) * (oh - ol) / (ih - il);
            vm.echo(&format!("{}", result));
            return Some(Ok(Value::Number(result)));
        }
    }

    // Факториал и простые числа считаем целиком тут чтобы скрипт не гонял свои циклы

    if code.starts_with("~fact(") && code.ends_with(')') {
        let n = match vm.eval_expr(&code[6..code.len() - 1]) {
            Ok(v) => v as u64,
            Err(e) => return Some(Err(e)),
        };
        let result = (1..=n.min(20)).product::<u64>() as f64;
        vm.echo(&format!("{}", result));
        return Some(Ok(Value::Number(result)));
    }

    if code.starts_with("~fib(") && code.ends_with(')') {
        let n = match vm.eval_expr(&code[5..code.len() - 1]) {
            Ok(v) => v as u64,
            Err(e) => return Some(Err(e)),
        };
        let (mut a, mut b) = (0u64, 1u64);
        for _ in 0..n {
            let t = a + b;
            a = b;
            b = t;
        }
        vm.echo(format!("{}", a));
        return Some(Ok(Value::Number(a as f64)));
    }

    if code.starts_with("~is_prime(") && code.ends_with(')') {
        let n = match vm.eval_expr(&code[10..code.len() - 1]) {
            Ok(v) => v as u64,
            Err(e) => return Some(Err(e)),
        };
        let is_prime = |x: u64| -> bool {
            if x < 2 { return false; }
            if x == 2 { return true; }
            if x % 2 == 0 { return false; }
            let mut i = 3;
            while i * i <= x {
                if x % i == 0 { return false; }
                i += 2;
            }
            true
        };
        let ok = is_prime(n);
        vm.echo(&format!("{}", ok));
        return Some(Ok(Value::Bool(ok)));
    }

    if code.starts_with("~prime(") && code.ends_with(')') {
        let n = match vm.eval_expr(&code[7..code.len() - 1]) {
            Ok(v) => v as u64,
            Err(e) => return Some(Err(e)),
        };
        let mut count = 0u64;
        let mut candidate = 1u64;
        while count < n {
            candidate += 1;
            let is_prime = |x: u64| -> bool {
                if x < 2 { return false; }
                if x == 2 { return true; }
                if x % 2 == 0 { return false; }
                let mut i = 3;
                while i * i <= x {
                    if x % i == 0 { return false; }
                    i += 2;
                }
                true
            };
            if is_prime(candidate) {
                count += 1;
            }
        }
        vm.echo(&format!("{}", candidate));
        return Some(Ok(Value::Number(candidate as f64)));
    }

    // Римские и порядковые нужны чтобы число читалось человеком а не пугало

    if code.starts_with("~to_ordinal(") && code.ends_with(')') {
        let n = match vm.eval_expr(&code[12..code.len() - 1]) {
            Ok(v) => v as i64,
            Err(e) => return Some(Err(e)),
        };
        let abs = n.unsigned_abs();
        let suffix = match abs % 100 {
            11 | 12 | 13 => "th",
            _ => match abs % 10 {
                1 => "st",
                2 => "nd",
                3 => "rd",
                _ => "th",
            },
        };
        let s = format!("{}{}", n, suffix);
        vm.echo(format!("{}", s));
        return Some(Ok(Value::Text(s)));
    }

    if code.starts_with("~to_roman(") && code.ends_with(')') {
        let mut n = match vm.eval_expr(&code[10..code.len() - 1]) {
            Ok(v) => v as u64,
            Err(e) => return Some(Err(e)),
        };
        const V: &[(u64, &str)] = &[
            (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
            (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
            (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
        ];
        let mut out = String::new();
        for &(val, s) in V {
            while n >= val {
                out.push_str(s);
                n -= val;
            }
        }
        vm.echo(&format!("{}", out));
        return Some(Ok(Value::Text(out)));
    }

    if code.starts_with("~from_roman(") && code.ends_with(')') {
        let s = VM::tq(&code[12..code.len() - 1]);
        let m: std::collections::HashMap<char, u64> = [
            ('I', 1), ('V', 5), ('X', 10), ('L', 50),
            ('C', 100), ('D', 500), ('M', 1000),
        ].iter().cloned().collect();
        let mut total = 0u64;
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let v = *m.get(&chars[i]).unwrap_or(&0);
            let nv = if i + 1 < chars.len() { *m.get(&chars[i + 1]).unwrap_or(&0) } else { 0 };
            if v < nv {
                total += nv - v;
                i += 2;
            } else {
                total += v;
                i += 1;
            }
        }
        vm.echo(&format!("{}", total));
        return Some(Ok(Value::Number(total as f64)));
    }

    // Кубик и пароль даем тут чтобы скрипту не требовался внешний генератор

    if code == "~randf()" || code == "~randf" || code == "~rndf()" || code == "~rndf" {
        let n = rand::thread_rng().gen::<f64>();
        vm.echo(format!("{}", n));
        return Some(Ok(Value::Number(n)));
    }

    if code == "~rndbool()" || code == "~rndbool" {
        let b = rand::thread_rng().gen::<bool>();
        vm.echo(format!("{}", b));
        return Some(Ok(Value::Bool(b)));
    }

    if code == "~coin()" || code == "~coin" {
        let s = if rand::thread_rng().gen::<bool>() { "heads" } else { "tails" };
        vm.echo(format!("{}", s));
        return Some(Ok(Value::Text(s.to_string())));
    }

    if code.starts_with("~dice(") && code.ends_with(')') {
        let n = match vm.eval_expr(&code[6..code.len() - 1]) {
            Ok(v) => v as u64,
            Err(e) => return Some(Err(e)),
        };
        let result = rand::thread_rng().gen_range(1..=n.max(1)) as f64;
        vm.echo(&format!("{}", result));
        return Some(Ok(Value::Number(result)));
    }

    if code.starts_with("~rndcolor(") && code.ends_with(')') {
        let r = rand::thread_rng().gen::<u8>();
        let g = rand::thread_rng().gen::<u8>();
        let b = rand::thread_rng().gen::<u8>();
        let s = format!("{},{},{}", r, g, b);
        vm.echo(format!("{}", s));
        return Some(Ok(Value::Text(s)));
    }

    if code.starts_with("~rnd(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() == 2 {
            let lo = match vm.eval_expr(&p[0]) {
                Ok(v) => v as i64,
                Err(e) => return Some(Err(e)),
            };
            let hi = match vm.eval_expr(&p[1]) {
                Ok(v) => v as i64,
                Err(e) => return Some(Err(e)),
            };
            let n = rand::thread_rng().gen_range(lo..=hi);
            vm.echo(n.to_string());
            return Some(Ok(Value::Number(n as f64)));
        }
    }

    if code.starts_with("~randint(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len() - 1], ',');
        if p.len() == 2 {
            let lo = match vm.eval_expr(&p[0]) {
                Ok(v) => v as i64,
                Err(e) => return Some(Err(e)),
            };
            let hi = match vm.eval_expr(&p[1]) {
                Ok(v) => v as i64,
                Err(e) => return Some(Err(e)),
            };
            let n = rand::thread_rng().gen_range(lo..=hi);
            vm.echo(format!("{}", n));
            return Some(Ok(Value::Number(n as f64)));
        }
    }

    if code.starts_with("~randrange(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len() - 1], ',');
        if p.len() == 2 {
            let lo = match vm.eval_expr(&p[0]) {
                Ok(v) => v as i64,
                Err(e) => return Some(Err(e)),
            };
            let hi = match vm.eval_expr(&p[1]) {
                Ok(v) => v as i64,
                Err(e) => return Some(Err(e)),
            };
            if hi <= lo {
                return Some(Err("~randrange: hi must be > lo".to_string()));
            }
            let n = rand::thread_rng().gen_range(lo..hi);
            vm.echo(format!("{}", n));
            return Some(Ok(Value::Number(n as f64)));
        }
    }

    if code.starts_with("~rnds(") && code.ends_with(')') {
        let len = match vm.eval_expr(&code[6..code.len() - 1]) {
            Ok(v) => v as usize,
            Err(e) => return Some(Err(e)),
        };
        let s: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(len)
            .map(char::from)
            .collect();
        vm.echo(format!("{}", s));
        return Some(Ok(Value::Text(s)));
    }

    if code.starts_with("~passwd(") && code.ends_with(')') {
        let len = match vm.eval_expr(&code[8..code.len() - 1]) {
            Ok(v) => v as usize,
            Err(e) => return Some(Err(e)),
        };
        let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*"
            .chars()
            .collect();
        let s: String = (0..len)
            .map(|_| chars[rand::thread_rng().gen_range(0..chars.len())])
            .collect();
        vm.echo(format!("{}", s));
        return Some(Ok(Value::Text(s)));
    }

    // Готовые константы пи и е отдаем сразу чтобы не набирать цифры вручную

    if code == "~pi" {
        vm.echo(&format!("{}", std::f64::consts::PI));
        return Some(Ok(Value::Number(std::f64::consts::PI)));
    }
    if code == "~e" {
        vm.echo(&format!("{}", std::f64::consts::E));
        return Some(Ok(Value::Number(std::f64::consts::E)));
    }

    // Среднее и медиану считаем одной командой чтобы обойтись без цикла

    if code.starts_with("~mean(") && code.ends_with(')') {
        let name = code[6..code.len() - 1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let n = list.len() as f64;
        if n == 0.0 { return Some(Err("Empty list".to_string())); }
        let sum: f64 = list.iter().filter_map(|v| v.as_number()).sum();
        let r = sum / n;
        vm.echo(format!("{}", Value::fmt_num(r)));
        return Some(Ok(Value::Number(r)));
    }

    if code.starts_with("~median(") && code.ends_with(')') {
        let name = code[8..code.len() - 1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let mut nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
        if nums.is_empty() { return Some(Err("Empty list".to_string())); }
        nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = nums.len() / 2;
        let r = if nums.len() % 2 == 0 { (nums[mid - 1] + nums[mid]) / 2.0 } else { nums[mid] };
        vm.echo(format!("{}", Value::fmt_num(r)));
        return Some(Ok(Value::Number(r)));
    }

    if code.starts_with("~variance(") && code.ends_with(')') {
        let name = code[10..code.len() - 1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
        if nums.len() < 2 { return Some(Err("Need at least 2 values".to_string())); }
        let n = nums.len() as f64;
        let sum: f64 = nums.iter().sum();
        let mean = sum / n;
        let variance: f64 = nums.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
        vm.echo(format!("{}", Value::fmt_num(variance)));
        return Some(Ok(Value::Number(variance)));
    }

    if code.starts_with("~stddev(") && code.ends_with(')') {
        let name = code[8..code.len() - 1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
        if nums.len() < 2 { return Some(Err("Need at least 2 values".to_string())); }
        let n = nums.len() as f64;
        let sum: f64 = nums.iter().sum();
        let mean = sum / n;
        let stddev = (nums.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
        vm.echo(format!("{}", Value::fmt_num(stddev)));
        return Some(Ok(Value::Number(stddev)));
    }

    if code.starts_with("~product(") && code.ends_with(')') {
        let name = code[9..code.len() - 1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let r: f64 = list.iter().filter_map(|v| v.as_number()).product();
        vm.echo(format!("{}", Value::fmt_num(r)));
        return Some(Ok(Value::Number(r)));
    }

    // Размещения и сочетания считаем тут чтобы не перебирать варианты вручную

    if code.starts_with("~perm(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 2 {
            let n = vm.eval_expr(&p[0]).ok()? as u64;
            let k = vm.eval_expr(&p[1]).ok()? as u64;
            if k > n { return Some(Err("k > n".to_string())); }
            let r = (n - k + 1..=n).product::<u64>() as f64;
            vm.echo(format!("{}", Value::fmt_num(r)));
            return Some(Ok(Value::Number(r)));
        }
    }

    if code.starts_with("~comb(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 2 {
            let n = vm.eval_expr(&p[0]).ok()? as u64;
            let k = vm.eval_expr(&p[1]).ok()? as u64;
            if k > n { return Some(Err("k > n".to_string())); }
            let k = k.min(n - k);
            let r = (1..=k).fold(1u64, |acc, i| acc * (n - k + i) / i) as f64;
            vm.echo(format!("{}", Value::fmt_num(r)));
            return Some(Ok(Value::Number(r)));
        }
    }

    // Расстояния меряем одной формулой чтобы убрать рутину из скрипта

    if code.starts_with("~distance(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 4 {
            let (x1, y1, x2, y2) = (vm.eval_expr(&p[0]).ok()?, vm.eval_expr(&p[1]).ok()?, vm.eval_expr(&p[2]).ok()?, vm.eval_expr(&p[3]).ok()?);
            let r = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
            vm.echo(format!("{}", Value::fmt_num(r)));
            return Some(Ok(Value::Number(r)));
        }
    }

    if code.starts_with("~dist3d(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 6 {
            let (x1, y1, z1, x2, y2, z2) = (vm.eval_expr(&p[0]).ok()?, vm.eval_expr(&p[1]).ok()?, vm.eval_expr(&p[2]).ok()?, vm.eval_expr(&p[3]).ok()?, vm.eval_expr(&p[4]).ok()?, vm.eval_expr(&p[5]).ok()?);
            let r = ((x2 - x1).powi(2) + (y2 - y1).powi(2) + (z2 - z1).powi(2)).sqrt();
            vm.echo(format!("{}", Value::fmt_num(r)));
            return Some(Ok(Value::Number(r)));
        }
    }

    if code.starts_with("~midpoint(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len() - 1], ',');
        if p.len() == 4 {
            let (x1, y1, x2, y2) = (vm.eval_expr(&p[0]).ok()?, vm.eval_expr(&p[1]).ok()?, vm.eval_expr(&p[2]).ok()?, vm.eval_expr(&p[3]).ok()?);
            let r = Value::List(vec![Value::Number((x1 + x2) / 2.0), Value::Number((y1 + y2) / 2.0)]);
            vm.echo(format!("{}", r.display()));
            return Some(Ok(r));
        }
    }

    // Проверки четности и знака отвечают да или нет чтобы убрать лишний код

    macro_rules! check1 {
        ($p:expr, $l:expr, $f:expr) => {
            if code.starts_with($p) && code.ends_with(')') {
                let n = match vm.eval_expr(&code[$l..code.len() - 1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
                let r = ($f)(n);
                vm.echo(format!("{}", r));
                return Some(Ok(Value::Bool(r)));
            }
        };
    }

    check1!("~is_even(", 9, |n: f64| n as i64 % 2 == 0);
    check1!("~is_odd(", 8, |n: f64| n as i64 % 2 != 0);
    check1!("~is_positive(", 13, |n: f64| n > 0.0);
    check1!("~is_negative(", 13, |n: f64| n < 0.0);
    check1!("~is_zero(", 9, |n: f64| n == 0.0);

    if code.starts_with("~wrap(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[6..code.len() - 1], ',');
        if p.len() == 3 {
            let (x, lo, hi) = (vm.eval_expr(&p[0]).ok()?, vm.eval_expr(&p[1]).ok()?, vm.eval_expr(&p[2]).ok()?);
            let range = hi - lo;
            if range <= 0.0 { return Some(Err("wrap: hi must be > lo".to_string())); }
            let r = ((x - lo) % range + range) % range + lo;
            vm.echo(format!("{}", Value::fmt_num(r)));
            return Some(Ok(Value::Number(r)));
        }
    }

    if code.starts_with("~smoothstep(") && code.ends_with(')') {
        let t = match vm.eval_expr(&code[12..code.len() - 1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
        let t = t.max(0.0).min(1.0);
        let r = t * t * (3.0 - 2.0 * t);
        vm.echo(format!("{}", Value::fmt_num(r)));
        return Some(Ok(Value::Number(r)));
    }

    if code.starts_with("~shuffle(") && code.ends_with(')') {
        let name = code[9..code.len() - 1].trim();
        let mut list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        list.shuffle(&mut rand::thread_rng());
        vm.set_var(name, Value::List(list.clone()));
        let r = Value::List(list);
        vm.echo(format!("{}", r.display()));
        return Some(Ok(r));
    }

    if code.starts_with("~choice(") && code.ends_with(')') {
        let name = code[8..code.len() - 1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        if list.is_empty() { return Some(Err("Empty list".to_string())); }
        let i = rand::thread_rng().gen_range(0..list.len());
        let r = list[i].clone();
        vm.echo(format!("{}", r.display()));
        return Some(Ok(r));
    }

    if code.starts_with("~sample(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let k = vm.eval_expr(&p[1]).ok()? as usize;
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            if k > list.len() { return Some(Err("k > list size".to_string())); }
            let mut indices: Vec<usize> = (0..list.len()).collect();
            indices.shuffle(&mut rand::thread_rng());
            let sampled: Vec<Value> = indices.iter().take(k).map(|&i| list[i].clone()).collect();
            let r = Value::List(sampled);
            vm.echo(format!("{}", r.display()));
            return Some(Ok(r));
        }
    }

    // Скалярное и векторное произведения прячем тут чтобы скрипт читался проще

    if code.starts_with("~dot(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len() - 1], ',');
        if p.len() == 2 {
            let a = match vm.get_var(p[0].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let b = match vm.get_var(p[1].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            if a.len() != b.len() { return Some(Err("Vectors must have same length".to_string())); }
            let r: f64 = a.iter().zip(b.iter()).filter_map(|(x, y)| Some(x.as_number()? * y.as_number()?)).sum();
            vm.echo(format!("{}", Value::fmt_num(r)));
            return Some(Ok(Value::Number(r)));
        }
    }

    if code.starts_with("~cross(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[7..code.len() - 1], ',');
        if p.len() == 2 {
            let a = match vm.get_var(p[0].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let b = match vm.get_var(p[1].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            if a.len() != 3 || b.len() != 3 { return Some(Err("Cross product needs 3D vectors".to_string())); }
            let v: Vec<f64> = a.iter().filter_map(|x| x.as_number()).collect();
            let w: Vec<f64> = b.iter().filter_map(|x| x.as_number()).collect();
            let r = Value::List(vec![
                Value::Number(v[1] * w[2] - v[2] * w[1]),
                Value::Number(v[2] * w[0] - v[0] * w[2]),
                Value::Number(v[0] * w[1] - v[1] * w[0]),
            ]);
            vm.echo(format!("{}", r.display()));
            return Some(Ok(r));
        }
    }

    if code.starts_with("~vnorm(") && code.ends_with(')') {
        let name = code[7..code.len() - 1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let mag: f64 = list.iter().filter_map(|x| x.as_number()).map(|x| x * x).sum::<f64>().sqrt();
        if mag == 0.0 { return Some(Err("Zero vector".to_string())); }
        let r: Vec<Value> = list.iter().filter_map(|x| Some(Value::Number(x.as_number()? / mag))).collect();
        let r = Value::List(r);
        vm.echo(format!("{}", r.display()));
        return Some(Ok(r));
    }

    if code.starts_with("~percentile(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let perc = vm.eval_expr(&p[1]).ok()? / 100.0;
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let mut nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
            if nums.is_empty() { return Some(Err("Empty list".to_string())); }
            nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let idx = (perc * (nums.len() - 1) as f64).round() as usize;
            let r = nums[idx.min(nums.len() - 1)];
            vm.echo(format!("{}", Value::fmt_num(r)));
            return Some(Ok(Value::Number(r)));
        }
    }

    if code.starts_with("~covariance(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len() - 1], ',');
        if p.len() == 2 {
            let a = match vm.get_var(p[0].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let b = match vm.get_var(p[1].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            if a.len() != b.len() || a.len() < 2 { return Some(Err("Need same length >= 2".to_string())); }
            let n = a.len() as f64;
            let x: Vec<f64> = a.iter().filter_map(|v| v.as_number()).collect();
            let y: Vec<f64> = b.iter().filter_map(|v| v.as_number()).collect();
            let mx = x.iter().sum::<f64>() / n;
            let my = y.iter().sum::<f64>() / n;
            let r = x.iter().zip(y.iter()).map(|(xi, yi)| (xi - mx) * (yi - my)).sum::<f64>() / (n - 1.0);
            vm.echo(format!("{}", Value::fmt_num(r)));
            return Some(Ok(Value::Number(r)));
        }
    }

    if code.starts_with("~zscore(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len() - 1], ',');
        if p.len() == 2 {
            let name = p[0].trim();
            let val = vm.eval_expr(&p[1]).ok()?;
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let nums: Vec<f64> = list.iter().filter_map(|v| v.as_number()).collect();
            if nums.len() < 2 { return Some(Err("Need at least 2 values".to_string())); }
            let n = nums.len() as f64;
            let mean = nums.iter().sum::<f64>() / n;
            let stddev = (nums.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
            let r = (val - mean) / stddev;
            vm.echo(format!("{}", Value::fmt_num(r)));
            return Some(Ok(Value::Number(r)));
        }
    }

    if code.starts_with("~entropy(") && code.ends_with(')') {
        let name = code[9..code.len() - 1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let total = list.len() as f64;
        if total == 0.0 { return Some(Err("Empty list".to_string())); }
        let mut freq: Vec<f64> = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for v in &list {
            let s = v.display();
            if let Some(pos) = seen.iter().position(|x| x == &s) { freq[pos] += 1.0; }
            else { seen.push(s); freq.push(1.0); }
        }
        let r: f64 = freq.iter().map(|&c| { let p = c / total; -p * p.log2() }).sum();
        vm.echo(format!("{}", Value::fmt_num(r)));
        return Some(Ok(Value::Number(r)));
    }

    // Плавные кривые тормозят к концу чтобы движение выглядело живо

    macro_rules! easing1 {
        ($p:expr, $l:expr, $f:expr) => {
            if code.starts_with($p) && code.ends_with(')') {
                let t = match vm.eval_expr(&code[$l..code.len() - 1]) { Ok(v) => v.max(0.0).min(1.0), Err(e) => return Some(Err(e)) };
                let r = ($f)(t);
                vm.echo(format!("{}", Value::fmt_num(r)));
                return Some(Ok(Value::Number(r)));
            }
        };
    }

    easing1!("~ease_quad_in(", 14, |t: f64| t * t);
    easing1!("~ease_quad_out(", 15, |t: f64| -(t * (t - 2.0)));
    easing1!("~ease_cubic_in(", 15, |t: f64| t * t * t);
    easing1!("~ease_cubic_out(", 16, |t: f64| (t - 1.0).powi(3) + 1.0);
    easing1!("~ease_sine_in(", 14, |t: f64| 1.0 - (t * std::f64::consts::PI / 2.0).cos());
    easing1!("~ease_sine_out(", 15, |t: f64| (t * std::f64::consts::PI / 2.0).sin());
    easing1!("~ease_bounce_out(", 17, |t: f64| {
        if t < 1.0 / 2.75 { 7.5625 * t * t }
        else if t < 2.0 / 2.75 { let t = t - 1.5 / 2.75; 7.5625 * t * t + 0.75 }
        else if t < 2.5 / 2.75 { let t = t - 2.25 / 2.75; 7.5625 * t * t + 0.9375 }
        else { let t = t - 2.625 / 2.75; 7.5625 * t * t + 0.984375 }
    });

    // Сигмоида и relu жмут любое число в понятный диапазон чтобы сети получали готовый вход
    m1!("~sigmoid(", 9, |x: f64| 1.0 / (1.0 + (-x).exp()));
    m1!("~relu(", 6, |x: f64| x.max(0.0));
    m1!("~softplus(", 10, |x: f64| (1.0 + x.exp()).ln());
    m1!("~logit(", 7, |p: f64| (p / (1.0 - p)).ln());

    // Колокол нормального распределения считаем тут чтобы не тащить формулы в скрипт
    m1!("~normal_pdf(", 11, |x: f64| (-x.powi(2) / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt());
    if code.starts_with("~normal_cdf(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() >= 2 {
            let x = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let a = x.abs();
            let cdf = 1.0 - (1.0 + 0.278393 * a + 0.230389 * a.powi(2) + 0.000972 * a.powi(3) + 0.078108 * a.powi(4)).powi(-4);
            let r = if x >= 0.0 { cdf } else { 1.0 - cdf };
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        } else { return None; }
    }

    if code.starts_with("~rand_normal(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[13..code.len()-1], ',');
        if p.len() >= 2 {
            let mean = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let std = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let u1 = rand::thread_rng().gen::<f64>(); let u2 = rand::thread_rng().gen::<f64>();
            let r = mean + std * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        } else { return None; }
    }

    // Вектора складываем целиком со списком сразу чтобы не идти по элементам
    if code.starts_with("~vec_add(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() == 2 {
            let a = match vm.get_var(p[0].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let b = match vm.get_var(p[1].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            if a.len() != b.len() { return Some(Err("Different lengths".to_string())); }
            let r = Value::List(a.iter().zip(b.iter()).map(|(x,y)| Value::Number(x.as_number().unwrap_or(0.0) + y.as_number().unwrap_or(0.0))).collect());
            vm.echo(format!("{}", r.display())); return Some(Ok(r));
        } else { return None; }
    }

    if code.starts_with("~vec_sub(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() == 2 {
            let a = match vm.get_var(p[0].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let b = match vm.get_var(p[1].trim()) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            if a.len() != b.len() { return Some(Err("Different lengths".to_string())); }
            let r = Value::List(a.iter().zip(b.iter()).map(|(x,y)| Value::Number(x.as_number().unwrap_or(0.0) - y.as_number().unwrap_or(0.0))).collect());
            vm.echo(format!("{}", r.display())); return Some(Ok(r));
        } else { return None; }
    }

    if code.starts_with("~vec_scale(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ','); if p.len() == 2 {
            let name = p[0].trim();
            let s = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
            let r = Value::List(list.iter().map(|v| Value::Number(v.as_number().unwrap_or(0.0) * s)).collect());
            vm.echo(format!("{}", r.display())); return Some(Ok(r));
        } else { return None; }
    }

    // Определитель и поворот маленьких матриц считаем тут чтобы не расписывать вручную
    if code.starts_with("~mat_det(") && code.ends_with(')') {
        let name = code[9..code.len()-1].trim();
        let m = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let rows: Vec<Vec<f64>> = m.iter().filter_map(|r| match r { Value::List(cells) => Some(cells.iter().filter_map(|c| c.as_number()).collect()), _ => None }).collect();
        if rows.len() == 2 && rows[0].len() == 2 { let d = rows[0][0]*rows[1][1] - rows[0][1]*rows[1][0]; vm.echo(Value::fmt_num(d)); return Some(Ok(Value::Number(d))); }
        if rows.len() == 3 && rows[0].len() == 3 { let d = rows[0][0]*(rows[1][1]*rows[2][2]-rows[1][2]*rows[2][1]) - rows[0][1]*(rows[1][0]*rows[2][2]-rows[1][2]*rows[2][0]) + rows[0][2]*(rows[1][0]*rows[2][1]-rows[1][1]*rows[2][0]); vm.echo(Value::fmt_num(d)); return Some(Ok(Value::Number(d))); }
        return Some(Err("Need 2x2 or 3x3 matrix".to_string()));
    }

    if code.starts_with("~mat_transpose(") && code.ends_with(')') {
        let name = code[15..code.len()-1].trim();
        let m = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let rows: Vec<Vec<f64>> = m.iter().filter_map(|r| match r { Value::List(cells) => Some(cells.iter().filter_map(|c| c.as_number()).collect()), _ => None }).collect();
        if rows.is_empty() { return Some(Err("Empty matrix".to_string())); }
        let cols = rows[0].len();
        let r: Vec<Value> = (0..cols).map(|c| Value::List(rows.iter().map(|row| Value::Number(row[c])).collect())).collect();
        let r = Value::List(r); vm.echo(format!("{}", r.display())); return Some(Ok(r));
    }

    // Разную мелочь держим рядом чтобы не искать по другим файлам
    if code.starts_with("~binomial_pmf(") && code.ends_with(')') {
        let parts = VM::split_top_level(&code[14..code.len()-1], ',');
        if parts.len() == 3 {
            let k = match vm.eval_expr(&parts[0]) { Ok(v) => v as u64, Err(e) => return Some(Err(e)) };
            let n = match vm.eval_expr(&parts[1]) { Ok(v) => v as u64, Err(e) => return Some(Err(e)) };
            let prob = match vm.eval_expr(&parts[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            if k > n { return Some(Err("k > n".to_string())); }
            let k2 = k.min(n - k); let comb = (1..=k2).fold(1u64, |acc, i| acc * (n - k2 + i) / i);
            let r = comb as f64 * prob.powi(k as i32) * (1.0 - prob).powi((n - k) as i32);
            vm.echo(format!("{}", Value::fmt_num(r))); return Some(Ok(Value::Number(r)));
        } else { return None; }
    }

    if code.starts_with("~map_range(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() == 5 {
            let x = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let il = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let ih = match vm.eval_expr(&p[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let ol = match vm.eval_expr(&p[3]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let oh = match vm.eval_expr(&p[4]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let r = ol + (x - il) * (oh - ol) / (ih - il);
            vm.echo(format!("{}", Value::fmt_num(r))); return Some(Ok(Value::Number(r)));
        } else { return None; }
    }

    // Перевод единиц длины даем тут чтобы не гуглить коэффициенты для метров дюймов и парсеков
    m1!("~radians(", 9, |d: f64| d * std::f64::consts::PI / 180.0);
    m1!("~degrees(", 9, |r: f64| r * 180.0 / std::f64::consts::PI);
    m1!("~millimeters(", 13, |m: f64| m * 1000.0);
    m1!("~centimeters(", 13, |m: f64| m * 100.0);
    m1!("~kilometers(", 12, |m: f64| m / 1000.0);
    m1!("~inches(", 8, |m: f64| m * 39.3701);
    m1!("~feet(", 6, |m: f64| m * 3.28084);
    m1!("~miles(", 7, |m: f64| m / 1609.344);
    m1!("~au(", 4, |m: f64| m / 149_597_870_700.0);
    m1!("~light_years(", 13, |m: f64| m / 9_460_730_472_580_800.0);
    m1!("~parsecs(", 9, |m: f64| m / 3.085677581e16);
    m1!("~nautical_miles(", 16, |m: f64| m / 1852.0);
    m1!("~furlongs(", 10, |m: f64| m / 201.168);
    m1!("~chains(", 8, |m: f64| m / 20.1168);
    m1!("~yards(", 7, |m: f64| m / 0.9144);
    m1!("~micrometers(", 13, |m: f64| m * 1_000_000.0);
    m1!("~nanometers(", 12, |m: f64| m * 1_000_000_000.0);
    m1!("~angstroms(", 11, |m: f64| m * 10_000_000_000.0);
    m1!("~twips(", 7, |m: f64| m * 56692.9);
    m1!("~picas(", 7, |m: f64| m * 236.22);

    macro_rules! bit2 {
        ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let p = VM::split_top_level(&code[$p.len()..code.len()-1], ','); if p.len() >= 2 { let a = match vm.eval_expr(&p[0]) { Ok(v) => v as i64, Err(e) => return Some(Err(e)) }; let b = match vm.eval_expr(&p[1]) { Ok(v) => v as i64, Err(e) => return Some(Err(e)) }; let r = ($f)(a, b) as f64; vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } }; };
    }
    bit2!("~bit_and(", 9, |a: i64, b: i64| a & b);
    bit2!("~bit_or(", 8, |a: i64, b: i64| a | b);
    bit2!("~bit_xor(", 9, |a: i64, b: i64| a ^ b);
    bit2!("~shl(", 5, |a: i64, b: i64| a << b);
    bit2!("~shr(", 5, |a: i64, b: i64| a >> b);

    // Дробные сравниваем через допуск потому что прямое сравнение ломается на округлении

    // Допуск гасит ошибки округления поэтому сравнение дает честный ответ
    if code.starts_with("~approx(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ',');
        if p.len() >= 2 {
            let a = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let b = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let eps = if p.len() >= 3 { match vm.eval_expr(&p[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 1e-9 };
            let r = (a - b).abs() <= eps; vm.echo(r); return Some(Ok(Value::Bool(r)));
        }
    }

    // Гаусс и пуассон нужны чтобы шум выглядел как настоящий а не как ровный мусор
    if code.starts_with("~random_gaussian(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len()-1], ',');
        if p.len() >= 1 {
            let mean = if p.len() >= 1 { vm.eval_expr(&p[0]).unwrap_or(0.0) } else { 0.0 };
            let std = if p.len() >= 2 { vm.eval_expr(&p[1]).unwrap_or(1.0) } else { 1.0 };
            let u1 = rand::thread_rng().gen::<f64>(); let u2 = rand::thread_rng().gen::<f64>();
            let r = mean + std * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
    }
    if code.starts_with("~random_poisson(") && code.ends_with(')') {
        let lambda = vm.eval_expr(&code[16..code.len()-1]).unwrap_or(1.0);
        let l = (-lambda).exp(); let mut k = 0u64; let mut p = 1.0f64;
        loop { k += 1; p *= rand::thread_rng().gen::<f64>(); if p <= l { break; } if k > 1000000 { break; } }
        let r = (k - 1) as f64; vm.echo(r); return Some(Ok(Value::Number(r)));
    }
    if code.starts_with("~random_exponential(") && code.ends_with(')') {
        let rate = vm.eval_expr(&code[21..code.len()-1]).unwrap_or(1.0);
        let u = rand::thread_rng().gen::<f64>();
        let r = -u.ln() / rate; vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }

    // Длину и норму приводим к единице чтобы направления сравнивались честно
    if code.starts_with("~vector_len(") && code.ends_with(')') {
        let name = code[12..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let r: f64 = list.iter().filter_map(|v| v.as_number()).map(|x| x*x).sum::<f64>().sqrt();
        vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
    }
    if code.starts_with("~vector_normalize(") && code.ends_with(')') {
        let name = code[18..code.len()-1].trim();
        let list = match vm.get_var(name) { Some(Value::List(l)) => l, _ => return Some(Err("List not found".to_string())) };
        let mag: f64 = list.iter().filter_map(|v| v.as_number()).map(|x| x*x).sum::<f64>().sqrt();
        if mag == 0.0 { return Some(Err("Zero vector".to_string())); }
        let r: Vec<Value> = list.iter().filter_map(|v| v.as_number()).map(|x| Value::Number(x/mag)).collect();
        let r = Value::List(r); vm.echo(format!("{}", r.display())); return Some(Ok(r));
    }

    // Размеры матриц проверяем тут чтобы ловить ошибку рано а не в глубине счета
    fn matrix_from_list(list: &[Value]) -> Vec<Vec<f64>> {
        list.iter().filter_map(|r| match r { Value::List(cells) => Some(cells.iter().filter_map(|c| c.as_number()).collect()), _ => None }).collect()
    }
    fn matrix_to_value(m: Vec<Vec<f64>>) -> Value {
        Value::List(m.into_iter().map(|row| Value::List(row.into_iter().map(Value::Number).collect())).collect())
    }

    if code.starts_with("~matrix_add(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() == 2 {
            let a = matrix_from_list(&vm.get_var(p[0].trim()).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default());
            let b = matrix_from_list(&vm.get_var(p[1].trim()).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default());
            if a.len() != b.len() || (a.first().map(|r| r.len()).unwrap_or(0) != b.first().map(|r| r.len()).unwrap_or(0)) { return Some(Err("Matrix size mismatch".to_string())); }
            let r: Vec<Vec<f64>> = a.iter().zip(b.iter()).map(|(x, y)| x.iter().zip(y.iter()).map(|(m, n)| m + n).collect()).collect();
            let r = matrix_to_value(r); vm.echo(format!("{}", r.display())); return Some(Ok(r));
        }
    }
    if code.starts_with("~matrix_sub(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() == 2 {
            let a = matrix_from_list(&vm.get_var(p[0].trim()).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default());
            let b = matrix_from_list(&vm.get_var(p[1].trim()).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default());
            if a.len() != b.len() || (a.first().map(|r| r.len()).unwrap_or(0) != b.first().map(|r| r.len()).unwrap_or(0)) { return Some(Err("Matrix size mismatch".to_string())); }
            let r: Vec<Vec<f64>> = a.iter().zip(b.iter()).map(|(x, y)| x.iter().zip(y.iter()).map(|(m, n)| m - n).collect()).collect();
            let r = matrix_to_value(r); vm.echo(format!("{}", r.display())); return Some(Ok(r));
        }
    }
    if code.starts_with("~matrix_mul(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() == 2 {
            let a = matrix_from_list(&vm.get_var(p[0].trim()).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default());
            let b = matrix_from_list(&vm.get_var(p[1].trim()).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default());
            if a.is_empty() || b.is_empty() || a[0].len() != b.len() { return Some(Err("Matrix size mismatch".to_string())); }
            let rows = a.len(); let cols = b[0].len(); let n = a[0].len();
            let mut r = vec![vec![0.0; cols]; rows];
            for i in 0..rows { for k in 0..n { let ak = a[i][k]; for j in 0..cols { r[i][j] += ak * b[k][j]; } } }
            let r = matrix_to_value(r); vm.echo(format!("{}", r.display())); return Some(Ok(r));
        }
    }
    if code.starts_with("~matrix_identity(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[17..code.len()-1]).unwrap_or(3.0) as usize;
        let mut r = vec![vec![0.0; n]; n];
        for i in 0..n { r[i][i] = 1.0; }
        let r = matrix_to_value(r); vm.echo(format!("{}", r.display())); return Some(Ok(r));
    }
    if code.starts_with("~matrix_scale(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len()-1], ',');
        if p.len() == 2 {
            let a = matrix_from_list(&vm.get_var(p[0].trim()).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default());
            let s = vm.eval_expr(&p[1]).unwrap_or(1.0);
            let r: Vec<Vec<f64>> = a.iter().map(|row| row.iter().map(|v| v * s).collect()).collect();
            let r = matrix_to_value(r); vm.echo(format!("{}", r.display())); return Some(Ok(r));
        }
    }
    if code.starts_with("~matrix_inv(") && code.ends_with(')') {
        let name = code[12..code.len()-1].trim();
        let m = matrix_from_list(&vm.get_var(name).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default());
        if m.len() == 2 && m[0].len() == 2 {
            let (a, b, c, d) = (m[0][0], m[0][1], m[1][0], m[1][1]);
            let det = a*d - b*c;
            if det.abs() < 1e-12 { return Some(Err("Singular matrix".to_string())); }
            let r = matrix_to_value(vec![vec![d/det, -b/det], vec![-c/det, a/det]]);
            vm.echo(format!("{}", r.display())); return Some(Ok(r));
        }
        if m.len() == 3 && m[0].len() == 3 {
            let (a, b, c) = (m[0][0], m[0][1], m[0][2]);
            let (d, e, f) = (m[1][0], m[1][1], m[1][2]);
            let (g, h, i) = (m[2][0], m[2][1], m[2][2]);
            let det = a*(e*i - f*h) - b*(d*i - f*g) + c*(d*h - e*g);
            if det.abs() < 1e-12 { return Some(Err("Singular matrix".to_string())); }
            let r = matrix_to_value(vec![
                vec![(e*i - f*h)/det, (c*h - b*i)/det, (b*f - c*e)/det],
                vec![(f*g - d*i)/det, (a*i - c*g)/det, (c*d - a*f)/det],
                vec![(d*h - e*g)/det, (b*g - a*h)/det, (a*e - b*d)/det],
            ]);
            vm.echo(format!("{}", r.display())); return Some(Ok(r));
        }
        return Some(Err("Need 2x2 or 3x3 matrix".to_string()));
    }

    // Сигнал дополняем до степени двойки чтобы разложить его на частоты преобразованием Фурье
    if code.starts_with("~fft(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[5..code.len()-1], ',');
        if p.len() >= 1 {
            let name = p[0].trim();
            let real: Vec<f64> = vm.get_var(name).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default().into_iter().filter_map(|v| v.as_number()).collect();
            let imag: Vec<f64> = if p.len() >= 2 {
                vm.get_var(p[1].trim()).and_then(|v| match v { Value::List(l) => Some(l), _ => None }).unwrap_or_default().into_iter().filter_map(|v| v.as_number()).collect()
            } else { vec![0.0; real.len()] };
            let n = real.len().max(1);
            let mut re = real; let mut im = imag.clone();
            re.resize(n, 0.0); im.resize(n, 0.0);
            let n2 = n.next_power_of_two();
            re.resize(n2, 0.0); im.resize(n2, 0.0);
            // Тасуем и складываем бабочкой чтобы считать заметно быстрее чем прямым перебором
            let m = n2;
            let mut j = 0usize;
            for i in 1..m {
                let mut bit = m >> 1;
                while j & bit != 0 { j ^= bit; bit >>= 1; }
                j ^= bit;
                if i < j { re.swap(i, j); im.swap(i, j); }
            }
            let mut len = 2usize;
            while len <= m {
                let ang = -2.0 * std::f64::consts::PI / len as f64;
                let (wr, wi) = (ang.cos(), ang.sin());
                let mut half = len / 2;
                let mut i = 0usize;
                while i < m {
                    let mut cur_r = 1.0f64; let mut cur_i = 0.0f64;
                    let mut k = 0usize;
                    while k < half {
                        let l = i + k; let r = i + k + half;
                        let tr = cur_r * re[r] - cur_i * im[r];
                        let ti = cur_r * im[r] + cur_i * re[r];
                        re[r] = re[l] - tr; im[r] = im[l] - ti;
                        re[l] += tr; im[l] += ti;
                        let nr = cur_r * wr - cur_i * wi;
                        cur_i = cur_r * wi + cur_i * wr;
                        cur_r = nr;
                        k += 1;
                    }
                    i += len;
                }
                len <<= 1;
            }
            let out: Vec<Value> = re.iter().zip(im.iter()).map(|(r, i)| Value::List(vec![Value::Number(*r), Value::Number(*i)])).collect();
            let r = Value::List(out); vm.echo(format!("{}", r.display())); return Some(Ok(r));
        }
    }

    if code.starts_with("~bit_not(") && code.ends_with(')') {
        let n = match vm.eval_expr(&code[9..code.len()-1]) { Ok(v) => v as i64, Err(e) => return Some(Err(e)) };
        let r = (!n) as f64; vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~bit_rol(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() >= 2 {
            let a = match vm.eval_expr(&p[0]) { Ok(v) => v as u64, Err(e) => return Some(Err(e)) };
            let b = match vm.eval_expr(&p[1]) { Ok(v) => v as u32, Err(e) => return Some(Err(e)) };
            let r = a.rotate_left(b) as f64; vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
        } else { None }
    } else if code.starts_with("~bit_ror(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() >= 2 {
            let a = match vm.eval_expr(&p[0]) { Ok(v) => v as u64, Err(e) => return Some(Err(e)) };
            let b = match vm.eval_expr(&p[1]) { Ok(v) => v as u32, Err(e) => return Some(Err(e)) };
            let r = a.rotate_right(b) as f64; vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
        } else { None }
    } else {
        None
    }
}

fn is_prime(n: i64) -> bool {
    if n < 2 { return false; }
    if n % 2 == 0 { return n == 2; }
    let mut k = 3i64;
    while k * k <= n { if n % k == 0 { return false; } k += 2 }
    true
}

fn gamma_l(x: f64) -> f64 {
    let g = 7.0;
    let c = [0.99999999999980993, 676.5203681218851, -1259.1392167224028, 771.32342877765313,
             -176.61502916214059, 12.507343278686905, -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7];
    if x < 0.5 {
        std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma_l(1.0 - x))
    } else {
        let x = x - 1.0;
        let mut a = c[0];
        let mut t = x + g + 0.5;
        for i in 1..c.len() { a += c[i] / (x + i as f64); }
        (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * a
    }
}

fn erf_approx(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * (-x * x).exp();
    sign * y
}

fn erf_inv_approx(x: f64) -> f64 {
    if x.abs() >= 1.0 { return f64::NAN; }
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let a = 0.147;
    let ln1mx2 = (1.0 - x * x).ln();
    let part1 = 2.0 / (std::f64::consts::PI * a) + ln1mx2 / 2.0;
    sign * (part1.powi(2) - ln1mx2 / a).sqrt() - sign * part1
}

fn bessel_j0(x: f64) -> f64 {
    let ax = x.abs();
    if ax < 8.0 {
        let y = x * x;
        1.0 + y * (-2.2499997 + y * (1.2656208 + y * (-0.3163866 + y * (0.0444479 + y * (-0.0039444 + y * 0.00021)))))
    } else {
        let t = 8.0 / ax; let y = t * t;
        let f = 0.79788456 + y * (-0.00000077 + y * (-0.00552740 + y * (-0.00009512 + y * 0.00137237)));
        let t2 = ax - std::f64::consts::FRAC_PI_4 + y * (-0.04166397 + y * (-0.00003954 + y * 0.00262573));
        f * t2.cos() / ax.sqrt()
    }
}

fn bessel_j1(x: f64) -> f64 {
    let ax = x.abs();
    let r;
    if ax < 8.0 {
        let y = x * x;
        r = x * (0.5 + y * (-0.56249985 + y * (0.21093573 + y * (-0.03954289 + y * (0.00443319 + y * (-0.00031761 + y * 0.00001109))))));
    } else {
        let t = 8.0 / ax; let y = t * t;
        let f = 0.79788456 + y * (0.00000156 + y * (0.01659667 + y * (0.00017105 + y * (-0.00249511))));
        let t2 = ax - 2.35619449 + y * (0.12499612 + y * (0.00005650 + y * (-0.00637879)));
        r = f * t2.cos() / ax.sqrt();
    }
    if x < 0.0 { -r } else { r }
}

fn bessel_y0(x: f64) -> f64 {
    if x <= 0.0 { return f64::NEG_INFINITY; }
    let j0 = bessel_j0(x);
    (2.0 / std::f64::consts::PI) * (j0 * x.ln() + 0.36746691 + 0.60559366 * (x - 0.5).cos().ln())
}

fn bessel_y1(x: f64) -> f64 {
    if x <= 0.0 { return f64::NEG_INFINITY; }
    let j1 = bessel_j1(x);
    (2.0 / std::f64::consts::PI) * (j1 * x.ln() - 1.0 / x)
}