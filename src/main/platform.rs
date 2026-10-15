// Платформозависимые модули, ввод с клавиатуры и звук.
// Выделено из vm.rs чтобы держать unsafe код платформы в одном месте.

#[cfg(windows)]
pub mod win_key {
    unsafe extern "C" {
        fn _kbhit() -> i32;
        fn _getch() -> i32;
    }
    pub fn key() -> Option<char> {
        unsafe {
            if _kbhit() != 0 { Some(_getch() as u8 as char) } else { None }
        }
    }
    pub fn getch() -> char { unsafe { _getch() as u8 as char } }
}
#[cfg(not(windows))]
pub mod win_key {
    pub fn key() -> Option<char> { None }
    pub fn getch() -> char { '\0' }
}

#[cfg(windows)]
pub mod win_sound {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn Beep(f: u32, d: u32) -> i32;
    }
    #[link(name = "winmm")]
    unsafe extern "system" {
        fn PlaySoundW(pszSound: *const u16, hmod: *mut std::ffi::c_void, fdwSound: u32) -> i32;
    }
    pub fn beep(f: u32, ms: u32) { unsafe { Beep(f, ms); } }
    const SND_FILENAME: u32 = 0x00020000;
    const SND_ASYNC: u32 = 0x00000001;
    /// Проиграть WAV-файл асинхронно (winmm, без зависимостей). Только WAV.
    pub fn play_wav(path: &str) -> Result<(), String> {
        if !std::path::Path::new(path).exists() {
            return Err(format!("audio file not found: {}", path));
        }
        let w: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let ok = unsafe { PlaySoundW(w.as_ptr(), std::ptr::null_mut(), SND_FILENAME | SND_ASYNC) };
        if ok == 0 { Err("PlaySound failed (needs WAV file)".to_string()) } else { Ok(()) }
    }
    /// Остановить текущее воспроизведение.
    pub fn stop_wav() { unsafe { PlaySoundW(std::ptr::null(), std::ptr::null_mut(), 0); } }
}
#[cfg(not(windows))]
pub mod win_sound {
    pub fn beep(_f: u32, _ms: u32) {}
    pub fn play_wav(_path: &str) -> Result<(), String> { Err("WAV playback: Windows only".to_string()) }
    pub fn stop_wav() {}
}
