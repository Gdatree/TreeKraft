use crate::main::vm::{VM, Value};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static DL_LIBS: RefCell<HashMap<String, isize>> = RefCell::new(HashMap::new());
}

#[cfg(windows)]
mod win_dl {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryW(lpFileName: *const u16) -> *mut std::ffi::c_void;
        fn FreeLibrary(hModule: *mut std::ffi::c_void) -> i32;
    }
    pub fn dl_open(path: &str) -> Result<isize, String> {
        let w: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let h = unsafe { LoadLibraryW(w.as_ptr()) };
        if h.is_null() { Err(format!("dl_open failed: {}", path)) } else { Ok(h as isize) }
    }
    pub fn dl_close(h: isize) -> bool {
        unsafe { FreeLibrary(h as *mut std::ffi::c_void) != 0 }
    }
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~dl_open(") && code.ends_with(')') {
        let path = vm.eval_value(&code[9..code.len()-1]).ok()?.as_text()?;
        #[cfg(windows)]
        {
            match win_dl::dl_open(&path) {
                Ok(h) => {
                    DL_LIBS.with(|m| { m.borrow_mut().insert(path.clone(), h); });
                    vm.echo(&format!("loaded {}", path));
                    return Some(Ok(Value::Bool(true)));
                }
                Err(e) => return Some(Err(e)),
            }
        }
        #[cfg(not(windows))]
        {
            let _ = path;
            return Some(Err("dl_open: Windows only".to_string()));
        }
    } else if code.starts_with("~dl_call(") && code.ends_with(')') {
        return Some(Err("dl_call: calling arbitrary DLL signatures not implemented (dl_open/dl_close/dl_list work)".to_string()));
    } else if code.starts_with("~dl_close(") && code.ends_with(')') {
        let name = vm.eval_value(&code[10..code.len()-1]).ok()?.as_text()?;
        let h = DL_LIBS.with(|m| m.borrow_mut().remove(&name));
        match h {
            #[cfg(windows)]
            Some(h) => {
                let ok = win_dl::dl_close(h);
                vm.echo(&format!("unloaded {}: {}", name, ok));
                return Some(Ok(Value::Bool(ok)));
            }
            #[cfg(not(windows))]
            Some(_) => return Some(Err("dl_close: Windows only".to_string())),
            None => return Some(Err(format!("dl_close: '{}' not loaded", name))),
        }
    } else if code.starts_with("~dl_list") {
        let names: Vec<Value> = DL_LIBS.with(|m| m.borrow().keys().cloned().map(Value::Text).collect());
        let list = Value::List(names);
        vm.echo(list.display());
        Some(Ok(list))
    } else {
        None
    }
}
