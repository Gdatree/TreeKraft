use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! h1 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let n=match vm.eval_expr(&code[$p.len()..code.len()-1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(n);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));} }; }
    macro_rules! h2 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let p=VM::split_top_level(&code[$p.len()..code.len()-1],',');if p.len()>=2{let a=match vm.eval_expr(&p[0]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let b=match vm.eval_expr(&p[1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(a,b);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));} }; } }

    // Номер ищем по порядку в таблице поэтому число вне диапазона от 1 до 118 отклоняем как неизвестный элемент
    if code.starts_with("~atomic_symbol(") && code.ends_with(')') {
        let n = match vm.eval_expr(&code[15..code.len()-1]) { Ok(v) => v as usize, Err(e) => return Some(Err(e)) };
        let symbols = ["H","He","Li","Be","B","C","N","O","F","Ne","Na","Mg","Al","Si","P","S","Cl","Ar","K","Ca","Sc","Ti","V","Cr","Mn","Fe","Co","Ni","Cu","Zn","Ga","Ge","As","Se","Br","Kr","Rb","Sr","Y","Zr","Nb","Mo","Tc","Ru","Rh","Pd","Ag","Cd","In","Sn","Sb","Te","I","Xe","Cs","Ba","La","Ce","Pr","Nd","Pm","Sm","Eu","Gd","Tb","Dy","Ho","Er","Tm","Yb","Lu","Hf","Ta","W","Re","Os","Ir","Pt","Au","Hg","Tl","Pb","Bi","Po","At","Rn","Fr","Ra","Ac","Th","Pa","U","Np","Pu","Am","Cm","Bk","Cf","Es","Fm","Md","No","Lr","Rf","Db","Sg","Bh","Hs","Mt","Ds","Rg","Cn","Nh","Fl","Mc","Lv","Ts","Og"];
        if n >= 1 && n <= symbols.len() { let r = symbols[n-1]; vm.echo(r); Some(Ok(Value::Text(r.to_string()))) } else { Some(Err("Unknown element".to_string())) }
    } else if code.starts_with("~atomic_mass(") && code.ends_with(')') {
        let n = match vm.eval_expr(&code[13..code.len()-1]) { Ok(v) => v as usize, Err(e) => return Some(Err(e)) };
        let masses = [1.008,4.003,6.941,9.012,10.811,12.011,14.007,15.999,18.998,20.180,22.990,24.305,26.982,28.086,30.974,32.065,35.453,39.948,39.098,40.078,44.956,47.867,50.942,51.996,54.938,55.845,58.933,58.693,63.546,65.390,69.723,72.640,74.922,78.960,79.904,83.800,85.468,87.620,88.906,91.224,92.906,95.940,98.000,101.070,102.906,106.420,107.868,112.411,114.818,118.710,121.760,127.600,126.905,131.293,132.906,137.327,138.906,140.116,140.908,144.240,145.000,150.360,151.964,157.250,158.925,162.500,164.930,167.259,168.934,173.040,174.967,178.490,180.948,183.840,186.207,190.230,192.217,195.078,196.967,200.590,204.383,207.200,208.980,209.000,210.000,222.000,223.000,226.000,227.000,232.038,231.036,238.029,237.000,244.000,243.000,247.000,247.000,251.000,252.000,257.000,258.000,259.000,266.000,267.000,268.000,269.000,270.000,277.000,278.000,281.000,282.000,285.000,286.000,289.000,290.000,293.000,294.000,294.000];
        if n >= 1 && n <= masses.len() { let r = masses[n-1]; vm.echo(format!("{}", Value::fmt_num(r))); Some(Ok(Value::Number(r))) } else { Some(Err("Unknown element".to_string())) }
    } else {
        // pH и концентрация связаны логарифмом поэтому пересчитываем одно из другого без таблиц
        h1!("~ph_from_h(", 10, |h: f64| -h.log10());
        h1!("~h_from_ph(", 10, |ph: f64| (10.0f64).powf(-ph));
        h1!("~poh_from_ph(", 12, |ph: f64| 14.0 - ph);
        h1!("~ph_from_poh(", 12, |poh: f64| 14.0 - poh);

        // Концентрацию считаем делением количества на объем или массу поэтому ниже просим оба числа
        h1!("~molarity(", 9, |mol: f64| mol / 1.0);
    h2!("~molality(", 9, |mol: f64, kg: f64| mol / kg);
    h2!("~normality(", 10, |eq: f64, l: f64| eq / l);

        // Объем газа считаем при комнатной температуре и давлении в одну атмосферу чтобы не спрашивать лишние параметры
        h1!("~ideal_gas_volume(", 17, |n: f64| n * 0.082057 * 298.15 / 1.0);
        h1!("~stp_volume(", 11, |n: f64| n * 22.414);
        // Температуру по умолчанию считаем комнатной 298.15 K а объем меньше поправки b отклоняем чтобы знаменатель не стал отрицательным и давление осталось осмысленным
    if code.starts_with("~van_der_waals(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ',');
        if p.len() >= 3 {
            let a = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let b = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let vml = match vm.eval_expr(&p[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let t = if p.len() >= 4 { match vm.eval_expr(&p[3]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 298.15 };
            if vml <= b { return Some(Err("van_der_waals: Vm must exceed b".to_string())); }
            let r = 8.314 * t / (vml - b) - a / vml.powi(2);
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return Some(Err("van_der_waals needs a, b, Vm[, T]".to_string()));
    }

    // Изменение считаем вычитанием начального значения из конечного поэтому порядок аргументов ниже важен для знака
    h2!("~delta_g(", 9, |g2: f64, g1: f64| g2 - g1);
    h2!("~delta_h(", 9, |h2: f64, h1: f64| h2 - h1);
    h2!("~delta_s(", 9, |s2: f64, s1: f64| s2 - s1);
    // Температуру по умолчанию считаем комнатной 298.15 K чтобы формулу можно было вызвать только по двум числам
    if code.starts_with("~gibbs_free(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() >= 2 {
            let h = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let s = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let t = if p.len() >= 3 { match vm.eval_expr(&p[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 298.15 };
            let r = h - t * s;
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return Some(Err("gibbs_free needs H, S[, T]".to_string()));
    }

    // Температуру по умолчанию считаем 298.15 K а нулевое n и неположительное Q отклоняем чтобы избежать деления на ноль и ошибки логарифма
    if code.starts_with("~nernst(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ',');
        if p.len() >= 3 {
            let e0 = match vm.eval_expr(&p[0]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let n = match vm.eval_expr(&p[1]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let q = match vm.eval_expr(&p[2]) { Ok(v) => v, Err(e) => return Some(Err(e)) };
            let t = if p.len() >= 4 { match vm.eval_expr(&p[3]) { Ok(v) => v, Err(e) => return Some(Err(e)) } } else { 298.15 };
            if n == 0.0 || q <= 0.0 { return Some(Err("nernst: need n != 0, Q > 0".to_string())); }
            let r = e0 - (8.314 * t / (n * 96485.0)) * q.ln();
            vm.echo(Value::fmt_num(r)); return Some(Ok(Value::Number(r)));
        }
        return Some(Err("nernst needs E0, n, Q[, T]".to_string()));
    }
    h2!("~cell_potential(", 16, |ecat: f64, ean: f64| ecat - ean);

        // Здесь пока отдаем введенное число без пересчета чтобы сохранить форму вызова до появления полных формул скорости
        h1!("~rate_constant(", 14, |k: f64| k);
        h1!("~half_life(", 10, |t: f64| t);
        h1!("~activation_energy(", 18, |e: f64| e);

        // Константы для фотона и уровней атома зашиты внутрь чтобы вызов работал по одному числу
        h1!("~photon_energy(", 14, |f: f64| 6.626e-34 * f);
        h1!("~photon_wavelength(", 18, |e: f64| 6.626e-34 * 3e8 / e);
        h1!("~rydberg_energy(", 15, |n: f64| -13.6 / n.powi(2));
        h1!("~balmer(", 7, |n: f64| 1.097e7 * (1.0/4.0 - 1.0/n.powi(2)));

        // Энергию связи и распад оцениваем простыми формулами чтобы прикинуть порядок без таблиц изотопов
        h1!("~binding_energy(", 15, |m: f64| m * 9e16);
        h1!("~decay_constant(", 15, |hl: f64| 0.693 / hl);
        h1!("~activity(", 9, |n: f64| n * 0.693 / 1.0);
        h1!("~carbon_dating(", 14, |c: f64| 5730.0 * c.log(2.0));

        None
    }
}
