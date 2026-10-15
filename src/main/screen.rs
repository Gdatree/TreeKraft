use std::fs;
use std::io::Write;

pub struct Screen {
    pub width:  usize,
    pub height: usize,
    pub buffer: Vec<Vec<char>>,
    pub fg:     Vec<Vec<Option<(u8,u8,u8)>>>,
    pub bg:     Vec<Vec<Option<(u8,u8,u8)>>>,
    pub border: char,
    pub empty:  char,
}

impl Screen {
    pub fn new(width: usize, height: usize, border: char, empty: char) -> Self {
        let buf = vec![vec![empty; width]; height];
        let fg  = vec![vec![None; width]; height];
        let bg  = vec![vec![None; width]; height];
        let mut s = Self { width, height, buffer: buf, fg, bg, border, empty };
        if border != '\0' && width >= 2 && height >= 2 {
            for x in 0..width  {
                s.buffer[0][x]          = border;
                s.buffer[height-1][x]   = border;
            }
            for y in 0..height {
                s.buffer[y][0]          = border;
                s.buffer[y][width-1]    = border;
            }
        }
        s
    }

    // БАЗОВЫЕ

    pub fn set(&mut self, x: usize, y: usize, ch: char) {
        if x < self.width && y < self.height {
            self.buffer[y][x] = ch;
        }
    }

    pub fn get(&self, x: usize, y: usize) -> char {
        if x < self.width && y < self.height { self.buffer[y][x] } else { ' ' }
    }

    pub fn set_color(&mut self, x: usize, y: usize,
        fg: Option<(u8,u8,u8)>, bg: Option<(u8,u8,u8)>)
    {
        if x < self.width && y < self.height {
            self.fg[y][x] = fg;
            self.bg[y][x] = bg;
        }
    }

    pub fn clear(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let is_border = self.border != '\0'
                    && self.width >= 2 && self.height >= 2
                    && (x == 0 || x == self.width-1 || y == 0 || y == self.height-1);
                self.buffer[y][x] = if is_border { self.border } else { self.empty };
                self.fg[y][x] = None;
                self.bg[y][x] = None;
            }
        }
    }

    pub fn fill(&mut self, ch: char) {
        for row in &mut self.buffer {
            for c in row.iter_mut() { *c = ch; }
        }
    }

    // ФИГУРЫ

    pub fn draw_box(&mut self, x: usize, y: usize, w: usize, h: usize, ch: char) {
        if w < 2 || h < 2 { return; }
        for dx in 0..w {
            self.set(x+dx, y,     ch);
            self.set(x+dx, y+h-1, ch);
        }
        for dy in 0..h {
            self.set(x,     y+dy, ch);
            self.set(x+w-1, y+dy, ch);
        }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, ch: char) {
        for dy in 0..h {
            for dx in 0..w {
                self.set(x+dx, y+dy, ch);
            }
        }
    }

    pub fn draw_line(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, ch: char) {
        let dx = (x2 as i64 - x1 as i64).abs();
        let dy = (y2 as i64 - y1 as i64).abs();
        let sx: i64 = if x1 < x2 { 1 } else { -1 };
        let sy: i64 = if y1 < y2 { 1 } else { -1 };
        let (mut cx, mut cy, mut err) = (x1 as i64, y1 as i64, dx - dy);
        loop {
            if cx >= 0 && cy >= 0 { self.set(cx as usize, cy as usize, ch); }
            if cx == x2 as i64 && cy == y2 as i64 { break; }
            let e2 = 2 * err;
            if e2 > -dy { err -= dy; cx += sx; }
            if e2 <  dx { err += dx; cy += sy; }
        }
    }

    pub fn draw_circle(&mut self, ocx: usize, ocy: usize, r: usize, ch: char) {
        if r == 0 { self.set(ocx, ocy, ch); return; }
        let (cx, cy, r2) = (ocx as i64, ocy as i64, r as i64);
        let (mut x, mut y, mut d) = (0i64, r2, 3 - 2*r2);
        while y >= x {
            for &(px, py) in &[
                (cx+x, cy+y), (cx-x, cy+y), (cx+x, cy-y), (cx-x, cy-y),
                (cx+y, cy+x), (cx-y, cy+x), (cx+y, cy-x), (cx-y, cy-x),
            ] {
                if px >= 0 && py >= 0 { self.set(px as usize, py as usize, ch); }
            }
            if d < 0 { d += 4*x + 6; } else { d += 4*(x-y) + 10; y -= 1; }
            x += 1;
        }
    }

    pub fn fill_circle(&mut self, ocx: usize, ocy: usize, r: usize, ch: char) {
        let (cx, cy, ir) = (ocx as i64, ocy as i64, r as i64);
        for dy in -ir..=ir {
            for dx in -ir..=ir {
                if dx*dx + dy*dy <= ir*ir {
                    let (px, py) = (cx+dx, cy+dy);
                    if px >= 0 && py >= 0 {
                        self.set(px as usize, py as usize, ch);
                    }
                }
            }
        }
    }

    // ПРОКРУТКА

    pub fn scroll(&mut self, dx: i32, dy: i32) {
        if dy > 0 {
            let n = dy as usize;
            for y in (n..self.height).rev() {
                for x in 0..self.width { self.buffer[y][x] = self.buffer[y-n][x]; }
            }
            for y in 0..n.min(self.height) {
                for x in 0..self.width { self.buffer[y][x] = self.empty; }
            }
        } else if dy < 0 {
            let n = (-dy) as usize;
            for y in 0..self.height.saturating_sub(n) {
                for x in 0..self.width { self.buffer[y][x] = self.buffer[y+n][x]; }
            }
            for y in self.height.saturating_sub(n)..self.height {
                for x in 0..self.width { self.buffer[y][x] = self.empty; }
            }
        }
        if dx > 0 {
            let n = dx as usize;
            for y in 0..self.height {
                for x in (n..self.width).rev() { self.buffer[y][x] = self.buffer[y][x-n]; }
                for x in 0..n.min(self.width)  { self.buffer[y][x] = self.empty; }
            }
        } else if dx < 0 {
            let n = (-dx) as usize;
            for y in 0..self.height {
                for x in 0..self.width.saturating_sub(n) { self.buffer[y][x] = self.buffer[y][x+n]; }
                for x in self.width.saturating_sub(n)..self.width { self.buffer[y][x] = self.empty; }
            }
        }
    }

    // ТЕКСТ

    pub fn draw_text(&mut self, x: usize, y: usize, text: &str) {
        for (i, ch) in text.chars().enumerate() {
            self.set(x + i, y, ch);
        }
    }

    // ВЫВОД

    pub fn show(&self) {
        for (y, row) in self.buffer.iter().enumerate() {
            let mut line = String::new();
            for (x, &ch) in row.iter().enumerate() {
                let fc = self.fg[y][x];
                let bc = self.bg[y][x];
                if fc.is_some() || bc.is_some() {
                    if let Some((r, g, b)) = fc {
                        line.push_str(&format!("\x1B[38;2;{};{};{}m", r, g, b));
                    }
                    if let Some((r, g, b)) = bc {
                        line.push_str(&format!("\x1B[48;2;{};{};{}m", r, g, b));
                    }
                    line.push(ch);
                    line.push_str("\x1B[0m");
                } else {
                    line.push(ch);
                }
            }
            println!("{}", line);
        }
    }

    // ФАЙЛЫ

    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let content: String = self.buffer
            .iter()
            .map(|row| {
                let s: String = row.iter().collect();
                format!("{}\n", s)
            })
            .collect();
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn load_from_file(&mut self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        for (y, line) in content.lines().enumerate() {
            if y >= self.height { break; }
            for (x, ch) in line.chars().enumerate() {
                if x >= self.width { break; }
                self.buffer[y][x] = ch;
            }
        }
        Ok(())
    }

    pub fn render(&self) {
        for (y, row) in self.buffer.iter().enumerate() {
            print!("\x1B[{};1H", y + 1);
            for (x, &ch) in row.iter().enumerate() {
                print!("{}", ch);
            }
        }
        print!("\x1B[{};1H", self.height + 1);
        let _ = std::io::stdout().flush();
    }
}

// ПУБЛИЧНЫЕ ХЕЛПЕРЫ

pub fn eval_ldsc(args: &[String]) -> Result<Screen, String> {
    if args.len() < 2 {
        return Err("~ldsc(w,h,border?,fill?)".to_string());
    }
    let w = args[0].trim().parse::<usize>()
        .map_err(|_| "width must be a number".to_string())?;
    let h = args[1].trim().parse::<usize>()
        .map_err(|_| "height must be a number".to_string())?;
    let border = if args.len() > 2 {
        args[2].trim().trim_matches('"').trim_matches('\'')
               .chars().next().unwrap_or('#')
    } else { '#' };
    let empty = if args.len() > 3 {
        args[3].trim().trim_matches('"').trim_matches('\'')
               .chars().next().unwrap_or(' ')
    } else { ' ' };
    Ok(Screen::new(w, h, border, empty))
}

pub fn parse_color(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim().trim_matches('"').trim_matches('\'');
    // числовой формат "r, g, b"
    let p: Vec<&str> = s.split(',').collect();
    if p.len() == 3 {
        return Some((
            p[0].trim().parse().ok()?,
            p[1].trim().parse().ok()?,
            p[2].trim().parse().ok()?,
        ));
    }
    // hex формат #rrggbb (только ASCII, иначе байтовые срезы ниже паникуют
    // на границе символа; не-ASCII hex заведомо невалиден)
    if s.starts_with('#') && s.is_ascii() && (s.len() == 7 || s.len() == 4) {
        if s.len() == 7 {
            let r = u8::from_str_radix(&s[1..3], 16).ok()?;
            let g = u8::from_str_radix(&s[3..5], 16).ok()?;
            let b = u8::from_str_radix(&s[5..7], 16).ok()?;
            return Some((r, g, b));
        } else {
            // #rgb > #rrggbb
            let r = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&s[3..4].repeat(2), 16).ok()?;
            return Some((r, g, b));
        }
    }
    // имена цветов
    Some(match s.to_lowercase().as_str() {
        "red"     | "r"   => (255,   0,   0),
        "green"   | "g"   => (  0, 255,   0),
        "blue"    | "b"   => (  0,   0, 255),
        "yellow"  | "y"   => (255, 255,   0),
        "white"   | "w"   => (255, 255, 255),
        "black"   | "k"   => (  0,   0,   0),
        "cyan"    | "c"   => (  0, 255, 255),
        "magenta" | "m"   => (255,   0, 255),
        "orange"          => (255, 165,   0),
        "pink"            => (255, 192, 203),
        "purple"          => (128,   0, 128),
        "gray"   | "grey" => (128, 128, 128),
        "lightgray"       => (211, 211, 211),
        "darkgray"        => ( 64,  64,  64),
        "lime"            => (  0, 255,   0),
        "teal"            => (  0, 128, 128),
        "navy"            => (  0,   0, 128),
        "maroon"          => (128,   0,   0),
        "olive"           => (128, 128,   0),
        "aqua"            => (  0, 255, 255),
        "silver"          => (192, 192, 192),
        "gold"            => (255, 215,   0),
        "indigo"          => ( 75,   0, 130),
        "violet"          => (238, 130, 238),
        "brown"           => (165,  42,  42),
        "coral"           => (255, 127,  80),
        "salmon"          => (250, 128, 114),
        "tan"             => (210, 180, 140),
        "khaki"           => (240, 230, 140),
        "turquoise"       => ( 64, 224, 208),
        _                 => return None,
    })
}
