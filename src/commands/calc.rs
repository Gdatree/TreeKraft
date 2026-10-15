use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! c1 {
        ($p:expr, $l:expr, $f:expr) => { if code.starts_with($p) && code.ends_with(')') { let n = match vm.eval_expr(&code[$p.len()..code.len()-1]) { Ok(v) => v, Err(e) => return Some(Err(e)) }; let r = ($f)(n); vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r))); } };
    }

    // Аргумент здесь игнорируем и всегда отдаем одну константу а сам вызов с числом сохраняем чтобы форма команды оставалась единой для всех функций
    c1!("~speed_of_light(", 15, |_: f64| 299792458.0);
    c1!("~planck(", 7, |_: f64| 6.62607015e-34);
    c1!("~gravitational(", 14, |_: f64| 6.67430e-11);
    c1!("~boltzmann(", 10, |_: f64| 1.380649e-23);
    c1!("~avogadro(", 9, |_: f64| 6.02214076e23);
    c1!("~gas_constant(", 13, |_: f64| 8.314462618);
    c1!("~stefan_boltzmann(", 17, |_: f64| 5.670374419e-8);
    c1!("~elementary_charge(", 18, |_: f64| 1.602176634e-19);
    c1!("~electron_mass(", 14, |_: f64| 9.1093837015e-31);
    c1!("~proton_mass(", 12, |_: f64| 1.67262192369e-27);
    c1!("~neutron_mass(", 13, |_: f64| 1.67492749804e-27);
    c1!("~fine_structure(", 15, |_: f64| 7.2973525693e-3);
    c1!("~rydberg(", 8, |_: f64| 10973731.56816);
    c1!("~bohr_radius(", 12, |_: f64| 5.29177210903e-11);
    c1!("~hartree(", 8, |_: f64| 4.3597447222071e-18);
    c1!("~amu(", 4, |_: f64| 1.66053906660e-27);
    c1!("~magnetic_flux(", 14, |_: f64| 2.067833848e-15);
    c1!("~conductance(", 12, |_: f64| 7.748091729e-5);
    c1!("~vacuum_perm(", 12, |_: f64| 8.8541878128e-12);
    c1!("~vacuum_perm_mag(", 16, |_: f64| 1.25663706212e-6);

    // Константы вроде скорости света зашиты внутрь чтобы формулы считались по одному введенному числу и не приходилось спрашивать лишнее
    c1!("~kinetic_energy(", 15, |v: f64| 0.5 * v * v);
    c1!("~relativistic_gamma(", 19, |v: f64| 1.0 / (1.0 - (v/299792458.0).powi(2)).sqrt());
    c1!("~doppler_shift(", 14, |v: f64| ((1.0 + v/299792458.0) / (1.0 - v/299792458.0)).sqrt() - 1.0);
    c1!("~wien_displacement(", 18, |t: f64| 2.897771955e-3 / t);
    c1!("~blackbody_peak(", 15, |t: f64| 5.878925757e10 / t);
    c1!("~thermal_energy(", 15, |t: f64| 1.380649e-23 * t);
    c1!("~de_broglie(", 11, |p: f64| 6.62607015e-34 / p);
    c1!("~compton_wavelength(", 19, |_: f64| 2.42631023867e-12);
    c1!("~schwarzschild(", 14, |m: f64| 2.0 * 6.67430e-11 * m / 299792458.0_f64.powi(2));
    c1!("~mach_to_ms(", 11, |m: f64| m * 343.0);
    c1!("~ms_to_mach(", 11, |ms: f64| ms / 343.0);
    c1!("~horsepower_to_w(", 16, |hp: f64| hp * 745.7);
    c1!("~w_to_horsepower(", 16, |w: f64| w / 745.7);

    // Площадь и объем считаем сразу по радиусу чтобы не выводить формулы вручную в скрипте
    c1!("~circle_perimeter(", 17, |r: f64| 2.0 * std::f64::consts::PI * r);
    c1!("~sphere_surface(", 15, |r: f64| 4.0 * std::f64::consts::PI * r * r);
    c1!("~sphere_volume_fn(", 17, |r: f64| 4.0/3.0 * std::f64::consts::PI * r.powi(3));
    c1!("~cylinder_volume_fn(", 19, |r: f64| std::f64::consts::PI * r * r * r);
    c1!("~cone_volume(", 12, |r: f64| std::f64::consts::PI * r * r * r / 3.0);

    // Значение здесь всегда одно и то же а аргумент сохраняем только чтобы форма вызова оставалась единой
    c1!("~golden_ratio(", 13, |_: f64| 1.618033988749894);
    c1!("~silver_ratio(", 13, |_: f64| 2.414213562373095);
    c1!("~euler_mascheroni(", 17, |_: f64| 0.577215664901532);
    c1!("~apery(", 6, |_: f64| 1.202056903159594);
    c1!("~twin_prime(", 11, |_: f64| 0.660161815846869);

    // Суммы делители и спецфункции считаем внутри чтобы в скрипте не приходилось писать циклы
    c1!("~sum_of_squares(", 15, |n: f64| { let n = n as i64; (n*(n+1)*(2*n+1)/6) as f64 });
    c1!("~sum_of_cubes(", 13, |n: f64| { let n = n as i64; let t = n*(n+1)/2; (t*t) as f64 });
    c1!("~triangular(", 11, |n: f64| { let n = n as i64; (n*(n+1)/2) as f64 });
    c1!("~tetrahedral(", 12, |n: f64| { let n = n as i64; (n*(n+1)*(n+2)/6) as f64 });
    c1!("~penta_num(", 10, |n: f64| { let n = n as i64; (n*(3*n-1)/2) as f64 });
    c1!("~hex_num(", 8, |n: f64| { let n = n as i64; (n*(2*n-1)) as f64 });
    c1!("~pronic(", 7, |n: f64| { let n = n as i64; (n*(n+1)) as f64 });
    c1!("~prime_count(", 12, |n: f64| count_primes(n as u64) as f64);
    c1!("~divisor_count(", 14, |n: f64| divisors(n as u64).len() as f64);
    c1!("~sigma(", 6, |n: f64| sigma(n as u64) as f64);
    c1!("~aliquot_sum(", 12, |n: f64| (sigma(n as u64) - n as u64) as f64);
    c1!("~is_abundant(", 12, |n: f64| { let n = n as u64; if sigma(n) > 2*n { 1.0 } else { 0.0 } });
    c1!("~is_deficient(", 13, |n: f64| { let n = n as u64; if sigma(n) < 2*n { 1.0 } else { 0.0 } });
    c1!("~is_perfect(", 11, |n: f64| { let n = n as u64; if sigma(n) == 2*n { 1.0 } else { 0.0 } });
    c1!("~catalan(", 8, |n: f64| catalan(n as u64) as f64);
    c1!("~collatz_steps(", 14, |n: f64| collatz_steps(n as u64) as f64);
    c1!("~digit_product(", 14, |n: f64| { let mut d = n.abs() as u64; let mut p = 1u64; if d == 0 { 0.0 } else { while d > 0 { p *= d % 10; d /= 10; } p as f64 } });
    c1!("~mult_persist(", 13, |n: f64| mult_persist(n as u64) as f64);
    c1!("~add_persist(", 12, |n: f64| add_persist(n as u64) as f64);
    c1!("~rev_num(", 8, |n: f64| { let mut d = n.abs() as i64; let mut r = 0i64; while d > 0 { r = r*10 + d % 10; d /= 10; } r as f64 });
    c1!("~is_palindrome_num(", 18, |n: f64| { let mut d = n.abs() as i64; let mut r = 0i64; let o = d; while d > 0 { r = r*10 + d % 10; d /= 10; } if o == r { 1.0 } else { 0.0 } });
    c1!("~binary_weight(", 14, |n: f64| (n as u64).count_ones() as f64);
    c1!("~trailing_zeros(", 15, |n: f64| { let mut z = 0u64; let mut p = 5u64; let mut n = n as u64; while p <= n { z += n / p; p *= 5; } z as f64 });
    c1!("~harmonic(", 9, |n: f64| { let mut h = 0.0; for k in 1..=(n as u64) { h += 1.0 / k as f64; } h });
    c1!("~bell_num(", 9, |n: f64| bell(n as usize) as f64);
    c1!("~derangements(", 13, |n: f64| derangements(n as u64) as f64);
    c1!("~moebius(", 8, |n: f64| moebius(n as u64) as f64);
    c1!("~mertens(", 8, |n: f64| { let mut s = 0i64; for k in 1..=(n as u64) { s += moebius(k); } s as f64 });
    c1!("~liouville(", 10, |n: f64| { let mut x = n as u64; let mut omega = 0i64; let mut p = 2u64; while p*p <= x { while x % p == 0 { x /= p; omega += 1; } p += 1; } if x > 1 { omega += 1; } if omega % 2 == 0 { 1.0 } else { -1.0 } });
    c1!("~superfactorial(", 15, |n: f64| { let mut r = 1u64; let mut f = 1u64; for k in 1..=(n as u64).min(15) { f *= k; r *= f; } r as f64 });

    None
}

fn count_primes(n: u64) -> u64 {
    if n < 2 { return 0; }
    let mut sieve = vec![true; (n + 1) as usize];
    sieve[0] = false; sieve[1] = false;
    let mut p = 2u64;
    while p * p <= n {
        if sieve[p as usize] {
            let mut m = p * p;
            while m <= n { sieve[m as usize] = false; m += p; }
        }
        p += 1;
    }
    sieve.iter().filter(|&&b| b).count() as u64
}

fn divisors(n: u64) -> Vec<u64> {
    let mut d = Vec::new();
    let mut i = 1u64;
    while i * i <= n {
        if n % i == 0 {
            d.push(i);
            if i != n / i { d.push(n / i); }
        }
        i += 1;
    }
    d
}

fn sigma(n: u64) -> u64 {
    if n == 0 { return 0; }
    divisors(n).iter().sum()
}

fn catalan(n: u64) -> u64 {
    let mut num = 1u64; let mut den = 1u64;
    for k in 2..=n {
        num *= n + k; den *= k;
    }
    num / den
}

fn collatz_steps(n: u64) -> u64 {
    let mut x = n; let mut s = 0u64;
    while x != 1 {
        x = if x % 2 == 0 { x / 2 } else { x * 3 + 1 };
        s += 1;
    }
    s
}

fn mult_persist(n: u64) -> u64 {
    let mut x = n; let mut s = 0u64;
    while x >= 10 {
        let mut p = 1u64;
        while x > 0 { p *= x % 10; x /= 10; }
        x = p; s += 1;
    }
    s
}

fn add_persist(n: u64) -> u64 {
    let mut x = n; let mut s = 0u64;
    while x >= 10 {
        let mut t = 0u64;
        while x > 0 { t += x % 10; x /= 10; }
        x = t; s += 1;
    }
    s
}

fn bell(n: usize) -> u64 {
    let mut prev = vec![1u64];
    for k in 1..n {
        let mut cur = vec![prev[prev.len() - 1]];
        for j in 1..=k { let prev_v = if j - 1 < prev.len() { prev[j - 1] } else { 0 }; cur.push(cur[cur.len() - 1] + prev_v); }
        prev = cur;
    }
    prev[prev.len() - 1]
}

fn derangements(n: u64) -> u64 {
    let mut a = 1u64; let mut b = 0u64; // Стартуем с базы для нуля и единицы чтобы цикл рекуррентности посчитал беспорядки верно
    for k in 2..=n {
        let t = (k - 1) * (a + b);
        a = b; b = t;
    }
    if n == 0 { 1 } else { b }
}

fn moebius(n: u64) -> i64 {
    if n == 1 { return 1; }
    let mut x = n; let mut count = 0i64;
    let mut p = 2u64;
    while p * p <= x {
        if x % p == 0 {
            let mut k = 0;
            while x % p == 0 { x /= p; k += 1; }
            if k > 1 { return 0; }
            count += 1;
        }
        p += 1;
    }
    if x > 1 { count += 1; }
    if count % 2 == 0 { 1 } else { -1 }
}
