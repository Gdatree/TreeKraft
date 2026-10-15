use crate::main::vm::{VM, Value};
use std::cell::RefCell;

struct WinState {
    window: Option<minifb::Window>,
    buffer: Vec<u32>,
    w: usize,
    h: usize,
    tx: f64,
    ty: f64,
    rot_deg: f64,
    scale_x: f64,
    scale_y: f64,
    particles: Vec<(f64, f64, f64, f64, u32, i32)>,
}

thread_local! {
    static WIN: RefCell<WinState> = const { RefCell::new(WinState { window: None, buffer: Vec::new(), w: 800, h: 600, tx: 0.0, ty: 0.0, rot_deg: 0.0, scale_x: 1.0, scale_y: 1.0, particles: Vec::new() }) };
}

fn xform(x: f64, y: f64, tx: f64, ty: f64, rot: f64, sx: f64, sy: f64) -> (f64, f64) {
    let rad = rot.to_radians();
    let (cos, sin) = (rad.cos(), rad.sin());
    let (mx, my) = (x * sx, y * sy);
    (mx * cos - my * sin + tx, mx * sin + my * cos + ty)
}

// Встроенный шрифт 5 на 7, чтобы рисовать текст без картинок
const FONT5X7: [u8; 672] = [
    0x00,0x00,0x00,0x00,0x00,0x00,0x00,
    0x5F,0x00,0x00,0x00,0x00,0x00,0x00,
    0x00,0x03,0x00,0x03,0x00,0x00,0x00,
    0x14,0x7F,0x14,0x7F,0x14,0x00,0x00,
    0x24,0x2A,0x7F,0x2A,0x12,0x00,0x00,
    0x23,0x13,0x08,0x64,0x62,0x00,0x00,
    0x36,0x49,0x55,0x22,0x50,0x00,0x00,
    0x00,0x05,0x03,0x00,0x00,0x00,0x00,
    0x00,0x1C,0x22,0x41,0x00,0x00,0x00,
    0x00,0x41,0x22,0x1C,0x00,0x00,0x00,
    0x08,0x2A,0x1C,0x2A,0x08,0x00,0x00,
    0x08,0x08,0x3E,0x08,0x08,0x00,0x00,
    0x00,0x50,0x30,0x00,0x00,0x00,0x00,
    0x08,0x08,0x08,0x08,0x08,0x00,0x00,
    0x00,0x60,0x60,0x00,0x00,0x00,0x00,
    0x20,0x10,0x08,0x04,0x02,0x00,0x00,
    0x3E,0x51,0x49,0x45,0x3E,0x00,0x00,
    0x00,0x42,0x7F,0x40,0x00,0x00,0x00,
    0x42,0x61,0x51,0x49,0x46,0x00,0x00,
    0x21,0x41,0x45,0x4B,0x31,0x00,0x00,
    0x18,0x14,0x12,0x7F,0x10,0x00,0x00,
    0x27,0x45,0x45,0x45,0x39,0x00,0x00,
    0x3C,0x4A,0x49,0x49,0x30,0x00,0x00,
    0x01,0x71,0x09,0x05,0x03,0x00,0x00,
    0x36,0x49,0x49,0x49,0x36,0x00,0x00,
    0x06,0x49,0x49,0x29,0x1E,0x00,0x00,
    0x00,0x36,0x36,0x00,0x00,0x00,0x00,
    0x00,0x56,0x36,0x00,0x00,0x00,0x00,
    0x00,0x08,0x14,0x22,0x41,0x00,0x00,
    0x14,0x14,0x14,0x14,0x14,0x00,0x00,
    0x41,0x22,0x14,0x08,0x00,0x00,0x00,
    0x02,0x01,0x51,0x09,0x06,0x00,0x00,
    0x32,0x49,0x79,0x41,0x3E,0x00,0x00,
    0x7E,0x11,0x11,0x11,0x7E,0x00,0x00,
    0x7F,0x49,0x49,0x49,0x36,0x00,0x00,
    0x3E,0x41,0x41,0x41,0x22,0x00,0x00,
    0x7F,0x41,0x41,0x22,0x1C,0x00,0x00,
    0x7F,0x49,0x49,0x49,0x41,0x00,0x00,
    0x7F,0x09,0x09,0x01,0x01,0x00,0x00,
    0x3E,0x41,0x41,0x51,0x32,0x00,0x00,
    0x7F,0x08,0x08,0x08,0x7F,0x00,0x00,
    0x00,0x41,0x7F,0x41,0x00,0x00,0x00,
    0x20,0x40,0x41,0x3F,0x01,0x00,0x00,
    0x7F,0x08,0x14,0x22,0x41,0x00,0x00,
    0x7F,0x40,0x40,0x40,0x40,0x00,0x00,
    0x7F,0x02,0x04,0x02,0x7F,0x00,0x00,
    0x7F,0x04,0x08,0x10,0x7F,0x00,0x00,
    0x3E,0x41,0x41,0x41,0x3E,0x00,0x00,
    0x7F,0x09,0x09,0x09,0x06,0x00,0x00,
    0x3E,0x41,0x51,0x21,0x5E,0x00,0x00,
    0x7F,0x09,0x19,0x29,0x46,0x00,0x00,
    0x46,0x49,0x49,0x49,0x31,0x00,0x00,
    0x01,0x01,0x7F,0x01,0x01,0x00,0x00,
    0x3F,0x40,0x40,0x40,0x3F,0x00,0x00,
    0x1F,0x20,0x40,0x20,0x1F,0x00,0x00,
    0x3F,0x40,0x38,0x40,0x3F,0x00,0x00,
    0x63,0x14,0x08,0x14,0x63,0x00,0x00,
    0x03,0x04,0x78,0x04,0x03,0x00,0x00,
    0x61,0x51,0x49,0x45,0x43,0x00,0x00,
    0x00,0x00,0x7F,0x41,0x41,0x00,0x00,
    0x02,0x04,0x08,0x10,0x20,0x00,0x00,
    0x41,0x41,0x7F,0x00,0x00,0x00,0x00,
    0x04,0x02,0x01,0x02,0x04,0x00,0x00,
    0x40,0x40,0x40,0x40,0x40,0x00,0x00,
    0x00,0x01,0x02,0x04,0x00,0x00,0x00,
    0x20,0x54,0x54,0x54,0x78,0x00,0x00,
    0x7F,0x48,0x44,0x44,0x38,0x00,0x00,
    0x38,0x44,0x44,0x44,0x20,0x00,0x00,
    0x38,0x44,0x44,0x48,0x7F,0x00,0x00,
    0x38,0x54,0x54,0x54,0x18,0x00,0x00,
    0x08,0x7E,0x09,0x01,0x02,0x00,0x00,
    0x0C,0x52,0x52,0x52,0x3E,0x00,0x00,
    0x7F,0x08,0x04,0x04,0x78,0x00,0x00,
    0x00,0x44,0x7D,0x40,0x00,0x00,0x00,
    0x20,0x40,0x44,0x3D,0x00,0x00,0x00,
    0x7F,0x10,0x28,0x44,0x00,0x00,0x00,
    0x00,0x41,0x7F,0x40,0x00,0x00,0x00,
    0x7C,0x04,0x18,0x04,0x78,0x00,0x00,
    0x7C,0x08,0x04,0x04,0x78,0x00,0x00,
    0x38,0x44,0x44,0x44,0x38,0x00,0x00,
    0x7C,0x14,0x14,0x14,0x08,0x00,0x00,
    0x08,0x14,0x14,0x18,0x7C,0x00,0x00,
    0x7C,0x08,0x04,0x04,0x08,0x00,0x00,
    0x48,0x54,0x54,0x54,0x20,0x00,0x00,
    0x04,0x3F,0x44,0x40,0x20,0x00,0x00,
    0x3C,0x40,0x40,0x20,0x7C,0x00,0x00,
    0x1C,0x20,0x40,0x20,0x1C,0x00,0x00,
    0x3C,0x40,0x30,0x40,0x3C,0x00,0x00,
    0x44,0x28,0x10,0x28,0x44,0x00,0x00,
    0x0C,0x50,0x50,0x50,0x3C,0x00,0x00,
    0x44,0x64,0x54,0x4C,0x44,0x00,0x00,
    0x00,0x08,0x36,0x41,0x00,0x00,0x00,
    0x00,0x00,0x7F,0x00,0x00,0x00,0x00,
    0x00,0x41,0x36,0x08,0x00,0x00,0x00,
    0x08,0x08,0x2A,0x1C,0x08,0x00,0x00,
    0x08,0x1C,0x2A,0x08,0x08,0x00,0x00
];

fn draw_char(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, ch: char, c: u32) {
    let idx = ch as usize;
    if !(32..=127).contains(&idx) { return; }
    let base = (idx - 32) * 7;
    for row in 0..7 {
        let line = FONT5X7[base + row];
        for col in 0..5 {
            if line & (1 << (4 - col)) != 0 {
                put(buf, w, h, x + col as i32, y + row as i32, c);
            }
        }
    }
}

fn draw_text(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, text: &str, c: u32) {
    let mut cx = x;
    for ch in text.chars() {
        if ch == '\n' { cx = x; continue; }
        draw_char(buf, w, h, cx, y, ch, c);
        cx += 6;
    }
}

fn color(r: f64, g: f64, b: f64) -> u32 {
    (r as u8) as u32 | ((g as u8) as u32) << 8 | ((b as u8) as u32) << 16 | 0xFF000000
}

fn with_buf<F>(f: F) where F: FnOnce(&mut [u32], usize, usize) {
    let (w, h) = WIN.with(|win| { let ws = win.borrow(); (ws.w, ws.h) });
    let buf_ptr = WIN.with(|win| { let mut ws = win.borrow_mut(); &mut ws.buffer as *mut Vec<u32> });
    let buf = unsafe { &mut *buf_ptr };
    f(buf, w, h);
}

fn key_name(k: minifb::Key) -> &'static str {
    use minifb::Key::*;
    match k { W=>"w",A=>"a",S=>"s",D=>"d",Q=>"q",E=>"e",R=>"r",T=>"t",Y=>"y",U=>"u",I=>"i",O=>"o",P=>"p",F=>"f",G=>"g",H=>"h",J=>"j",K=>"k",L=>"l",Z=>"z",X=>"x",C=>"c",V=>"v",B=>"b",N=>"n",M=>"m",Key0=>"0",Key1=>"1",Key2=>"2",Key3=>"3",Key4=>"4",Key5=>"5",Key6=>"6",Key7=>"7",Key8=>"8",Key9=>"9",Space=>"space",Enter=>"enter",Escape=>"esc",Tab=>"tab",Backspace=>"backspace",Delete=>"delete",Home=>"home",End=>"end",Up=>"up",Down=>"down",Left=>"left",Right=>"right",LeftShift=>"lshift",RightShift=>"rshift",LeftCtrl=>"lctrl",RightCtrl=>"rctrl",LeftAlt=>"lalt",RightAlt=>"ralt",F1=>"f1",F2=>"f2",F3=>"f3",F4=>"f4",F5=>"f5",F6=>"f6",F7=>"f7",F8=>"f8",F9=>"f9",F10=>"f10",F11=>"f11",F12=>"f12",_=>"unknown"}
}

fn parse_key(s: &str) -> Option<minifb::Key> {
    use minifb::Key::*;
    Some(match s {"w"=>W,"a"=>A,"s"=>S,"d"=>D,"q"=>Q,"e"=>E,"r"=>R,"t"=>T,"y"=>Y,"u"=>U,"i"=>I,"o"=>O,"p"=>P,"f"=>F,"g"=>G,"h"=>H,"j"=>J,"k"=>K,"l"=>L,"z"=>Z,"x"=>X,"c"=>C,"v"=>V,"b"=>B,"n"=>N,"m"=>M,"0"=>Key0,"1"=>Key1,"2"=>Key2,"3"=>Key3,"4"=>Key4,"5"=>Key5,"6"=>Key6,"7"=>Key7,"8"=>Key8,"9"=>Key9,"space"=>Space,"enter"=>Enter,"esc"|"escape"=>Escape,"tab"=>Tab,"backspace"=>Backspace,"delete"=>Delete,"home"=>Home,"end"=>End,"up"=>Up,"down"=>Down,"left"=>Left,"right"=>Right,"lshift"=>LeftShift,"rshift"=>RightShift,"lctrl"=>LeftCtrl,"rctrl"=>RightCtrl,"lalt"=>LeftAlt,"ralt"=>RightAlt,"f1"=>F1,"f2"=>F2,"f3"=>F3,"f4"=>F4,"f5"=>F5,"f6"=>F6,"f7"=>F7,"f8"=>F8,"f9"=>F9,"f10"=>F10,"f11"=>F11,"f12"=>F12,_=>return None})
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    // Ниже управление окном, чтобы открывать, закрывать и обновлять кадр
    if code.starts_with("~win_open(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 3 {
            let w = vm.eval_expr(&p[0]).ok()? as usize;
            let h = vm.eval_expr(&p[1]).ok()? as usize;
            let title = vm.eval_value(&p[2]).ok()?.as_text()?;
            match minifb::Window::new(&title, w, h, minifb::WindowOptions::default()) {
                Ok(window) => {
                    WIN.with(|win| { let mut ws = win.borrow_mut(); ws.window = Some(window); ws.buffer = vec![0xFF333333u32; w*h]; ws.w = w; ws.h = h; });
                    println!("Window opened: {}x{}", w, h); Some(Ok(Value::None))
                }
                Err(e) => Some(Err(e.to_string())),
            }
        } else { None }
    } else if code.starts_with("~win_close") && (code.len() == 10 || (code.starts_with("~win_close(") && code.ends_with(')'))) {
        WIN.with(|win| win.borrow_mut().window = None); println!("Window closed"); Some(Ok(Value::None))
    } else if code.starts_with("~win_running") && (code.len() == 12 || (code.starts_with("~win_running(") && code.ends_with(')'))) {
        let r = WIN.with(|win| win.borrow().window.as_ref().map(|w| w.is_open()).unwrap_or(false));
        vm.echo(r); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~win_wait_close") && (code.len() == 15 || (code.starts_with("~win_wait_close(") && code.ends_with(')'))) {
        loop {
            let open = WIN.with(|win| win.borrow().window.as_ref().map(|w| w.is_open()).unwrap_or(false));
            if !open { break; }
            WIN.with(|win| {
                let (buf, w, h) = { let ws = win.borrow(); (ws.buffer.clone(), ws.w, ws.h) };
                if let Some(ref mut wnd) = win.borrow_mut().window {
                    if wnd.is_open() { wnd.update_with_buffer(&buf, w, h).ok(); }
                }
            });
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        Some(Ok(Value::None))
    } else if code.starts_with("~win_update") && (code.len() == 11 || (code.starts_with("~win_update(") && code.ends_with(')'))) {
        let (buf, w, h) = WIN.with(|win| { let ws = win.borrow(); (ws.buffer.clone(), ws.w, ws.h) });
        WIN.with(|win| { if let Some(ref mut wnd) = win.borrow_mut().window { if wnd.is_open() { wnd.update_with_buffer(&buf, w, h).ok(); } } }); Some(Ok(Value::None))
    } else if code.starts_with("~win_clear(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 3 {
            let c = color(vm.eval_expr(&p[0]).ok()?, vm.eval_expr(&p[1]).ok()?, vm.eval_expr(&p[2]).ok()?);
            WIN.with(|win| { for p in win.borrow_mut().buffer.iter_mut() { *p = c; } }); Some(Ok(Value::None))
        } else { None }

    // Ниже рисование точки, чтобы красить один пиксель в нужный цвет
    } else if code.starts_with("~win_pixel(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 5 {
            let (x,y,c) = (vm.eval_expr(&p[0]).ok()? as i32, vm.eval_expr(&p[1]).ok()? as i32, color(vm.eval_expr(&p[2]).ok()?, vm.eval_expr(&p[3]).ok()?, vm.eval_expr(&p[4]).ok()?));
            with_buf(|buf, w, h| { if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 { buf[y as usize * w + x as usize] = c; } });
            Some(Ok(Value::None))
        } else { None }

    // Ниже рисование линии, чтобы соединять две точки прямой
    } else if code.starts_with("~win_line(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 7 {
            let (x1,y1,x2,y2,c) = (vm.eval_expr(&p[0]).ok()? as i32, vm.eval_expr(&p[1]).ok()? as i32, vm.eval_expr(&p[2]).ok()? as i32, vm.eval_expr(&p[3]).ok()? as i32, color(vm.eval_expr(&p[4]).ok()?, vm.eval_expr(&p[5]).ok()?, vm.eval_expr(&p[6]).ok()?));
            with_buf(|buf, w, h| {
                let (dx, dy) = ((x2-x1).abs(), -(y2-y1).abs()); let (sx, sy) = (if x1<x2{1}else{-1}, if y1<y2{1}else{-1}); let (mut cx,mut cy,mut err) = (x1,y1,dx+dy);
                loop { if cx>=0&&cx<w as i32&&cy>=0&&cy<h as i32{buf[cy as usize*w+cx as usize]=c;} if cx==x2&&cy==y2{break;} let e2=2*err; if e2>=dy{err+=dy;cx+=sx;} if e2<=dx{err+=dx;cy+=sy;} }
            }); Some(Ok(Value::None))
        } else { None }

    // Ниже прямоугольники, чтобы делать контур и заливку для рамок
    } else if code.starts_with("~win_rect(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 7 {
            let (x,y,rw,rh,c) = (vm.eval_expr(&p[0]).ok()? as i32, vm.eval_expr(&p[1]).ok()? as i32, vm.eval_expr(&p[2]).ok()? as i32, vm.eval_expr(&p[3]).ok()? as i32, color(vm.eval_expr(&p[4]).ok()?, vm.eval_expr(&p[5]).ok()?, vm.eval_expr(&p[6]).ok()?));
            with_buf(|buf, w, h| { for i in 0..rw { put(buf,w,h,x+i,y,c); put(buf,w,h,x+i,y+rh-1,c); } for i in 0..rh { put(buf,w,h,x,y+i,c); put(buf,w,h,x+rw-1,y+i,c); } });
            Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~win_fill_rect(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ','); if p.len() >= 7 {
            let (x,y,rw,rh,c) = (vm.eval_expr(&p[0]).ok()? as i32, vm.eval_expr(&p[1]).ok()? as i32, vm.eval_expr(&p[2]).ok()? as i32, vm.eval_expr(&p[3]).ok()? as i32, color(vm.eval_expr(&p[4]).ok()?, vm.eval_expr(&p[5]).ok()?, vm.eval_expr(&p[6]).ok()?));
            with_buf(|buf, w, h| { for dy in 0..rh { for dx in 0..rw { put(buf,w,h,x+dx,y+dy,c); } } });
            Some(Ok(Value::None))
        } else { None }

    // Ниже круги, чтобы делать контур и заливку для простых фигур
    } else if code.starts_with("~win_circle(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 6 {
            let (cx,cy,r,c) = (vm.eval_expr(&p[0]).ok()? as i32, vm.eval_expr(&p[1]).ok()? as i32, vm.eval_expr(&p[2]).ok()? as i32, color(vm.eval_expr(&p[3]).ok()?, vm.eval_expr(&p[4]).ok()?, vm.eval_expr(&p[5]).ok()?));
            with_buf(|buf, w, h| { let (mut x, mut y, mut d) = (0i32, r, 3-2*r); while y>=x { for &(px,py) in &[(cx+x,cy+y),(cx-x,cy+y),(cx+x,cy-y),(cx-x,cy-y),(cx+y,cy+x),(cx-y,cy+x),(cx+y,cy-x),(cx-y,cy-x)] { put(buf,w,h,px,py,c); } if d<0 { d+=4*x+6; } else { d+=4*(x-y)+10; y-=1; } x+=1; } });
            Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~win_fill_circle(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[17..code.len()-1], ','); if p.len() >= 6 {
            let (cx,cy,r,c) = (vm.eval_expr(&p[0]).ok()? as i32, vm.eval_expr(&p[1]).ok()? as i32, vm.eval_expr(&p[2]).ok()? as i32, color(vm.eval_expr(&p[3]).ok()?, vm.eval_expr(&p[4]).ok()?, vm.eval_expr(&p[5]).ok()?));
            with_buf(|buf, w, h| { for dy in -r..=r { for dx in -r..=r { if dx*dx+dy*dy<=r*r { put(buf,w,h,cx+dx,cy+dy,c); } } } });
            Some(Ok(Value::None))
        } else { None }

    // Ниже ввод, чтобы опрашивать клавиши и мышь для управления
    } else if code.starts_with("~win_key(") && code.ends_with(')') || code == "~win_key" {
        let r = WIN.with(|win| win.borrow().window.as_ref().and_then(|w| w.get_keys().first().map(|&k| key_name(k).to_string())));
        if let Some(k) = r { vm.echo(&k); Some(Ok(Value::Text(k))) } else { Some(Ok(Value::None)) }
    } else if code.starts_with("~win_key_down(") && code.ends_with(')') {
        let key_str = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text()?;
        let r = WIN.with(|win| win.borrow().window.as_ref().and_then(|w| parse_key(&key_str).map(|k| w.is_key_down(k))).unwrap_or(false));
        vm.echo(r); Some(Ok(Value::Bool(r)))
    } else if code.starts_with("~win_mouse_x") && (code.len() == 12 || (code.starts_with("~win_mouse_x(") && code.ends_with(')'))) {
        let r = WIN.with(|win| win.borrow().window.as_ref().and_then(|w| w.get_mouse_pos(minifb::MouseMode::Clamp)).map(|(x,_)| x as f64).unwrap_or(0.0));
        vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~win_mouse_y") && (code.len() == 12 || (code.starts_with("~win_mouse_y(") && code.ends_with(')'))) {
        let r = WIN.with(|win| win.borrow().window.as_ref().and_then(|w| w.get_mouse_pos(minifb::MouseMode::Clamp)).map(|(_,y)| y as f64).unwrap_or(0.0));
        vm.echo(Value::fmt_num(r)); Some(Ok(Value::Number(r)))
    } else if code.starts_with("~win_mouse_btn(") && code.ends_with(')') {
        let n = vm.eval_expr(&code[15..code.len()-1]).ok()? as usize;
        let btn = [minifb::MouseButton::Left, minifb::MouseButton::Middle, minifb::MouseButton::Right][n.min(2)];
        let r = WIN.with(|win| win.borrow().window.as_ref().map(|w| w.get_mouse_down(btn)).unwrap_or(false));
        vm.echo(r); Some(Ok(Value::Bool(r)))

    // Ниже картинки, чтобы класть готовое изображение на экран
    } else if code.starts_with("~win_img(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[9..code.len()-1], ','); if p.len() >= 3 {
            let name = p[0].trim().to_string(); let dx = vm.eval_expr(&p[1]).ok()? as i32; let dy = vm.eval_expr(&p[2]).ok()? as i32;
            if let Some((iw, ih, ref pixels)) = vm.images.get(&name) {
                let iw = *iw; let ih = *ih;
                for iy in 0..ih { for ix in 0..iw {
                    let (r,g,b) = pixels[iy*iw+ix];
                    let px = dx + ix as i32; let py = dy + iy as i32;
                    let c = color(r as f64, g as f64, b as f64);
                    with_buf(|buf, w, h| { if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 { buf[py as usize * w + px as usize] = c; } });
                } }
                Some(Ok(Value::None))
            } else { Some(Err("Image not found".to_string())) }
        } else { None }

    // Ниже текст в окне, чтобы писать буквы поверх графики
    } else if code.starts_with("~win_text(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 3 {
            let x = vm.eval_expr(&p[0]).ok()? as i32;
            let y = vm.eval_expr(&p[1]).ok()? as i32;
            let text = vm.eval_value(&p[2]).ok()?.as_text().unwrap_or_default();
            let c = if p.len() >= 6 { color(vm.eval_expr(&p[3]).ok()?, vm.eval_expr(&p[4]).ok()?, vm.eval_expr(&p[5]).ok()?) } else { 0xFFFFFFFF };
            let (tx, ty, rot, sx, sy) = WIN.with(|win| { let ws = win.borrow(); (ws.tx, ws.ty, ws.rot_deg, ws.scale_x, ws.scale_y) });
            let (fx, fy) = xform(x as f64, y as f64, tx, ty, rot, sx, sy);
            with_buf(|buf, w, h| draw_text(buf, w, h, fx as i32, fy as i32, &text, c));
            Some(Ok(Value::None))
        } else { None }

    // Ниже спрайты, чтобы рисовать мелкие фигурки из символов
    } else if code.starts_with("~win_sprite(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() >= 3 {
            let name = p[0].trim().to_string();
            let x = vm.eval_expr(&p[1]).ok()? as i32;
            let y = vm.eval_expr(&p[2]).ok()? as i32;
            if let Some(sprite) = vm.sprites.get(&name) {
                let (tx, ty, rot, sx, sy) = WIN.with(|win| { let ws = win.borrow(); (ws.tx, ws.ty, ws.rot_deg, ws.scale_x, ws.scale_y) });
                for (i, row) in sprite.iter().enumerate() {
                    for (j, ch) in row.chars().enumerate() {
                        if ch == '.' || ch == ' ' { continue; }
                        let c = match ch {
                            '#' => 0xFFFFFFFFu32, '@' => 0xFF555555, '*' => 0xFFFFFFFF,
                            'r' => 0xFF0000FF, 'R' => 0xFF0000FF, 'g' => 0xFF00FF00, 'G' => 0xFF00FF00,
                            'b' => 0xFFFF0000, 'B' => 0xFFFF0000, 'y' => 0xFF00FFFF, 'Y' => 0xFF00FFFF,
                            'o' => 0xFF007FFF, 'O' => 0xFF007FFF, 'c' => 0xFFFFFF00, 'C' => 0xFFFFFF00,
                            'm' => 0xFFFF00FF, 'M' => 0xFFFF00FF, 'p' => 0xFFFF00AA, 'w' => 0xFFFFFFFF,
                            _ => 0xFFFFFFFF,
                        };
                        let (fx, fy) = xform((x + j as i32) as f64, (y + i as i32) as f64, tx, ty, rot, sx, sy);
                        with_buf(|buf, w, h| put(buf, w, h, fx as i32, fy as i32, c));
                    }
                }
                Some(Ok(Value::None))
            } else { Some(Err(format!("Sprite '{}' not found", name))) }
        } else { None }

    // Ниже снимок экрана, чтобы сохранять кадр в файл
    } else if code.starts_with("~win_screenshot(") && code.ends_with(')') {
        let path = vm.eval_value(&code[16..code.len()-1]).ok()?.as_text()?;
        let (buf, w, h) = WIN.with(|win| { let ws = win.borrow(); (ws.buffer.clone(), ws.w, ws.h) });
        let mut img = image::RgbaImage::new(w as u32, h as u32);
        for y in 0..h { for x in 0..w {
            let px = buf[y * w + x];
            img.put_pixel(x as u32, y as u32, image::Rgba([(px & 0xFF) as u8, ((px >> 8) & 0xFF) as u8, ((px >> 16) & 0xFF) as u8, 255]));
        } }
        match img.save(&path) { Ok(_) => { println!("Screenshot saved: {}", path); Some(Ok(Value::Bool(true))) } Err(e) => Some(Err(e.to_string())) }

    // Ниже частицы, чтобы делать искры и дым для простых эффектов
    } else if code.starts_with("~win_particles(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ',');
        if p.len() >= 3 {
            let x = vm.eval_expr(&p[0]).ok()? as f64;
            let y = vm.eval_expr(&p[1]).ok()? as f64;
            let n = vm.eval_expr(&p[2]).ok()? as usize;
            let spread = if p.len() >= 4 { vm.eval_expr(&p[3]).ok()? } else { 100.0 };
            let speed = if p.len() >= 5 { vm.eval_expr(&p[4]).ok()? } else { 2.0 };
            let life = if p.len() >= 6 { vm.eval_expr(&p[5]).ok()? as i32 } else { 60 };
            let c = if p.len() >= 9 { color(vm.eval_expr(&p[6]).ok()?, vm.eval_expr(&p[7]).ok()?, vm.eval_expr(&p[8]).ok()?) } else { 0xFFFFFFFF };
            WIN.with(|win| {
                let mut ws = win.borrow_mut();
                for _ in 0..n {
                    let ang = rand::random::<f64>() * std::f64::consts::TAU;
                    let v = rand::random::<f64>() * speed;
                    ws.particles.push((x, y, ang.cos() * v, ang.sin() * v, c, life));
                }
            });
            Some(Ok(Value::None))
        } else { None }

    // Ниже сдвиг и поворот, чтобы двигать всё рисование сразу
    } else if code.starts_with("~win_rotate(") && code.ends_with(')') {
        let deg = vm.eval_expr(&code[12..code.len()-1]).ok()?;
        WIN.with(|win| win.borrow_mut().rot_deg = deg);
        Some(Ok(Value::None))
    } else if code.starts_with("~win_scale(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 1 {
            let sx = vm.eval_expr(&p[0]).ok()?;
            let sy = if p.len() >= 2 { vm.eval_expr(&p[1]).ok()? } else { sx };
            WIN.with(|win| { let mut ws = win.borrow_mut(); ws.scale_x = sx; ws.scale_y = sy; });
            Some(Ok(Value::None))
        } else { None }
    } else if code.starts_with("~win_translate(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ',');
        if p.len() >= 1 {
            let tx = vm.eval_expr(&p[0]).ok()?;
            let ty = if p.len() >= 2 { vm.eval_expr(&p[1]).ok()? } else { 0.0 };
            WIN.with(|win| { let mut ws = win.borrow_mut(); ws.tx = tx; ws.ty = ty; });
            Some(Ok(Value::None))
        } else { None }
    } else if code == "~win_reset_transform" || code.starts_with("~win_reset_transform(") {
        WIN.with(|win| { let mut ws = win.borrow_mut(); ws.tx = 0.0; ws.ty = 0.0; ws.rot_deg = 0.0; ws.scale_x = 1.0; ws.scale_y = 1.0; });
        Some(Ok(Value::None))

    // Ниже шаг частиц, чтобы двигать их при каждом кадре для оживления
    } else if code.starts_with("~win_particles_step(") && code.ends_with(')') {
        WIN.with(|win| {
            let mut ws = win.borrow_mut();
            let (buf_w, buf_h) = (ws.w, ws.h);
            for (px, py, vx, vy, c, life) in ws.particles.iter_mut() {
                *px += *vx; *py += *vy;
                *vx *= 0.98; *vy = *vy * 0.98 + 0.05;
                *life -= 1;
            }
            ws.particles.retain(|&(_, _, _, _, _, life)| life > 0);
            let particles = ws.particles.clone();
            drop(ws);
            for (px, py, _, _, c, _) in &particles {
                with_buf(|buf, w, h| put(buf, w, h, *px as i32, *py as i32, *c));
            }
        });
        Some(Ok(Value::None))
    } else {
        None
    }
}

#[inline]
fn put(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, c: u32) {
    if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
        buf[y as usize * w + x as usize] = c;
    }
}
