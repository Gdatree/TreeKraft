use std::io::{self, Write};

fn banner(title: &str) {
    let w = title.len() + 8;
    println!();
    println!("  \x1B[1;37m╔{}╗\x1B[0m", "═".repeat(w));
    println!("  \x1B[1;37m║   \x1B[1;92m{}\x1B[1;37m   ║\x1B[0m", title);
    println!("  \x1B[1;37m╚{}╝\x1B[0m", "═".repeat(w));
    println!();
}

fn section(name: &str) {
    println!("\n  \x1B[1;96m{}\x1B[0m", name);
    println!("  \x1B[2;36m{}\x1B[0m", "─".repeat(60));
}

fn cmd(c: &str, d: &str) {
    println!("  \x1B[97m{:<42}\x1B[2;37m{}\x1B[0m", c, d);
}

fn tip(t: &str) {
    println!("  \x1B[2;33m  > {}\x1B[0m", t);
}

fn sep() {
    println!("  \x1B[2;36m{}\x1B[0m", "─".repeat(60));
}

pub fn show_welcome() {
    println!();
    println!("  \x1B[1;92m  ╔════════════════════════════════════════════╗\x1B[0m");
    println!("  \x1B[1;92m  ║     \x1B[1;97mT R E E K R A F T   [ u1 ]\x1B[1;92m     ║\x1B[0m");
    println!("  \x1B[1;92m  ║     \x1B[2;37m2900+ команд · 57 модулей\x1B[1;92m     ║\x1B[0m");
    println!("  \x1B[1;92m  ╚════════════════════════════════════════════╝\x1B[0m");
    println!();
    println!("  \x1B[2;37m  Использование:\x1B[0m");
    println!("  \x1B[97m    help <тема>\x1B[2;37m    — справка по теме\x1B[0m");
    println!("  \x1B[97m    help\x1B[2;37m               — это окно (повторно — полный список)\x1B[0m");
    println!("  \x1B[97m    help en\x1B[2;37m            — English\x1B[0m");
    println!("  \x1B[97m    license\x1B[2;37m            — лицензия\x1B[0m");
    println!();
    sep();
    
    let topics = [
        ("core",      "ядро: переменные, if/while/for, функции"),
        ("math",      "математика + статистика + комбинаторика"),
        ("strings",   "строки: upper, split, base64, uuid, regex"),
        ("lists",     "списки, словари, стеки, очереди, сеты"),
        ("window ★",  "2D графика: окно, пиксели, линии, спрайты"),
        ("sqlite ★",  "базы данных SQLite"),
        ("server ★",  "HTTP-сервер"),
        ("files",     "файлы: read, write, copy, dir"),
        ("display",   "ASCII-экран (ld команды)"),
        ("tui",       "ASCII интерфейс: окна, кнопки, бары"),
        ("output",    "вывод: цвета, таблицы, чарты, прогресс"),
        ("process",   "процессы: exec, kill, list"),
        ("encoding",  "JSON, CSV, YAML, TOML кодеки"),
        ("crypto",    "шифры: md5, sha256, encrypt, base64"),
        ("audio",     "звук: beep, play, volume"),
        ("pattern",   "pluralize, slugify, truncate"),
        ("valid",     "валидация: email, url, ip, uuid"),
        ("regex",     "RegEx: match, find, replace, split"),
        ("units",     "конвертация: 100+ единиц измерения"),
        ("convert",   "bin, hex, int, float, ascii, roman"),
        ("html",      "генерация HTML: h1, div, table, ul, li"),
        ("compress",  "сжатие: gzip, zip"),
        ("algo",      "алгоритмы: bfs, dfs, factorial, gcd"),
        ("physics",   "физика: коллизии, гравитация, сила"),
        ("game",      "игровые системы: HUD, меню, счёт"),
        ("sprites",   "спрайты, анимации, частицы"),
        ("maps",      "карты и тайлы"),
        ("images",    "изображения: load, save, resize"),
        ("net",       "сеть: fetch, ping, dns"),
        ("system",    "система: env, exec, clipboard, дата"),
        ("calc",      "физ. константы: c, h, G, π и формулы"),
        ("chem",      "химия: элементы, pH, газовые законы"),
        ("bio",       "биология: BMI, BSA, калории, ЧСС"),
        ("sport",     "спорт: темп, пульс, 1ПМ, калории"),
        ("cook",      "кулинария: температура, вес, пиво"),
        ("geo",       "геометрия: площадь, объём, тригонометрия"),
        ("eng",       "инженерия: сопромат, гидравлика, электро"),
        ("music",     "музыка: ноты, темп, фильтры"),
        ("vet",       "ветеринария: возраст, вес, пульс"),
        ("agri",      "сельское хозяйство: урожай, pH, GDD"),
        ("debug",     "отладка: trace, profile, assert, watch"),
        ("tests",     "тестирование: assert_eq, test, benchmark"),
        ("auto",      "автоматизация: key, mouse, click, hotkey"),
        ("watch",     "отслеживание файлов"),
        ("ffi",       "FFI вызов C-функций [WIP]"),
        ("help",      "эта справка"),
    ];
    
    for (i, (name, desc)) in topics.iter().enumerate() {
        let branch = if i == topics.len() - 1 { "└──" } else { "├──" };
        println!("  \x1B[2;37m{}\x1B[97m{}/ \x1B[2;37m{}\x1B[0m", branch, name, desc);
    }
    
    println!();
    sep();
    println!("  \x1B[2;33m  Все команды работают без :  len(x)  =  len(x)\x1B[0m");
    println!("  \x1B[2;33m  help en  |  license  |  https://treekraft.tilda.ws/\x1B[0m");
    println!();
}

pub fn show_full_ru() {
    println!();
    println!("  \x1B[1;92m  ╔════════════════════════════════════════════╗\x1B[0m");
    println!("  \x1B[1;92m  ║     \x1B[1;97mTREEKRAFT   u1   —  полное дерево\x1B[1;92m  ║\x1B[0m");
    println!("  \x1B[1;92m  ╚════════════════════════════════════════════╝\x1B[0m");
    println!();
    
    section("ВЫВОД / ЦВЕТ");
    cmd("tx\"текст\", text(\"текст\")", "вывод текста");
    cmd("print(x), println(x), dump(x)", "вывод / отладка");
    cmd("red green blue yellow cyan magenta", "цветной вывод");
    cmd("rgb(r,g,b,t), gradient(t,c1,c2), rainbow(t)", "RGB / градиент");
    cmd("fg(c), bg(c), cur(x,y), bold italic underline", "ANSI стили");
    cmd("banner(t), box(t,c), hr(c,n), table(n), chart_bar(n,w)", "рамки / таблицы");
    cmd("progress(cur,max,w), loading_bar(cur,max,w)", "прогресс-бары");
    cmd("al(t)", "toast-уведомление");
    cmd("quote, bullet, check, cross, arrow", "маркеры");
    cmd("typewriter(t), scroll_text", "анимация текста");
    
    section("МАТЕМАТИКА");
    cmd("+ - * / ^ %, inc dec", "арифметика");
    cmd("abs sqrt cbrt root round floor ceil sign trunc", "базовые функции");
    cmd("sin cos tan asin acos atan2 sinh cosh tanh", "тригонометрия");
    cmd("log log2 log10 exp pow deg rad hypot", "логарифмы / степени");
    cmd("min max clamp lerp map between smoothstep", "диапазоны");
    cmd("gcd lcm fact fib perm comb is_prime prime", "теория чисел");
    cmd("pi e randf rndbool coin dice rnd(a,b)", "константы / RNG");
    cmd("mean median variance stddev product percentile", "статистика");
    cmd("dot cross vnorm distance dist3d midpoint", "векторы / геометрия");
    cmd("sigmoid relu softplus logit normal_pdf normal_cdf", "ML функции");
    cmd("ease_* (quad, cubic, sine, bounce)", "easing-функции");
    
    section("СТРОКИ");
    cmd("upper lower capitalize title swapcase reverse", "регистр");
    cmd("trim lstrip rstrip", "обрезка");
    cmd("len count find contains startswith endswith", "длина / поиск");
    cmd("split concat replace sub rep chunk join", "изменение");
    cmd("center ljust rjust pad pad_left pad_right pad_both", "выравнивание");
    cmd("isnum isalpha isdigit isalnum isspace isupper islower", "проверки");
    cmd("url_encode url_decode base64_encode base64_decode", "кодирование");
    cmd("html_escape, html_unescape, escape_json, escape_xml", "экранирование");
    cmd("rot13, caesar, atbash, pig_latin, morse", "шифры");
    cmd("uuid, hash, crc32", "генерация");
    cmd("regex_match, regex_find, regex_replace, regex_split", "RegEx");
    
    section("СПИСКИ / СЛОВАРИ");
    cmd("list add get set remove insert pop sort len", "базовые операции");
    cmd("list_map(l,fn) list_filter(l,fn) list_reduce(l,fn,i)", "функциональный стиль");
    cmd("list_find list_findall list_sortby list_any list_all", "поиск / проверка");
    cmd("range slice join shuffle choice sample", "генерация / выбор");
    cmd("list_zip_with list_cartesian list_permutations list_combinations", "комбинаторика");
    cmd("list_window list_pairs list_chunk list_fill list_repeat", "нарезка");
    cmd("list_flatten list_partition list_group_by list_intersperse", "преобразование");
    cmd("dict dset dget ddel dkeys dvals dhas dlen", "словари");
    cmd("stack push pop_stack queue enqueue dequeue", "стек / очередь");
    cmd("set_new set_add set_has set_union set_inter set_diff", "сеты");
    
    section("2D ГРАФИКА  ★ НОВОЕ");
    cmd("win_open(w,h,title), win_close, win_running, win_update", "управление окном");
    cmd("win_clear(r,g,b), win_pixel(x,y,r,g,b)", "заливка / пиксель");
    cmd("win_line(x1,y1,x2,y2,r,g,b)", "линия");
    cmd("win_rect, win_fill_rect(x,y,w,h,r,g,b)", "прямоугольник");
    cmd("win_circle, win_fill_circle(cx,cy,r,r,g,b)", "окружность");
    cmd("win_tri, win_fill_tri(x1,y1,..,x3,y3,r,g,b)", "треугольник");
    cmd("win_key, win_key_down(k), win_mouse_x/y, win_mouse_btn(n)", "ввод");
    cmd("win_img(name,x,y)", "изображение");
    
    section("БАЗЫ ДАННЫХ  ★ НОВОЕ");
    cmd("db_open(path), db_close", "открыть / закрыть");
    cmd("db_exec(sql), db_query(sql)", "SQL запросы");
    cmd("db_tables, db_columns(table), db_create(t,cols)", "схема");
    cmd("db_insert(table,values)", "вставка");
    
    section("HTTP-СЕРВЕР  ★ НОВОЕ");
    cmd("srv_start(port), srv_stop, srv_accept(timeout)", "сервер");
    
    section("ENCODING / ПРОЦЕССЫ / PATTERN");
    cmd("json_encode(var), json_decode(str), csv_encode(list)", "JSON/CSV");
    cmd("exec_output(cmd), exec_status(cmd), exec_bg(cmd)", "процессы");
    cmd("process_kill(pid), process_list, process_exists(pid)", "управление процессами");
    cmd("pluralize(n,one,few,many), slugify(s), truncate(s,n)", "строки");
    cmd("wrap_text(s,w), indent(s,n), dedent(s)", "форматирование");
    
    section("ФАЙЛЫ / СИСТЕМА");
    cmd("exs, read, write, append, readlines, dir, mkdir, rmdir", "файлы");
    cmd("copy, rename, move, delfile, filesize, filetype, touch", "файлы 2");
    cmd("env_get, env_set, hostname, username, pid, cwd", "система");
    cmd("os, arch, tempdir, homedir, cpu_count, uptime", "система 2");
    cmd("cls, beep, waitms, sleepms, key, inkey", "консоль");
    cmd("now, date, year, month, day, hour, minute, second", "время");
    cmd("shutdown, restart, logoff, sleep", "питание");
    
    section("ФИЗИЧЕСКИЕ КОНСТАНТЫ / CALC");
    cmd("speed_of_light, planck, gravitational, boltzmann", "константы");
    cmd("avogadro, gas_constant, stefan_boltzmann, rydberg", "константы 2");
    cmd("electron_mass, proton_mass, neutron_mass, amu", "массы");
    cmd("bohr_radius, hartree, fine_structure", "квантовая");
    cmd("wien_displacement, schwarzschild, doppler_shift", "формулы");
    
    section("ХИМИЯ / CHEM");
    cmd("atomic_symbol(n), atomic_mass(n)", "элементы");
    cmd("ph_from_h, h_from_ph, poh_from_ph", "pH");
    cmd("molarity, normality, ideal_gas_volume, stp_volume", "растворы/газы");
    cmd("photon_energy, rydberg_energy, binding_energy", "квантовая химия");
    
    section("БИОЛОГИЯ / BIO");
    cmd("bmi, bsa_mosteller, bsa_dubois, ibm_devine", "антропометрия");
    cmd("body_fat, lean_mass, blood_volume, gfr", "состав тела");
    cmd("max_hr, target_hr, vo2max, calorie_resting", "физиология");
    cmd("pregnancy_week, due_date, apgar_score", "акушерство");
    
    section("СПОРТ / SPORT");
    cmd("pace_speed, speed_pace, race_time, race_distance", "бег");
    cmd("calories_run, calories_walk, calories_cycle, calories_swim", "калории");
    cmd("one_rep_max, epley, brzycki, wilks", "силовые");
    
    section("ГЕОМЕТРИЯ / GEO");
    cmd("circle_area, circle_circ, ellipse_area, torus_volume", "объёмы/площади");
    cmd("capsule_volume, frustum_volume, heron_area, hypotenuse", "объёмы 2");
    cmd("gps_distance, bearing, gps_midpoint, point_in_poly", "GPS/карты");
    cmd("line_intersect, segment_intersect, poly_area", "геометрия");
    
    section("ИНЖЕНЕРИЯ / ENG");
    cmd("beam_deflection, beam_stress, column_buckling", "сопромат");
    cmd("reynolds, drag_force, lift_force, bernoulli", "гидравлика");
    cmd("ohm_law, power_law, resonant_freq, time_constant", "электроника");
    cmd("stress, strain, modulus, poisson, snell", "прочность / оптика");
    
    section("МУЗЫКА / MUSIC");
    cmd("note_freq(n), midi_note(f), cents(r)", "частоты нот");
    cmd("bpm_to_ms, bpm_to_hz, delay_time", "темп / задержка");
    cmd("lowpass(fc[,sr]), highpass(fc[,sr]), reverb_time", "фильтры");
    
    section("КУЛИНАРИЯ / COOK / VET / AGRI / FISH");
    cmd("baking_temp, cooking_time, roast_time, turkey_time", "готовка");
    cmd("brew_gravity, abv, ibu, srm_to_ebc", "пивоварение");
    cmd("dog_age, cat_age, horse_age, pet_calories", "ветеринария");
    cmd("pregnancy_dog, pregnancy_cat, pregnancy_horse", "вет. акушерство");
    cmd("acre_to_sqm, hectare_to_acre, yield_per_acre", "агрономия");
    cmd("fish_age, fish_weight, tide_height, moon_phase", "рыбалка");
    cmd("dissolved_oxygen, water_temp, wave_height", "рыбалка 2");
    
    section("ДИСПЛЕЙ / TUI");
    cmd("ldsc(w,h,b,fill), ld(x,y,c), ldtext, ldbox, ldm", "ASCII-экран");
    cmd("ldrect, ldcircle, ldfill_circle, ldscroll, ldsave", "ASCII доп");
    cmd("tui_init, tui_win, tui_text, tui_btn, tui_bar", "TUI");
    
    section("ИГРОВЫЕ СИСТЕМЫ");
    cmd("sprite, anim, play, fps, delta", "спрайты/анимация");
    cmd("loadmap, tile, settile, drawmap, obj, spawn", "карты");
    cmd("collide_rect, collide_circ, camera, shake, zoom", "физика");
    cmd("score, lives, hp, xp, menu, hud, pause", "UI/счёт");
    cmd("particle_burst, flash, fade_in, fade_out", "эффекты");
    
    section("ИЗОБРАЖЕНИЯ");
    cmd("img_load, img_save, img_width, img_height", "загрузка");
    cmd("img_resize, img_crop, img_rotate, img_flip", "редактирование");
    cmd("img_pixel, img_setpixel, img_fill, img_draw_line", "рисование");
    cmd("img_grayscale, img_invert", "фильтры");
    
    section("СЕТЬ / КРИПТО");
    cmd("fetch(url), http_get, http_post, ping", "HTTP");
    cmd("online, ip, dns, netstat, mac", "сеть");
    cmd("md5, sha256, base64enc/dec, encrypt/decrypt", "крипто");
    cmd("rot13, caesar_cipher, random_bytes", "крипто 2");
    
    section("ОТЛАДКА / ТЕСТЫ / АВТОМАТИЗАЦИЯ");
    cmd("trace on/off, profile_start, profile_stop, profile_report", "отладка");
    cmd("assert, assert_eq, test, test_report, benchmark", "тесты");
    cmd("send(k), click(x,y), hotkey(k,cmd)", "автокликер");
    cmd("watch_file, watch_check, watch_stop", "file watcher");
    cmd("dl_open, dl_call, dl_close", "FFI (WIP)");
    
    section("ПЕРЕМЕННЫЕ / УПРАВЛЕНИЕ");
    cmd("x = val, const x = val, del x, local x=v", "переменные");
    cmd("if/then/else/end, while/do/end, for/in/do/end", "условия / циклы");
    cmd("loop/break/end, continue", "циклы");
    cmd("def fn(a,b), body, call fn(a,b), return", "функции");
    cmd("try, catch, throw", "ошибки");
    cmd("import \"file.tk\" as ns, export var", "модули");
    cmd("noecho on/off", "тихий режим");
    cmd("vars, funcs, consts, modules, history", "интроспекция");
    cmd("mem, memclear, reload", "память");
    
    println!();
    sep();
    println!("  \x1B[2;37m  help <тема> — детальная справка  |  help en  |  license\x1B[0m");
    println!();
}

pub fn show_full_en() {
    println!();
    println!("  \x1B[1;92mTreeKraft u1 — English Reference\x1B[0m");
    println!();
    println!("  Commands work WITHOUT  prefix: len(x) = len(x)");
    println!();
    println!("  Use: help en <topic>");
    println!("  Topics: start math str list window sqlite server files display tui");
    println!("          output process encoding crypto audio pattern valid regex");
    println!("          units convert html compress algo physics game sprites maps");
    println!("          images net system calc chem bio sport cook geo eng music");
    println!("          vet agri debug tests auto watch ffi");
    println!();
    println!("  \x1B[1;96mNEW in u1:\x1B[0m");
    let new_cmds = [
        "win_* — 2D graphics (minifb): window, pixels, lines, circles, sprites, keyboard/mouse",
        "db_* — SQLite: open, exec, query, tables, columns, insert",
        "srv_* — HTTP server: start, stop, accept",
        "json_encode/decode, csv_encode/decode, yaml/toml encode/decode",
        "exec_output/status/bg, process_kill/list/exists, shell",
        "regex_match/find/findall/replace/split",
        "100+ unit conversions (temp, length, weight, speed, volume...)",
        "valid_email/url/ip/port/hex/uuid/date/alpha/alnum/digit/glob",
        "html_tag/div/span/p/a/img/h1-6/table/ul/li/code/pre/strong/em...",
        "gzip_encode/decode, zip_list/extract",
        "bfs, dfs, factorial, fibonacci, collatz, gcd, lcm",
        "sigmoid, relu, softplus, logit, normal_pdf/cdf",
        "vec_add/sub/scale, mat_det/transpose",
        "dot, cross, vnorm, distance, dist3d, midpoint",
        "pluralize, slugify, truncate, wrap_text, indent, dedent",
        "audio_beep, audio_play, audio_volume",
        "tui_init, tui_win, tui_text, tui_btn, tui_bar",
        "dl_open/call/close (FFI stub)",
        "watch_file/check/stop/list",
        "profile_start/stop/report, stopwatch",
        "import \"file.tk\" as ns, export var",
        "speed_of_light, planck, gravitational, avogadro — physics constants",
        "atomic_symbol, atomic_mass, ph formulas — chemistry",
        "bmi, bsa, body_fat, max_hr — biology/medicine",
        "pace, race_time, calories, one_rep_max — sports",
        "baking_temp, cooking_time, brew_gravity — cooking",
        "circle_area, ellipse, torus, gps_distance — geometry",
        "beam_deflection, ohm_law, reynolds — engineering",
        "note_freq, bpm_to_ms, lowpass — music/DSP",
        "dog_age, cat_age, pet_calories — veterinary",
        "acre_to_sqm, yield_per_acre, gdd — agriculture",
        "fish_age, tide_height, moon_phase — fishing",
    ];
    for cmd in &new_cmds { println!("  \x1B[97m•\x1B[0m {}", cmd); }
    println!();
}

pub fn show_topic(topic: &str, lang: &str) {
    let ru = lang == "ru";
    let row = |c: &str, d: &str| println!("  \x1B[97m{:<44}\x1B[2;37m{}\x1B[0m", c, d);
    let sec = |s: &str| { println!("\n  \x1B[1;96m── {} {}\x1B[0m", s, "─".repeat(56_usize.saturating_sub(s.len()))); };
    let tip = |t: &str| println!("  \x1B[2;33m  > {}\x1B[0m", t);

    match topic {
        "start" => {
            sec(if ru{"ПЕРВЫЕ ШАГИ"}else{"GETTING STARTED"});
            row("x = 10", if ru{"переменная"}else{"variable"});
            row("text(\"Hello\")", if ru{"вывод"}else{"print"});
            row("name = in(\"Name: \")", if ru{"ввод"}else{"input"});
            row("arg(0), args()", if ru{"аргументы командной строки"}else{"CLI args"});
            row("if x>5 then text(\"big\")", "");
            row("while i<10 do i = i + 1 end", "");
            row("for x in list do text(x) end", "");
            row("def sum(a,b), a+b", "");
            row("call sum(2,3)", "→ 5");
            tip(if ru{"Все команды без : len(x) = len(x)"}else{"All commands work without : len(x) = len(x)"});
        }
        "window" | "win" | "2d" => {
            sec(if ru{"2D ГРАФИКА"}else{"2D GRAPHICS"});
            row("win_open(w,h,title)", if ru{"создать окно"}else{"create window"});
            row("win_close, win_running, win_update", "");
            row("win_clear(r,g,b), win_pixel(x,y,r,g,b)", "");
            row("win_line(x1,y1,x2,y2,r,g,b)", "линия / line");
            row("win_rect, win_fill_rect(x,y,w,h,r,g,b)", "прямоугольник / rect");
            row("win_circle, win_fill_circle(cx,cy,r,r,g,b)", "круг / circle");
            row("win_tri, win_fill_tri", "треугольник / triangle");
            row("win_key, win_key_down(k)", "клавиатура / keyboard");
            row("win_mouse_x, win_mouse_y, win_mouse_btn(n)", "мышь / mouse");
            row("win_img(name,x,y)", if ru{"изображение"}else{"draw image"});
            tip("win_open(800,600,\"Game\"); win_clear(0,0,255); win_update()");
        }
        "sqlite" | "db" => {
            sec(if ru{"SQLite"}else{"SQLite"});
            row("db_open(path)", if ru{"открыть БД"}else{"open database"});
            row("db_create(\"t\",\"id,name\")", if ru{"создать таблицу"}else{"create table"});
            row("db_insert(\"t\", list(1,\"A\"))", if ru{"вставить"}else{"insert"});
            row("db_query(\"SELECT * FROM t\")", if ru{"SQL запрос"}else{"SQL query"});
            row("db_tables, db_columns(\"t\")", if ru{"схема"}else{"schema"});
            row("db_exec(\"CREATE INDEX...\")", if ru{"произвольный SQL"}else{"custom SQL"});
            tip("db_open(\"test.db\"); db_exec(\"CREATE TABLE users(id,name)\")");
        }
        "math" => {
            sec(if ru{"МАТЕМАТИКА"}else{"MATH"});
            row("+ - * / ^ %, inc dec", "");
            row("abs sqrt cbrt root round floor ceil sign trunc fract", "");
            row("sin cos tan asin acos atan2 sinh cosh tanh", "");
            row("log log2 log10 exp pow deg rad hypot", "");
            row("min max clamp lerp map(x,il,ih,ol,oh) between smoothstep", "");
            row("gcd lcm fact fib perm comb is_prime prime", "");
            row("pi e randf rndbool coin dice rnd randint", "");
            row("mean median variance stddev product percentile entropy", "");
            row("sigmoid relu softplus logit normal_pdf normal_cdf", "");
            row("dot cross vnorm distance dist3d midpoint", "");
            row("ease_quad_in/out, ease_cubic_in/out, ease_sine_in/out, ease_bounce_out", "");
        }
        "algo" => {
            sec(if ru{"АЛГОРИТМЫ"}else{"ALGORITHMS"});
            row("bfs(adj, a, b), dfs(adj, a, b)", if ru{"поиск пути"}else{"path search"});
            row("factorial(n), fibonacci(n), collatz(n)", "");
            row("gcd(a, b), lcm(a, b), is_prime(n), prime(n)", "");
            row("euler_totient(n), digital_root(n), isqrt(n)", "");
        }
        "units" => {
            sec(if ru{"КОНВЕРТАЦИЯ"}else{"UNIT CONVERSION"});
            row("c_to_f, f_to_c, c_to_k, k_to_c", "температура");
            row("m_to_km, km_to_m, m_to_inch, m_to_ft, m_to_mi", "длина");
            row("kg_to_lb, lb_to_kg, kg_to_oz, oz_to_kg", "вес");
            row("l_to_gal, gal_to_l, l_to_ml, ml_to_l", "объём");
            row("kmh_to_ms, ms_to_kmh, mph_to_kmh, kmh_to_mph", "скорость");
            row("w_to_hp, hp_to_w, j_to_cal, cal_to_j", "мощность/энергия");
            row("au_to_km, ly_to_km, pc_to_ly", "астрономия");
            row("hz_to_khz, khz_to_hz, mhz_to_hz", "частота");
            row("v_to_kv, a_to_ma, ohm_to_kohm", "электричество");
            tip("Всего 100+ конвертеров: температура, длина, вес, скорость...");
        }
        "calc" | "physics" => {
            sec(if ru{"ФИЗИЧЕСКИЕ КОНСТАНТЫ"}else{"PHYSICAL CONSTANTS"});
            row("speed_of_light", "299,792,458 m/s");
            row("planck", "6.626e-34 J·s");
            row("gravitational", "6.674e-11 N·m²/kg²");
            row("boltzmann", "1.381e-23 J/K");
            row("avogadro", "6.022e23 mol⁻¹");
            row("electron_mass, proton_mass, neutron_mass", "массы частиц");
            row("schwarzschild(m)", "радиус Шварцшильда");
            row("wien_displacement(t)", "закон Вина");
            row("doppler_shift(v)", "релятивистский эффект Доплера");
        }
        "chem" => {
            sec(if ru{"ХИМИЯ"}else{"CHEMISTRY"});
            row("atomic_symbol(n), atomic_mass(n)", "элементы до 118");
            row("ph_from_h(h), h_from_ph(ph)", "pH");
            row("ideal_gas_volume(n), stp_volume(n)", "газы");
            row("photon_energy(f), photon_wavelength(e)", "фотоны");
            row("molality(m,kg), normality(eq,L), delta_g(g2,g1)", "растворы/дельты");
            row("gibbs_free(h,s[,t]), vdw(a,b,Vm[,t]), nernst(e0,n,Q[,t])", "термодинамика");
            row("cell_potential(ec,ea)", "электрохимия");
            row("binding_energy(m), decay_constant(hl)", "ядерная");
        }
        "bio" | "sport" => {
            sec(if ru{"БИОЛОГИЯ/СПОРТ"}else{"BIOLOGY/SPORTS"});
            row("bmi(w[,h]), bsa_mosteller(w[,h]), body_fat(bmi[,age])", "антропометрия");
            row("max_hr(a), target_hr(a), vo2max(a)", "пульс");
            row("calorie_resting(w[,h,age]), water_intake(w)", "калории/вода");
            row("apgar_score(a,b,c,d,e) — 5 оценок 0-2", "акушерство");
            row("pace_speed(p), race_time(d,p), calories_run(w,d)", "бег");
            row("one_rep_max(w,r), epley(w,r), brzycki(w,r)", "силовые");
        }
        "vet" => {
            sec(if ru{"ВЕТЕРИНАРИЯ"}else{"VETERINARY"});
            row("dog_age(h), cat_age(h), horse_age(h)", "возраст");
            row("pet_calories(w), pet_water(w), pet_food(w)", "питание");
            row("pregnancy_dog, pregnancy_cat, pregnancy_horse", "беременность");
            row("heart_rate_dog, heart_rate_cat, heart_rate_horse", "пульс");
        }
        "cook" => {
            sec(if ru{"КУЛИНАРИЯ"}else{"COOKING"});
            row("baking_temp(c,f), cooking_time(w)", "температура/время");
            row("turkey_time(w,s), roast_time(w)", "мясо");
            row("brew_gravity(sg), abv(og), ibu(h,a)", "пиво");
            row("dilution(c1,c2), dough_hydration(w,f)", "кулинария");
        }
        "geo" | "geo3" => {
            sec(if ru{"ГЕОМЕТРИЯ"}else{"GEOMETRY"});
            row("circle_area(r), circle_circ(r), ellipse_area(a,b)", "площади");
            row("torus_volume(R,r), capsule_volume(r,h), frustum_volume(r1,r2,h)", "объёмы");
            row("gps_distance(lat1,lon1,lat2,lon2)", "GPS дистанция");
            row("bearing(lat1,lon1,lat2,lon2)", "азимут");
            row("heron_area(a,b,c), hypotenuse(a,b)", "треугольники");
        }
        "eng" => {
            sec(if ru{"ИНЖЕНЕРИЯ"}else{"ENGINEERING"});
            row("beam_deflection(f,l), beam_stress(m,z)", "сопромат");
            row("column_buckling(e,i,l)", "устойчивость");
            row("reynolds(v,d), drag_force(v,a), lift_force(v,a)", "гидро/аэро");
            row("ohm_law(v,i), power_law(v,i), resonant_freq(l,c)", "электро");
            row("stress(f,a), strain(dl,l), modulus(s,st)", "прочность");
        }
        "music" => {
            sec(if ru{"МУЗЫКА"}else{"MUSIC"});
            row("note_freq(n), midi_note(f), cents(r)", "частоты");
            row("bpm_to_ms(b), bpm_to_hz(b), delay_time(b[,beats])", "темп");
            row("lowpass(fc[,sr]), highpass(fc[,sr]), reverb_time(v[,a])", "фильтры/зал");
            row("sawtooth(t), square_wave(t), triangle_wave(t), noise(t)", "волны");
        }
        "agri" => {
            sec(if ru{"СЕЛЬСКОЕ ХОЗЯЙСТВО"}else{"AGRICULTURE"});
            row("acre_to_sqm, hectare_to_acre, sqm_to_acre", "площадь");
            row("yield_per_acre(p,a), yield_per_ha(p,a), seed_rate(m,a)", "урожай/нормы");
            row("gdd(t), frost_days(t), fertilizer_rate(m,a)", "вегетация");
            row("compost_ratio(g,b), population(n,a)", "почва/популяция");
        }
        "fish" => {
            sec(if ru{"РЫБАЛКА"}else{"FISHING"});
            row("fish_age(l), fish_weight(l), fish_condition(k)", "рыба");
            row("tide_height(h), moon_phase(d), solunar_rating(r)", "природа");
            row("water_temp(t), dissolved_oxygen(t)", "вода");
            row("casting_distance(p), trolling_speed(s)", "снасти");
        }
        "packages" | "pkg" | "tkp" => {
            sec(if ru{"ПАКЕТЫ (tkp)"}else{"PACKAGES (tkp)"});
            row("tkp install ник/имя[@ветка]", if ru{"установить из GitHub"}else{"install from GitHub"});
            row("tkp update [ник/имя]", if ru{"переустановить всё/один"}else{"re-fetch all/one"});
            row("tkp remove имя, tkp list, tkp path имя", if ru{"управление"}else{"manage"});
            row("~import_pkg(\"имя\")", if ru{"использовать пакет в скрипте"}else{"use package in script"});
            row("~trek_pkg install/list/remove/search/path", if ru{"то же из скрипта"}else{"same from script"});
            tip("tkp install tirken/std_math@v1; ~import_pkg(\"std_math\")");
        }
        "build" | "exe" | "bundle" => {
            sec(if ru{"СБОРКА EXE (build)"}else{"BUILD EXE"});
            row("treekraft build app.tk [-o app.exe]", if ru{"скомпилировать скрипт в exe"}else{"compile script to exe"});
            tip("treekraft build game.tk");
        }
        "tests" | "test" => {
            sec(if ru{"ТЕСТИРОВАНИЕ"}else{"TESTING"});
            row("assert_eq(a, b), assert_true(x)", if ru{"проверки (рвут скрипт)"}else{"checks (abort script)"});
            row("test(\"имя\", команда)", if ru{"проверка без останова"}else{"non-aborting check"});
            row("test_report(), test_reset()", if ru{"отчёт / сброс"}else{"report / reset"});
            row("treekraft test a.tk [b.tk ...]", if ru{"раннер: все падения + exit-код"}else{"runner: all failures + exit code"});
            tip("treekraft test tests/correctness.tk");
        }
        _ => {
            if ru {
                println!("\n  \x1B[91mОшибка: тема '{}' не найдена.\x1B[0m", topic);
                print!("  \x1B[2;37mТемы: ");
            } else {
                println!("\n  \x1B[91mError: topic '{}' not found.\x1B[0m", topic);
                print!("  \x1B[2;37mTopics: ");
            }
            let all_topics = ["start","math","strings","lists","window","sqlite","server",
                "files","display","tui","output","process","encoding","crypto","audio",
                "pattern","valid","regex","units","convert","html","compress","algo",
                "physics","game","sprites","maps","images","net","system","calc",
                "chem","bio","sport","cook","geo","eng","music","vet","agri","fish",
                "debug","tests","auto","watch","ffi"];
            for (i, t) in all_topics.iter().enumerate() {
                if i > 0 { print!(" "); }
                print!("\x1B[97m{}\x1B[0m", t);
            }
            println!();
        }
    }
    println!();
}

pub fn show_license() {
    println!();
    println!("  \x1B[1;92m╔════════════════════════════════════════════╗\x1B[0m");
    println!("  \x1B[1;92m║     \x1B[1;97mTREEKRAFT  u1  ·  License\x1B[1;92m           ║\x1B[0m");
    println!("  \x1B[1;92m╚════════════════════════════════════════════╝\x1B[0m");
    println!();
    println!("  \x1B[97mTreeKraft Programming Language  u1\x1B[0m");
    println!("  \x1B[2;37mCopyright (c) TreeKraft Team\x1B[0m");
    println!();
    println!("  \x1B[2;37mThis software is provided \"as is\",\x1B[0m");
    println!("  \x1B[2;37mwithout warranty of any kind.\x1B[0m");
    println!("  \x1B[2;37mFree to use, modify and distribute.\x1B[0m");
    println!();
    println!("  \x1B[2;37mhttps://treekraft.tilda.ws/\x1B[0m");
    println!();
}
