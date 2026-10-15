// Модули команд TreeKraft
pub mod math;
pub mod lists;
pub mod sprites;
pub mod maps;
pub mod physics;
pub mod gameui;
pub mod display;
pub mod output;
pub mod auto;
pub mod net;
pub mod time;
pub mod crypto;
pub mod images;
pub mod sound;
pub mod debug;
pub mod fast;
pub mod strings;
pub mod regex;
pub mod units;
pub mod convert;
pub mod validator;
pub mod html;
pub mod compress;
pub mod algo;
pub mod window;
pub mod database;
pub mod server;
pub mod encoding;
pub mod process;
pub mod pattern;
pub mod audio;
pub mod tui;
pub mod ffi;
pub mod watch;
pub mod calc;
pub mod eng;
pub mod music;
pub mod textproc;
pub mod bio;
pub mod chem;
pub mod sport;
pub mod cook;
pub mod geo3;
pub mod agri;
pub mod vet;
pub mod fish;
pub mod datetime;
pub mod color;
pub mod fs;
pub mod tcp;
pub mod types;
pub mod websocket;
pub mod mega;
pub mod pkg;
pub mod stats;
pub mod iter;
pub mod json;

use crate::main::vm::VM;
use crate::main::vm::Value;

/// Тип функции-резолвера модуля команд
pub type Resolver = fn(&mut VM, &str) -> Option<Result<Value, String>>;

/// Массив всех резолверов для кешированной диспетчеризации
pub const RESOLVERS: &[Resolver] = &[
    math::resolve as Resolver,
    lists::resolve as Resolver,
    strings::resolve as Resolver,
    output::resolve as Resolver,
    display::resolve as Resolver,
    sprites::resolve as Resolver,
    maps::resolve as Resolver,
    physics::resolve as Resolver,
    gameui::resolve as Resolver,
    time::resolve as Resolver,
    images::resolve as Resolver,
    sound::resolve as Resolver,
    net::resolve as Resolver,
    crypto::resolve as Resolver,
    auto::resolve as Resolver,
    debug::resolve as Resolver,
    fast::resolve as Resolver,
    regex::resolve as Resolver,
    units::resolve as Resolver,
    convert::resolve as Resolver,
    validator::resolve as Resolver,
    html::resolve as Resolver,
    compress::resolve as Resolver,
    algo::resolve as Resolver,
    window::resolve as Resolver,
    database::resolve as Resolver,
    server::resolve as Resolver,
    encoding::resolve as Resolver,
    process::resolve as Resolver,
    pattern::resolve as Resolver,
    audio::resolve as Resolver,
    tui::resolve as Resolver,
    ffi::resolve as Resolver,
    watch::resolve as Resolver,
    calc::resolve as Resolver,
    eng::resolve as Resolver,
    music::resolve as Resolver,
    textproc::resolve as Resolver,
    bio::resolve as Resolver,
    chem::resolve as Resolver,
    sport::resolve as Resolver,
    cook::resolve as Resolver,
    geo3::resolve as Resolver,
    agri::resolve as Resolver,
    vet::resolve as Resolver,
    fish::resolve as Resolver,
    datetime::resolve as Resolver,
    color::resolve as Resolver,
    fs::resolve as Resolver,
    tcp::resolve as Resolver,
    types::resolve as Resolver,
    websocket::resolve as Resolver,
    mega::resolve as Resolver,
    pkg::resolve as Resolver,
    stats::resolve as Resolver,
    iter::resolve as Resolver,
    json::resolve as Resolver,
];

