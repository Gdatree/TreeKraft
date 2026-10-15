use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    macro_rules! g1 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let n=match vm.eval_expr(&code[$p.len()..code.len()-1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(n);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));} }; }
    macro_rules! g2 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let p=VM::split_top_level(&code[$p.len()..code.len()-1],',');if p.len()>=2{let a=match vm.eval_expr(&p[0]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let b=match vm.eval_expr(&p[1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(a,b);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));}}}; }
    macro_rules! g3 { ($p:expr,$l:expr,$f:expr) => { if code.starts_with($p)&&code.ends_with(')'){let p=VM::split_top_level(&code[$p.len()..code.len()-1],',');if p.len()>=3{let a=match vm.eval_expr(&p[0]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let b=match vm.eval_expr(&p[1]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let c=match vm.eval_expr(&p[2]){Ok(v)=>v,Err(e)=>return Some(Err(e))};let r=($f)(a,b,c);vm.echo(Value::fmt_num(r));return Some(Ok(Value::Number(r)));}}}; }

    // Площадь и периметр эллипса считаем по приближению Рамануджана поэтому ниже просим обе полуоси
    g2!("~ellipse_area(", 14, |a: f64, b: f64| std::f64::consts::PI * a * b);
    g2!("~ellipse_circ(", 14, |a: f64, b: f64| std::f64::consts::PI * (3.0*(a+b) - ((3.0*a+b)*(a+3.0*b)).sqrt()));

    // Площадь трапеции считаем по двум основаниям и высоте поэтому ниже просим все три числа
    g3!("~trapezoid_area(", 16, |a: f64, b: f64, h: f64| (a+b)*h/2.0);

    // Площадь сектора и длину дуги считаем по радиусу и углу в радианах чтобы не переводить градусы вручную
    g2!("~sector_area(", 13, |r: f64, a: f64| r*r*a/2.0);
    g2!("~arc_length(", 12, |r: f64, a: f64| r*a);

    // Объем пирамиды считаем по площади основания и высоте а третий аргумент держим запасным для единой формы
    g3!("~pyramid_volume(", 16, |b: f64, h: f64, _: f64| b*h/3.0);
    g2!("~pyramid_surface(", 17, |b: f64, s: f64| b + s);

    // Объем и площадь тора считаем по малому радиусу трубки и большому радиусу кольца поэтому порядок чисел ниже важен
    g2!("~torus_volume(", 14, |r: f64, R: f64| 2.0*std::f64::consts::PI.powi(2)*R*r.powi(2));
    g2!("~torus_surface(", 15, |r: f64, R: f64| 4.0*std::f64::consts::PI.powi(2)*R*r);

    // Объем капсулы считаем по радиусу и высоте как цилиндр с полусферами а третий аргумент держим запасным для единой формы
    g3!("~capsule_volume(", 16, |r: f64, h: f64, _: f64| std::f64::consts::PI*r*r*(h + 4.0*r/3.0));

    // Объем и площадь конуса считаем по радиусам и высоте чтобы одна формула покрывала и усеченный вариант
    g3!("~frustum_volume(", 16, |r1: f64, r2: f64, h: f64| std::f64::consts::PI*h*(r1*r1+r1*r2+r2*r2)/3.0);
    g2!("~cone_surface(", 14, |r: f64, s: f64| std::f64::consts::PI*r*(r+s));

    // Площадь правильного многоугольника считаем по числу сторон и длине стороны чтобы не хранить таблицы
    g2!("~regular_poly_area(", 19, |n: f64, s: f64| n*s*s/(4.0*(std::f64::consts::PI/n).tan()));
    g1!("~hexagon_area(", 14, |s: f64| 3.0*3.0_f64.sqrt()/2.0*s.powi(2));
    g1!("~octagon_area(", 14, |s: f64| 2.0*(1.0+2.0_f64.sqrt())*s.powi(2));

    // Хорды и сегменты считаем по радиусу и углу в радианах чтобы не переводить градусы вручную
    g2!("~chord_length(", 14, |r: f64, a: f64| 2.0*r*(a/2.0).sin());
    g2!("~segment_area(", 14, |r: f64, a: f64| r*r/2.0*(a-a.sin()));
    g2!("~segment_height(", 16, |r: f64, a: f64| r*(1.0-(a/2.0).cos()));

    // Длину касательных считаем по расстоянию между центрами и радиусам чтобы не строить чертеж вручную
    g3!("~internal_tangent(", 18, |d: f64, r1: f64, r2: f64| (d*d-(r1-r2).powi(2)).sqrt());
    g3!("~external_tangent(", 18, |d: f64, r1: f64, r2: f64| (d*d-(r1+r2).powi(2)).sqrt());

    // Гипотенузу и катет считаем по теореме Пифагора чтобы не выводить корень вручную
    g2!("~hypotenuse(", 12, |a: f64, b: f64| (a*a+b*b).sqrt());
    g2!("~pythagorean_leg(", 17, |c: f64, a: f64| (c*c-a*a).sqrt());

    // Площадь по Герону считаем по трем сторонам поэтому ниже просим все три числа
    g3!("~heron_area(", 12, |a: f64, b: f64, c: f64| { let s=(a+b+c)/2.0; (s*(s-a)*(s-b)*(s-c)).sqrt() });

    // Параллелограмм считаем по основанию и высоте а ромб и дельтоид по диагоналям чтобы каждая фигура считалась своими мерками
    g2!("~parallelogram_area(", 20, |b: f64, h: f64| b*h);
    g2!("~rhombus_area(", 14, |d1: f64, d2: f64| d1*d2/2.0);
    g2!("~kite_area(", 11, |d1: f64, d2: f64| d1*d2/2.0);

    // Объемные тела считаем каждое по своим размерам стандартными формулами чтобы не выводить их вручную
    g1!("~sphere_volume(", 15, |r: f64| 4.0/3.0*std::f64::consts::PI*r.powi(3));
    g2!("~sphere_cap_volume(", 19, |r: f64, h: f64| std::f64::consts::PI*h*h*(3.0*r-h)/3.0);
    g2!("~cylinder_volume(", 17, |r: f64, h: f64| std::f64::consts::PI*r.powi(2)*h);
    g2!("~cylinder_surface(", 18, |r: f64, h: f64| 2.0*std::f64::consts::PI*r*(r+h));
    g2!("~sphere_zone_area(", 18, |r: f64, h: f64| 2.0*std::f64::consts::PI*r*h);
    g2!("~prism_volume(", 14, |base: f64, h: f64| base*h);
    g1!("~cube_volume(", 13, |s: f64| s.powi(3));
    g1!("~cube_surface(", 14, |s: f64| 6.0*s.powi(2));
    g3!("~box_volume(", 12, |l: f64, w: f64, h: f64| l*w*h);
    g3!("~box_surface(", 13, |l: f64, w: f64, h: f64| 2.0*(l*w+l*h+w*h));
    g1!("~hemisphere_volume(", 19, |r: f64| 2.0/3.0*std::f64::consts::PI*r.powi(3));
    g2!("~annular_area(", 14, |r: f64, R: f64| std::f64::consts::PI*(R.powi(2)-r.powi(2)));

    // Плоские фигуры считаем по сторонам или радиусу чтобы площадь и периметр получались без ручных формул
    g2!("~triangle_area_bh(", 18, |b: f64, h: f64| b*h/2.0);
    g3!("~triangle_perimeter(", 20, |a: f64, b: f64, c: f64| a+b+c);
    g2!("~rectangle_area(", 16, |w: f64, h: f64| w*h);
    g2!("~rectangle_perimeter(", 21, |w: f64, h: f64| 2.0*(w+h));
    g1!("~square_perimeter(", 18, |s: f64| 4.0*s);
    g1!("~circle_circum(", 15, |r: f64| 2.0*std::f64::consts::PI*r);
    g1!("~circle_area(", 13, |r: f64| std::f64::consts::PI*r.powi(2));
    g1!("~pentagon_area(", 15, |s: f64| 1.72048*s.powi(2));
    g1!("~heptagon_area(", 15, |s: f64| 3.63391*s.powi(2));
    g1!("~nonagon_area(", 14, |s: f64| 6.18182*s.powi(2));
    g1!("~decagon_area(", 14, |s: f64| 7.69421*s.powi(2));
    g2!("~poly_perimeter(", 16, |n: f64, s: f64| n*s);

    g1!("~polygon_interior_angle(", 24, |n: f64| if n > 2.0 { (n - 2.0) * 180.0 / n } else { f64::NAN });
    g1!("~polygon_exterior_angle(", 24, |n: f64| if n > 2.0 { 360.0 / n } else { f64::NAN });
    g1!("~regular_hexagon_area(", 22, |s: f64| 1.5 * 3.0f64.sqrt() * s * s);
    g1!("~regular_hexagon_perimeter(", 27, |s: f64| 6.0 * s);
    g1!("~regular_hexagon_apothem(", 25, |s: f64| 3.0f64.sqrt() / 2.0 * s);
    g1!("~regular_octagon_area(", 22, |s: f64| 2.0 * (1.0 + 2.0f64.sqrt()) * s * s);
    g1!("~regular_pentagon_area(", 23, |s: f64| 0.25 * (5.0 * (5.0 + 2.0 * 5.0f64.sqrt())).sqrt() * s * s);
    g1!("~regular_tetra_volume(", 22, |e: f64| e.powi(3) / (6.0 * 2.0f64.sqrt()));
    g1!("~regular_octa_volume(", 21, |e: f64| 2.0f64.sqrt() / 3.0 * e.powi(3));
    g1!("~regular_icosa_volume(", 22, |e: f64| 5.0 / 12.0 * (3.0 + 5.0f64.sqrt()) * e.powi(3));
    g1!("~regular_dodeca_volume(", 23, |e: f64| (15.0 + 7.0 * 5.0f64.sqrt()) / 4.0 * e.powi(3));
    g1!("~regular_tetra_surface(", 23, |e: f64| 3.0f64.sqrt() * e * e);
    g1!("~regular_octa_surface(", 22, |e: f64| 2.0 * 3.0f64.sqrt() * e * e);
    g1!("~regular_icosa_surface(", 23, |e: f64| 5.0 * 3.0f64.sqrt() * e * e);
    g1!("~regular_dodeca_surface(", 24, |e: f64| 3.0 * (25.0 + 10.0 * 5.0f64.sqrt()).sqrt() * e * e);
    g1!("~hemisphere_vol2(", 17, |r: f64| 2.0 / 3.0 * std::f64::consts::PI * r.powi(3));
    g1!("~hemisphere_surface(", 20, |r: f64| 3.0 * std::f64::consts::PI * r * r);

    g2!("~regular_polygon_area(", 22, |n: f64, s: f64| if n < 3.0 { f64::NAN } else { n * s * s / (4.0 * (std::f64::consts::PI / n).tan()) });
    g2!("~regular_polygon_perimeter(", 27, |n: f64, s: f64| n * s);
    g2!("~regular_polygon_apothem(", 25, |n: f64, s: f64| if n < 3.0 { f64::NAN } else { s / (2.0 * (std::f64::consts::PI / n).tan()) });
    g2!("~circle_sector_area(", 20, |r: f64, d: f64| 0.5 * r * r * d.to_radians());
    g2!("~circle_sector_perimeter(", 25, |r: f64, d: f64| 2.0 * r + d.to_radians() * r);
    g2!("~circle_segment_area(", 21, |r: f64, h: f64| r * r * ((h / r).max(-1.0).min(1.0).acos()) - (r - h) * (2.0 * r * h - h * h).sqrt());
    g2!("~annulus_area(", 14, |r: f64, r2: f64| std::f64::consts::PI * (r * r - r2 * r2));
    g2!("~annulus_perimeter(", 19, |r: f64, r2: f64| 2.0 * std::f64::consts::PI * (r + r2));
    g2!("~torus_vol2(", 12, |r: f64, r2: f64| 2.0 * std::f64::consts::PI.powi(2) * r * r2 * r2);
    g2!("~torus_surf2(", 13, |r: f64, r2: f64| 4.0 * std::f64::consts::PI.powi(2) * r * r2);
    g2!("~capsule_vol2(", 14, |r: f64, h: f64| std::f64::consts::PI * r * r * (h - 2.0 * r / 3.0));
    g2!("~capsule_surf2(", 15, |r: f64, h: f64| 2.0 * std::f64::consts::PI * r * h);
    g2!("~sphere_cap_surface(", 20, |r: f64, h: f64| 2.0 * std::f64::consts::PI * r * h);
    g2!("~spherical_wedge_volume(", 24, |r: f64, d: f64| r.powi(3) * d.to_radians() / 3.0);
    g2!("~cylinder_side(", 15, |r: f64, h: f64| 2.0 * std::f64::consts::PI * r * h);
    g2!("~pyramid_vol2(", 14, |ba: f64, h: f64| ba * h / 3.0);
    g2!("~square_pyramid_volume(", 23, |s: f64, h: f64| s * s * h / 3.0);
    g2!("~square_pyramid_surface(", 24, |s: f64, l: f64| s * s + 2.0 * s * l);
    g2!("~spherical_shell_volume(", 24, |r: f64, r2: f64| 4.0 / 3.0 * std::f64::consts::PI * (r.powi(3) - r2.powi(3)));
    g2!("~spherical_shell_surface(", 25, |r: f64, r2: f64| 4.0 * std::f64::consts::PI * (r * r + r2 * r2));
    g2!("~prism_vol2(", 12, |ba: f64, h: f64| ba * h);
    g2!("~right_triangle_area(", 21, |a: f64, b: f64| a * b / 2.0);
    g2!("~trapezoid_midsegment(", 22, |a: f64, b: f64| (a + b) / 2.0);
    g2!("~circle_chord_length(", 21, |r: f64, d: f64| 2.0 * r * (d.to_radians() / 2.0).sin());
    g2!("~circle_arc_length(", 19, |r: f64, d: f64| r * d.to_radians());

    g3!("~rect_box_diagonal(", 19, |a: f64, b: f64, c: f64| (a * a + b * b + c * c).sqrt());
    g3!("~triangular_prism_volume(", 25, |b: f64, h: f64, l: f64| b * h / 2.0 * l);
    g3!("~rect_pyramid_volume(", 21, |l: f64, w: f64, h: f64| l * w * h / 3.0);
    g3!("~cylindrical_shell_volume(", 26, |r: f64, r2: f64, h: f64| std::f64::consts::PI * (r * r - r2 * r2) * h);
    g3!("~prism_surface(", 15, |p: f64, ba: f64, h: f64| p * h + 2.0 * ba);
    g3!("~triangle_inradius(", 19, |a: f64, b: f64, c: f64| { let s = (a + b + c) / 2.0; if s <= 0.0 { f64::NAN } else { ((s - a) * (s - b) * (s - c) / s).sqrt() } });
    g3!("~triangle_circumradius(", 23, |a: f64, b: f64, c: f64| { let s = (a + b + c) / 2.0; let area = (s * (s - a) * (s - b) * (s - c)).max(0.0).sqrt(); if area == 0.0 { f64::NAN } else { a * b * c / (4.0 * area) } });
    g3!("~triangle_median(", 17, |a: f64, b: f64, c: f64| 0.5 * (2.0 * b * b + 2.0 * c * c - a * a).sqrt());
    g3!("~triangle_altitude(", 19, |a: f64, b: f64, c: f64| { let s = (a + b + c) / 2.0; let area = (s * (s - a) * (s - b) * (s - c)).max(0.0).sqrt(); if a == 0.0 { f64::NAN } else { 2.0 * area / a } });

    None
}
