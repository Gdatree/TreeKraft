use crate::main::vm::{VM, Value};

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with('~') || code.starts_with("kg.")
        || code.starts_with("if ") || code.starts_with("while ") || code.starts_with("for ")
        || code == "loop" || code == "break" || code == "continue"
        || code == "end" || code == "else" || code.is_empty()
    {
        return None;
    }

    match code {
        "cls"|"beep"|"vars"|"now"|"date"|"sysinfo"|"version"|"pi"|"e"|"history"
        |"lds"|"ldclear"|"ldshow"|"mem"|"memclear"|"reload"|"showmap"
        |"trace on"|"trace off"|"randf"|"rndf"|"rndbool"|"coin"
        |"drawmap"|"mapsize"|"pause"|"framecount"|"fps_show"
        |"online"|"ip"|"uuid"|"netstat"
        |"procs"|"clipboard"|"homedir"|"hostname"|"username"
        |"os"|"arch"|"tempdir"|"nowms"|"time"|"datetime"
        |"today"|"timestamp"|"sleep"|"wait"
        => return Some(crate::main::eval::evaluate(vm, &format!("~{}", code))),
        _ => {}
    }

    let prefixes: &[&str] = &[
        // Ввод с клавиатуры и мыши пишем короткими формами без тильды чтобы движок сам подставил префикс в цикле ниже
        "keydown(","keyup(","keys(","keyname(","keycode(",
        "win_text(","win_sprite(","win_screenshot(","win_particles(","win_particles_step(",
        "win_rotate(","win_scale(","win_translate(","win_reset_transform",
        "mouse(","mouse_btn(","mouse_wheel(","inkey(","readkey(",
        "waitkey(","waitms(","waituntil(","debounce(",

        // Вывод текста и стили пишем коротко без тильды чтобы печать и цвета укладывались в одно слово
        "print(","println(","echo(","dump(","format(",
        "bold(","italic(","underline(","strikethrough(","blink(","hidden(",
        "resetcolor(","color(","bgcolor(","progressbar(","spinner(",

        // Математику пишем короткими именами без тильды чтобы формулы читались как обычно
        "inc ","dec ","del ",
        "rgb(","rgb_bg(","red(","green(","blue(","yellow(","cyan(","magenta(",
        "rnd(","rnds(","randf","rndf","rndbool","coin(","dice(","rndcolor(",
        "shuffle(","choice(","sample(","unique(","flatten(",
        "abs(","sqrt(","cbrt(","root(","sin(","cos(","tan(","asin(","acos(","atan2(",
        "sinh(","cosh(","tanh(",
        "floor(","ceil(","round(","log(","log2(","log10(","exp(",
        "min(","max(","clamp(","lerp(","hypot(","between(","gcd(","lcm(","fact(","fib(",
        "sign(","deg(","rad(","trunc(","fract(","pow(","map(",
        "is_prime(","prime(","randint(","randrange(",
        "to_ordinal(","to_roman(","from_roman(",

        // Строки пишем короткими именами без тильды чтобы движок сам дописал префикс при разборе в цикле ниже
        "upper(","lower(","trim(","lstrip(","rstrip(","rev(","rep(","len(",
        "split(","concat(","replace(","sub(","find(","count(",
        "startswith(","endswith(","contains(","empty(",
        "indexof(","lastindexof(",
        "isnum(","isalpha(","isdigit(","isalnum(","isspace(","isupper(","islower(",
        "capitalize(","swapcase(","title(","titlecase(","camelcase(","snakecase(",
        "center(","indent(","pad(","padleft(","padright(","printf(",
        "wordwrap(","chunk(","sortchars(",
        "charat(","codeat(","fromcode(",
        "url_encode(","url_decode(","html_escape(","html_unescape(",
        "levenshtein(","morse_encode(","caesar_cipher(","atbash(","pig_latin(",
        "rle_compress(","rle_decompress(",
        "rot13(","b64enc(","b64dec(","hash(","crc32(","uuid(","hex(","unhex(",

        // Списки пишем короткими именами без тильды чтобы вызывать те же команды движка короче
        "list(","add(","get(","set(","remove(","insert(","pop(",
        "sort(","sort_desc(","sort_by_len(","rev(",
        "range(","slice(","join(",
        "list_fill(","list_zip(","list_take(","list_drop(",
        "list_head(","list_last(","list_tail(","list_init(",
        "list_rotate(","list_chunk(",
        "list_contains(","list_index(","list_count(",
        "list_diff(","list_inter(","list_union(",
        "list_first(","list_shift(","list_unshift(",
        "list_removeval(","list_concat(",
        "lambda(",
        "list_map(","list_filter(","list_reduce(",
        "list_find(","list_findall(","list_sortby(",
        "list_sum(","list_avg(","list_min(","list_max(","list_std(",
        "stats(","cumsum(","normalize(","most_freq(","freq_map(",
        "running_sum(",

        // Словари пишем короткими именами без тильды чтобы не писать везде полную форму
        "dict","dset(","dget(","ddel(","dkeys(","dvals(","dhas(","dlen(",
        "ditems(","dmerge(","dcopy(","dclear(",
        "ddefault(","dupdate(","dgroupby(","dcountby(",
        "dict_from_lists(","dict_invert(","dict_to_list(",
        "keys(","values(",

        // Стеки очереди и множества пишем короткими именами без тильды чтобы вызывать те же команды движка короче
        "push(","pop_stack(","peek_stack(","stack_size(","stack_empty(","stack(",
        "enqueue(","dequeue(","queue_peek(","queue_size(","queue_empty(","queue(",
        "set_new(","set_add(","set_remove(","set_has(","set_size(","set_list(",
        "set_union(","set_inter(","set_diff(",

        // Кэш читаем и пишем короткими именами без тильды чтобы обращение укладывалось в одно слово
        "cache_set(","cache_get(","cache_del(","cache_has(",

        // Базу данных вызываем короткими именами без тильды чтобы команды движка писались короче
        "db_create(","db_insert(","db_select(","db_update(","db_delete(",
        "db_tables","db_columns(","db_drop(","db_query(","db_open","db_close",
        "db_backup(","db_restore(","db_query_row(","db_query_value(",
        "db_execute(","db_transaction","db_begin","db_commit","db_rollback",
        "db_last_insert_id","db_changes",

        // Шифрование и хэши вызываем короткими именами без тильды чтобы команды движка писались короче
        "md5(","sha256(","base64enc(","base64dec(","hexenc(","hexdec(",
        "randombytes(","encrypt(","decrypt(",

        // Изображения вызываем короткими именами без тильды чтобы команды движка писались короче
        "img_new(","img_setpixel(","img_pixel(","img_fill(",
        "img_size(","img_save(","img_load(",
        "img_grayscale(","img_invert(",

        // Звук вызываем короткими именами без тильды чтобы команды движка писались короче
        "snd_volume(","snd_sine(","snd_square(","snd_noise(","snd_melody(","snd_speak(",
        "snd_stop","snd_pause","snd_resume","snd_beep(",

        // Файлы читаем и пишем короткими именами без тильды чтобы вызовы выглядели как обычные слова
        "read(","write(","exs(","dir(","mkdir(","rmdir(","delfile(",
        "read_file(","write_file(","append_file(","file_exists(","is_file(","is_dir(","file_size(","file_modified(",
        "delete_file(","create_dir(","delete_dir(","list_dir(","copy_file(","move_file(",
        "file_ext(","file_stem(","file_name(","expand_user(","abs_path(",
        "readlines(","append(","cp(","mv(","rename(",
        "fsize(","ftype(","ftouch(",
        "isdir(","isfile(","modified(","glob(","tempdir",
        "readbytes(","readcsv(","writecsv(","readjson(","writejson(",
        "jsonparse(","jsonstringify(",
        "save(","load(","dcfg ","env(","run(","shell(",

        // Сеть и структуры вызываем короткими именами без тильды чтобы команды движка писались короче
        "fetch(","download(","ping(","post(","dns(",
        "http_get(","http_post(","http_put(","http_delete(",
        "tcp_listen(","tcp_connect(","tcp_send(","tcp_recv(",
        "udp_bind(","udp_send(","udp_recv(","udp_close",
        "struct ","enum ","new(","match(",
        "ws_server(","ws_connect(",
        "tau()","tau","phi()","phi","golden()","euler()","inf()",
        "clamp01(","percent(","is_int(","is_finite(","is_nan(","digit_sum(","digit_count(",
        "round_to(","signum(","mul_add(",
        "repeat_text(","contains_any(","count_letters(","count_digits(","count_words(",
        "first_char(","last_char(","suffix(","prefix(","truncate_text(","zfill(","sprintf(",
        "first(","last(","list_count_true(","list_swap(","list_average(","list_nunique(",
        "get_env(","set_env(","current_dir(","cpu_cores(","uptime(","shell_cmd(","clipboard_set(",
        "identity(","type_of(","noop(","noop","range_step(",
        "ring_new(","ring_add(","ring_get(","ring_size(",
        "heap_new(","heap_push(","heap_pop(","heap_peek(","heap_size(",
        "sort_fast(","binary_search(","lower_bound(","upper_bound(",
        "graph_new(","graph_edge(","graph_bfs(","graph_dfs(","graph_shortest(",
        "mem_size(","mem_copy(","mem_compare(",
        "approx(","random_gaussian(","random_poisson(","random_exponential(",
        "vector_len(","vector_normalize(",
        "matrix_add(","matrix_sub(","matrix_mul(","matrix_identity(","matrix_scale(","matrix_inv(",
        "fft(",
        "watch(","unwatch(","watches","call_stack","breakpoint(","continue",
        "sandbox_on","sandbox_off","sandbox_status","sandbox_allow(","sandbox_deny(","validate(",
        "trek_pkg","import_pkg(",

        // Систему и время вызываем короткими именами без тильды чтобы команды движка писались короче
        "setenv(","execoutput(","exec(","exit(","setclipboard(",
        "prockill(","procexists(",
        "os(","arch(","username","hostname","homedir",
        "nowms","time(","datetime","yearday(",
        "isleapyear(","daysinmonth(",
        "now(","today(","timestamp(","format_date(","from_timestamp(",
        "sleep(","wait(",
        "timer_start(","timer_elapsed(","timer_reset(","day_of_week(",
        "clr_rgb(","clr_hex(","clr_to_hex(","clr_red(","clr_green(","clr_blue(",
        "clr_brightness(","clr_grayscale(","clr_invert(","clr_blend(",

        // Журнал ведем короткими именами без тильды чтобы записи укладывались в одно слово
        "log_info(","log_warn(","log_error(","log_debug(",
        "log_to_file(","log_level(","log(",
        "loglevel(","logfile(",

        // Тесты и проверки вызываем короткими именами без тильды чтобы команды движка писались короче
        "assert(","assert_eq(","assert_ne(","assert_gt(","assert_lt(",
        "assert_true(","assert_false(","assert_throws(",
        "test(","test_report(","test_reset(","test_summary(",
        "benchmark(","benchmark_n(","typeof(","inspect(",

        // Алиасы вызываем короткими именами без тильды чтобы команды движка писались короче
        "alias(","unalias(","aliases(",

        // Проверки значений вызываем короткими именами без тильды чтобы команды движка писались короче
        "valid_email(","valid_url(","valid_ip(","valid_port(","valid_number(",
        "valid_date(","valid_uuid(","valid_length(","valid_range(",
        "valid_empty(","valid_hex_color(",
        "match_glob(","match_ip(","match_email(","match_url(",
        "extract_emails(","extract_numbers(","extract_urls(",

        // Красивый вывод таблиц прогресса и чисел делаем короткими именами без тильды чтобы оформление писалось коротко
        "progress(","progress_start(","progress_update(","progress_end(",
        "progress_percent(",
        "fmt_table(","fmt_json(","fmt_bytes(","fmt_time_human(",
        "fmt_number(","fmt_percentage(",
        "align_l(","align_r(","align_c(",
        "banner(","box(","hr(","chart_bar(",
        "spinner_step","spinner_clear",

        // Управление ходом условиями повторами и счетчиками делаем короткими именами без тильды чтобы команды писались короче
        "ternary(","when(","unless(","retry(","repeat(","swap(",
        "inc_by(","dec_by(","mul_var(","div_var(",
        "append_var(","prepend_var(",
        "measure(","bench_n(","tonum(","tostr(","tobool(",
        "type(","exists(",

        // Функции определяем и вызываем короткими словами без тильды чтобы запись оставалась компактной
        "def ","call ","ref ","exec ","local ","return",
        "fn(","call_fn(","curry(","compose(","pipe(","rec(",
        "import ","import(","eval(",

        // Сообщения пользователю пишем короткими именами без тильды чтобы вопросы и ответы укладывались в одно слово
        "error(","warn(","info(","success(","question(","confirm(","prompt(",

        // Настройки и хранилища вызываем короткими именами без тильды чтобы команды писались короче
        "config(","setconfig(",
        "ini_load(","ini_save(","ini_get(","ini_set(","ini_del(",
        "ini_keys(","ini_sections(",
        "kv_open(","kv_get(","kv_set(","kv_del(",

        // Переменные и историю смотрим короткими именами без тильды чтобы команды движка писались короче
        "vars(","funcs(","consts(","clear(","reset(","historyclear",
        "savehistory(","loadhistory(","backup(","restore(",

        // Адреса разбираем короткими именами без тильды чтобы команды движка писались короче
        "urlparse(","queryparse(","querystringify(",

        // Потоки и мьютексы вызываем короткими именами без тильды чтобы команды движка писались короче
        "thread(","threads(","thread_wait(","thread_stop(",
        "channel(","mutex(","mutex_lock(","mutex_unlock(",

        // Шаблоны и события вызываем короткими именами без тильды чтобы команды движка писались короче
        "template(","render(","template_var(",
        "on(","emit(","off(",

        // Спрайты вызываем короткими именами без тильды чтобы команды движка писались короче
        "sprite(","sprite_load(","sprite_size(","sprite_del(",
        "sprite_copy(","sprite_mirror(","sprite_rotate(",
        "draw_sprite(",

        // Анимации вызываем короткими именами без тильды чтобы команды движка писались короче
        "anim(","anim_speed(","play(","loop_anim(","stop_anim(",
        "anim_state(","anim_done(","anim_pingpong(",
        "anim_create(","anim_addframe(","anim_pause(",
        "anim_resume(","anim_loop(","anim_once(",
        "anim_frame(","anim_frames(","anim_current(","anim_remove(",

        // Кадры и календарное время смотрим короткими именами без тильды чтобы команды движка писались короче
        "fps(","delta(","framecount(",
        "year(","month(","day(","hour(","minute(","second(","sleepms(",
        "weekday(","pid(","ppid(","timestamp(",
        "date_add(","days_in_month(","is_leap(",
        "timer ","autosave ",

        // Карту мира вызываем короткими именами без тильды чтобы команды движка писались короче
        "loadmap(","tile(","settile(","drawmap_rect(",
        "map_save(","map_new(","map_fill(","map_replace(","map_count(",
        "map ","obj ","objmove ","objdel ","spawn ",

        // Камеру вызываем короткими именами без тильды чтобы команды движка писались короче
        "camera(","camera_follow(","viewport(",
        "world_to_screen(","screen_to_world(",
        "shake(","zoom(",

        // Счет и жизни показываем короткими именами без тильды чтобы команды движка писались короче
        "hud(","score(","score_add(","lives(","lives_add(",
        "hp(","hp_add(","xp(","xp_add(",

        // Меню и состояния игры вызываем короткими именами без тильды чтобы команды движка писались короче
        "menu(","menu_select(","menu_index(",
        "gamestate(","gamestate_is(","transition(","dialog(",
        "savegame(","loadgame(","deletegame(",

        // Эффекты и частицы вызываем короткими именами без тильды чтобы команды движка писались короче
        "particle(","particle_burst(","flash(","fade_in(","fade_out(",
        "screen_color(",

        // Столкновения проверяем короткими именами без тильды чтобы команды движка писались короче
        "collide(","collide_rect(","collide_circ(","collide_line(",
        "collide_grid(","collide_point_rect(","collide_point_circ(",

        // Игровой движок вызываем короткими именами без тильды чтобы команды писались короче
        "game_init(","game_quit","game_fps(","game_delta",
        "game_scene(","game_switch(","game_camera(",
        "game_camerazoom(","game_camerashake(",
        "game_score(","game_lives(","game_level(",
        "game_save(","game_load(","game_reset","game_run",

        // Символьный дисплей вызываем короткими именами без тильды чтобы команды писались короче
        "ldsc","ld(","ldm(","ldbox","ldtext","ldfill",
        "ldc(","ldg(","ldcolor(","ldsave(","ldload(",
        "ldsprite(","ldrect_fill(","ldcirc(","ldcirc_fill(",
        "ldscroll(","ldborder ","fill ","border ",

        // Команды kg не перечисляем здесь так как их перехватывает проверка на префикс kg выше и список не нужен

        // Экранные сокращения и звук вызываем короткими именами без тильды чтобы команды движка писались короче
        "cm(","al(","cur(","fg(","bg(","color ",
        "beep ","melody ","snd_beep(",
        "exporthelp","version","in(",
        "key(","waitkey(",

        // Справку вызываем короткими именами без тильды чтобы помощь и лицензия вызывались одним словом
        "help(","license(","credits(",
    ];

    for p in prefixes {
        if code.starts_with(p) {
            return Some(crate::main::eval::evaluate(vm, &format!("~{}", code)));
        }
    }

    // Буква p печатает текст а буква w ждет число чтобы короткие алиасы работали в одно слово
    // Строки здесь не склеиваем а вызываем готовые команды печати и ожидания чтобы не получать неизвестные имена
    if code.starts_with("p ") {
        return Some(crate::main::eval::evaluate(vm, &format!("~print({})", code[2..].trim())));
    }
    if code.starts_with("inp(") {
        return Some(crate::main::eval::evaluate(vm, &format!("~in{}", &code[3..])));
    }
    if let Some(rest) = code.strip_prefix("w ") {
        if rest.trim().parse::<f64>().is_ok() {
            return Some(crate::main::eval::evaluate(vm, &format!("~wait({})", rest.trim())));
        }
    }

    None
}