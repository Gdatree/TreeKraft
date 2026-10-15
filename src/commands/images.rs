use crate::main::vm::{VM, Value};
use image::{GenericImageView, RgbImage, Rgb, ImageFormat};

fn to_img(w: usize, h: usize, pixels: &[(u8,u8,u8)]) -> RgbImage {
    let mut img = RgbImage::new(w as u32, h as u32);
    for y in 0..h { for x in 0..w { let (r,g,b) = pixels[y*w+x]; img.put_pixel(x as u32, y as u32, Rgb([r,g,b])); } }
    img
}
fn from_img(img: &RgbImage) -> (usize, usize, Vec<(u8,u8,u8)>) {
    let (w,h) = img.dimensions(); let mut pixels = Vec::with_capacity((w*h) as usize);
    for y in 0..h { for x in 0..w { let p = img.get_pixel(x,y); pixels.push((p[0],p[1],p[2])); } }
    (w as usize, h as usize, pixels)
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~img_load_any(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim().to_string(); let path = VM::tq(&p[1]);
            match image::open(&path) { Ok(img) => { let rgb = img.to_rgb8(); let (w,h,pixels) = from_img(&rgb); println!("Image '{}' loaded ({}x{})", name, w, h); vm.images.insert(name, (w,h,pixels)); Some(Ok(Value::None)) } Err(e) => Some(Err(e.to_string())) }
        } else { None }
    } else if code.starts_with("~img_save_any(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim(); let path = VM::tq(&p[1]);
            if let Some(&(w,h,ref pixels)) = vm.images.get(name) { let img = to_img(w,h,pixels);
                let ext = std::path::Path::new(&path).extension().and_then(|e| e.to_str()).unwrap_or("png");
                let fmt = match ext.to_lowercase().as_str() { "jpg"|"jpeg" => ImageFormat::Jpeg, "bmp" => ImageFormat::Bmp, "gif" => ImageFormat::Gif, _ => ImageFormat::Png };
                match img.save_with_format(&path, fmt) { Ok(_) => { vm.echo(&format!("Image saved to {}", path)); Some(Ok(Value::None)) } Err(e) => Some(Err(e.to_string())) }
            } else { Some(Err("Image not found".to_string())) } } else { None }
    } else if code.starts_with("~img_width(") && code.ends_with(')') {
        let name = code[11..code.len()-1].trim();
        if let Some(&(w,_,_)) = vm.images.get(name) { vm.echo(w); Some(Ok(Value::Number(w as f64))) } else { Some(Err("Image not found".to_string())) }
    } else if code.starts_with("~img_height(") && code.ends_with(')') {
        let name = code[12..code.len()-1].trim();
        if let Some(&(_,h,_)) = vm.images.get(name) { vm.echo(h); Some(Ok(Value::Number(h as f64))) } else { Some(Err("Image not found".to_string())) }
    } else if code.starts_with("~img_resize(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 3 {
            let name = p[0].trim().to_string(); let nw = vm.eval_expr(&p[1]).ok()? as u32; let nh = vm.eval_expr(&p[2]).ok()? as u32;
            if let Some(&(w,h,ref pixels)) = vm.images.get(&name) {
                let img = image::imageops::resize(&to_img(w,h,pixels), nw, nh, image::imageops::FilterType::Lanczos3);
                let (nw2,nh2,pixels2) = from_img(&img); vm.images.insert(name, (nw2,nh2,pixels2)); Some(Ok(Value::None))
            } else { Some(Err("Image not found".to_string())) } } else { None }
    } else if code.starts_with("~img_crop(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 5 {
            let name = p[0].trim().to_string(); let x = vm.eval_expr(&p[1]).ok()? as u32; let y = vm.eval_expr(&p[2]).ok()? as u32; let w = vm.eval_expr(&p[3]).ok()? as u32; let h = vm.eval_expr(&p[4]).ok()? as u32;
            if let Some(&(ow,oh,ref pixels)) = vm.images.get(&name) {
                let img = to_img(ow,oh,pixels).view(x,y,w-x,h-y).to_image();
                let (nw2,nh2,pixels2) = from_img(&img); vm.images.insert(name, (nw2,nh2,pixels2)); Some(Ok(Value::None))
            } else { Some(Err("Image not found".to_string())) } } else { None }
    } else if code.starts_with("~img_rotate(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[12..code.len()-1], ','); if p.len() >= 2 {
            let name = p[0].trim().to_string(); let deg = vm.eval_expr(&p[1]).ok()? as i32;
            if let Some(&(w,h,ref pixels)) = vm.images.get(&name) {
                let img = to_img(w,h,pixels);
                let rotated = match deg.rem_euclid(360) { 90 => image::imageops::rotate90(&img), 180 => image::imageops::rotate180(&img), 270 => image::imageops::rotate270(&img), _ => img };
                let (nw2,nh2,pixels2) = from_img(&rotated); vm.images.insert(name, (nw2,nh2,pixels2)); Some(Ok(Value::None))
            } else { Some(Err("Image not found".to_string())) } } else { None }
    } else if code.starts_with("~img_flip_h(") && code.ends_with(')') {
        let name = code[12..code.len()-1].trim().to_string();
        if let Some(&(w,h,ref pixels)) = vm.images.get(&name) { let img = image::imageops::flip_horizontal(&to_img(w,h,pixels)); let (nw,nh,pixels2) = from_img(&img); vm.images.insert(name, (nw,nh,pixels2)); Some(Ok(Value::None)) } else { Some(Err("Image not found".to_string())) }
    } else if code.starts_with("~img_flip_v(") && code.ends_with(')') {
        let name = code[12..code.len()-1].trim().to_string();
        if let Some(&(w,h,ref pixels)) = vm.images.get(&name) { let img = image::imageops::flip_vertical(&to_img(w,h,pixels)); let (nw,nh,pixels2) = from_img(&img); vm.images.insert(name, (nw,nh,pixels2)); Some(Ok(Value::None)) } else { Some(Err("Image not found".to_string())) }
    } else if code.starts_with("~img_pixel(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ','); if p.len() >= 3 {
            let name = p[0].trim(); let x = vm.eval_expr(&p[1]).ok()? as usize; let y = vm.eval_expr(&p[2]).ok()? as usize;
            if let Some(&(w,h,ref pixels)) = vm.images.get(name) { if y < h && x < w { let (r,g,b) = pixels[y*w+x]; let s = format!("{},{},{}",r,g,b); vm.echo(&s); Some(Ok(Value::Text(s))) } else { Some(Err("Out of bounds".to_string())) } } else { Some(Err("Image not found".to_string())) }
        } else { None }
    } else if code.starts_with("~img_setpixel(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[14..code.len()-1], ','); if p.len() >= 5 {
            let name = p[0].trim().to_string(); let x = vm.eval_expr(&p[1]).ok()? as usize; let y = vm.eval_expr(&p[2]).ok()? as usize; let r = vm.eval_expr(&p[3]).ok()? as u8; let g = vm.eval_expr(&p[4]).ok()? as u8;
            let b = if p.len() >= 6 { vm.eval_expr(&p[5]).ok()? as u8 } else { 0 };
            if let Some(&mut (w,h,ref mut pixels)) = vm.images.get_mut(&name) { if y < h && x < w { pixels[y*w+x] = (r,g,b); Some(Ok(Value::None)) } else { Some(Err("Out of bounds".to_string())) } } else { Some(Err("Image not found".to_string())) }
        } else { None }
    } else if code.starts_with("~img_fill(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ','); if p.len() >= 4 {
            let name = p[0].trim().to_string(); let r = vm.eval_expr(&p[1]).ok()? as u8; let g = vm.eval_expr(&p[2]).ok()? as u8; let b = vm.eval_expr(&p[3]).ok()? as u8;
            if let Some(&mut (w,h,ref mut pixels)) = vm.images.get_mut(&name) { for y in 0..h { for x in 0..w { pixels[y*w+x] = (r,g,b); } } Some(Ok(Value::None)) } else { Some(Err("Image not found".to_string())) }
        } else { None }
    } else if code.starts_with("~img_grayscale(") && code.ends_with(')') {
        let name = code[15..code.len()-1].trim().to_string();
        if let Some(&(w,h,ref pixels)) = vm.images.clone().get(&name) {
            let mut new_pixels = pixels.clone();
            for y in 0..h { for x in 0..w { let (r,g,b) = new_pixels[y*w+x]; let gray = (r as u32 + g as u32 + b as u32) / 3; new_pixels[y*w+x] = (gray as u8, gray as u8, gray as u8); } }
            vm.images.insert(name, (w,h,new_pixels)); Some(Ok(Value::None))
        } else { Some(Err("Image not found".to_string())) }
    } else if code.starts_with("~img_invert(") && code.ends_with(')') {
        let name = code[12..code.len()-1].trim().to_string();
        if let Some(&(w,h,ref pixels)) = vm.images.clone().get(&name) {
            let mut new_pixels = pixels.clone();
            for y in 0..h { for x in 0..w { let (r,g,b) = new_pixels[y*w+x]; new_pixels[y*w+x] = (255-r,255-g,255-b); } }
            vm.images.insert(name, (w,h,new_pixels)); Some(Ok(Value::None))
        } else { Some(Err("Image not found".to_string())) }
    } else if code.starts_with("~img_draw_line(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[15..code.len()-1], ','); if p.len() >= 7 {
            let name = p[0].trim().to_string(); let (x1,y1,x2,y2) = (vm.eval_expr(&p[1]).ok()? as i32, vm.eval_expr(&p[2]).ok()? as i32, vm.eval_expr(&p[3]).ok()? as i32, vm.eval_expr(&p[4]).ok()? as i32);
            let (r,g,b) = (vm.eval_expr(&p[5]).ok()? as u8, vm.eval_expr(&p[6]).ok()? as u8, if p.len() >= 8 { vm.eval_expr(&p[7]).ok()? as u8 } else { 0 });
            if let Some(&mut (w,h,ref mut pixels)) = vm.images.get_mut(&name) {
                let dx = (x2-x1).abs(); let dy = (y2-y1).abs(); let sx = if x1<x2 {1} else {-1}; let sy = if y1<y2 {1} else {-1};
                let (mut cx, mut cy, mut err) = (x1, y1, dx-dy);
                loop { if cx >= 0 && cx < w as i32 && cy >= 0 && cy < h as i32 { pixels[cy as usize * w + cx as usize] = (r,g,b); } if cx == x2 && cy == y2 { break; } let e2 = 2*err; if e2 > -dy { err -= dy; cx += sx; } if e2 < dx { err += dx; cy += sy; } }
                Some(Ok(Value::None)) } else { Some(Err("Image not found".to_string())) }
        } else { None }
    } else {
        None
    }
}
