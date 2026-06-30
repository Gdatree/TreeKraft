╔═══════════════════════════════════════════════════════════════╗
║                    TREEKRAFT 7.0.0.0                          ║
║          Console language with built-in graphics              ║
╚═══════════════════════════════════════════════════════════════╝

INSTALLATION
═══════════════════════════════════════════════════════════════
  Run TreeKraft_Setup_v7.0.0.exe and follow the instructions.
  Or build from source: cargo build --release

RUN
═══════════════════════════════════════════════════════════════
  treekraft              → interactive REPL mode
  treekraft file.tk      → run script
  double-click on .tk    → auto-run

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ALL COMMANDS

┌─────────────────────────────────────────────────────────────┐
│ 1. TEXT OUTPUT                                              │
├─────────────────────────────────────────────────────────────┤
│  tx"text"              → short form                         │
│  text("text")          → full form                          │
│  ~red(text)            → red text                           │
│  ~green(text)          → green text                         │
│  ~blue(text)           → blue text                          │
│  ~yellow(text)         → yellow text                        │
│  ~fg("color")          → set text color                     │
│  ~bg("color")          → set background color               │
│  ~cur(x, y)            → move cursor to position            │
│  ~cm(text)             → animated letter-by-letter output   │
│  ~al(text)             → alert box, waits for Enter         │
│  tx.color("text")      → colored output without ~           │
│  ~printf("text %d", x) → formatted output                   │
│  ~pad(x, w, c)         → align text                         │
│  Interpolation: tx"Hello {name}!"                           │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 2. VARIABLES AND MATH                                       │
├─────────────────────────────────────────────────────────────┤
│  name = value          → assignment                         │
│  + - * / ^ %           → arithmetic                         │
│  ~inc x   ~dec x       → x += 1 / x -= 1                    │
│  ~abs(x)  ~sqrt(x)     → absolute value, square root        │
│  ~sin(x)  ~cos(x)      → trigonometry (radians)             │
│  ~atan2(y,x)           → arc tangent                        │
│  ~floor(x) ~ceil(x)    → round down / up                    │
│  ~round(x)             → round to nearest                   │
│  ~log(x)  ~exp(x)      → natural log / exponential          │
│  ~min(a,b) ~max(a,b)   → minimum / maximum                  │
│  ~clamp(x, lo, hi)     → clamp value to range               │
│  ~pi   ~e              → math constants                     │
│  ~randf()              → random float [0,1)                 │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 3. STRING OPERATIONS                                        │
├─────────────────────────────────────────────────────────────┤
│  ~upper(x)  ~lower(x)  → change case                        │
│  ~trim(x)   ~rev(x)    → trim spaces / reverse              │
│  ~rep(x, n)            → repeat N times                     │
│  ~len(x)               → length                             │
│  ~split(x, d)          → split by delimiter                 │
│  ~concat(a, b)         → concatenate strings                │
│  ~replace(x, old, new) → replace substring                  │
│  ~sub(x, a, b)         → substring                          │
│  ~find(x, s)           → find position of substring         │
│  ~startswith(x,s)      → check prefix                       │
│  ~endswith(x,s)        → check suffix                       │
│  ~rnds(n)  ~passwd(n)  → random string / password           │
│  ~isspace(x)           → check if whitespace                │
│  ~swapcase(x)          → invert case                        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 4. INPUT AND RANDOM                                         │
├─────────────────────────────────────────────────────────────┤
│  name = ~in("prompt")   → read string from user             │
│  ~rnd(min, max)         → random integer                    │
│  ~key()                 → read single key without Enter     │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 5. CONDITIONS AND LOOPS                                     │
├─────────────────────────────────────────────────────────────┤
│  if x > 5 then command else command                         │
│  if x > 5 then                                              │
│      commands                                               │
│  else                                                       │
│      commands                                               │
│  end                                                        │
│  while i < 10 do command                                    │
│  while i < 10 do                                            │
│      commands                                               │
│  end                                                        │
│  loop                                                       │
│      commands                                               │
│      break / continue                                       │
│  end                                                        │
│  for item in list do command                                │
│  for i from 0 to 9 do command                               │
│  ~case x                                                    │
│      "val" -> command                                       │
│      _     -> default                                       │
│  end                                                        │
│  ~try commands ~catch commands end                          │
│  ~return                                                    │
│  ~goto label                                                │
│  label:                                                     │
│  Operators: == != > < >= <= and or not                      │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 6. LISTS AND DICTIONARIES                                    │
├─────────────────────────────────────────────────────────────┤
│  LISTS:                                                     │
│  a = ~list(1, 2, 3)      → create                           │
│  ~add(a, val)            → append                           │
│  ~get(a, i)              → get by index (0-based)           │
│  ~set(a, i, val)         → set by index                     │
│  ~remove(a, i)           → remove by index                  │
│  ~sort(a)   ~rev(a)      → sort / reverse                   │
│  ~len(a)                 → length                           │
│  DICTIONARIES:                                              │
│  d = ~dict()              → create empty                    │
│  ~dset(d, "key", val)     → set key-value                   │
│  ~dget(d, "key")          → get value by key                │
│  ~dhas(d, "key")          → check existence                 │
│  ~dkeys(d)   ~dvals(d)    → get keys / values               │
│  ~ddel(d, "key")          → delete key                      │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 7. FUNCTIONS                                                │
├─────────────────────────────────────────────────────────────┤
│  ~def name, body                            → no parameters │
│  ~def sum(a, b), a + b                      → with params   │
│  ~call name                                 → call          │
│  ~call sum(2, 3)                            → call with args│
│  ~return                                    → return value  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 8. FILES AND SYSTEM                                         │
├─────────────────────────────────────────────────────────────┤
│  ~read("f.txt")          → read file                        │
│  ~write("f.txt","text")  → write file                       │
│  ~append("f.txt","text") → append to file                   │
│  ~readlines("f.txt")     → read into list of lines          │
│  ~exs("f.txt")           → check if exists                  │
│  ~fsize("f.txt")         → get file size                    │
│  ~dir(".")               → list directory                   │
│  ~mkdir("d") ~rmdir("d") → create / remove directory        │
│  ~delfile("f")           → delete file                      │
│  ~cp("a","b") ~mv("a","b") → copy / move file               │
│  ~save("s.dtam")         → save state                       │
│  ~load("s.dtam")         → load state                       │
│  ~dcfg set/get/del/keys/tovar → .dtam config                │
│  ~env("VAR")             → environment variable             │
│  ~run("cmd")             → system command                   │
│  ~exec("cmd")            → run and capture output           │
│  ~sysinfo                → OS and architecture info         │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 9. TIME AND TIMERS                                          │
├─────────────────────────────────────────────────────────────┤
│  ~now                    → Unix timestamp                   │
│  ~date                   → date-time string                 │
│  ~year ~month ~day       → current date parts               │
│  ~hour ~minute ~second   → current time parts               │
│  ~sleepms(ms)            → sleep milliseconds               │
│  ~timer start/stop       → measure time                     │
│  ~timer "name" start     → named timer start                │
│  ~timer "name" stop      → named timer stop                 │
│  ~mem                    → memory usage                     │
│  ~memclear               → garbage collection               │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 10. CONTROL AND DEBUG                                       │
├─────────────────────────────────────────────────────────────┤
│  ~cls                    → clear screen                     │
│  w~sec   wait(sec)       → pause                            │
│  ~wait5   ~waitk         → pause / wait for key             │
│  ~beep                   → sound signal                     │
│  ~type(x)  ~exists(x)    → type / existence check           │
│  ~vars                   → show all variables (table)       │
│  ~trace on/off           → command tracing                  │
│  ~history                → command history                  │
│  ~autosave on/off        → auto-save to file                │
│  ~reload                 → reload script                    │
│  ~args                   → command-line arguments           │
│  ~const name value       → create constant                  │
│  ~del name               → delete variable                  │
│  ~local name value       → create local variable            │
│  help                    → show help                        │
│  help topic              → help on specific topic           │
│  help en                 → English help                     │
│  license                 → show license                     │
│  exit()                  → exit                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 11. ANIMATION AND SOUND                                     │
├─────────────────────────────────────────────────────────────┤
│  ~melody(notes)          → play melody                      │
│  ~melody C D E F G A B   → notes (C4-D4-E4)                 │
│  ~beep                   → short beep                       │
│  ~cm(text)               → animated text                    │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ 12. MAPS AND OBJECTS                                        │
├─────────────────────────────────────────────────────────────┤
│  ~map(name, w, h)        → create map                       │
│  ~map(name)              → show map                         │
│  ~obj(name, x, y, char)  → create object                    │
│  ~objmove(name, dx, dy)  → move object                      │
│  ~objdel(name)           → delete object                    │
│  ~spawn(name, x, y, char)→ create and show object           │
│  ~showmap                → show map with objects            │
│  ~loadmap("file.map")    → load map from file               │
└─────────────────────────────────────────────────────────────┘

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ASCII DISPLAY (LD)

┌─────────────────────────────────────────────────────────────┐
│  ~ldsc(w, h, border, fill)  → create screen                 │
│  ~ld(x, y, char)            → put char at cell              │
│  ~ldbox(x, y, w, h, char)   → draw rectangle                │
│  ~ldm(x1, y1, x2, y2, char) → draw line (Bresenham)         │
│  ~ldtext(x, y, text)        → put text on screen            │
│  ~ldfill(char)              → fill screen                   │
│  ~ldc(x, y)                 → clear cell                    │
│  ~ldg(x, y)                 → get char from cell            │
│  ~ldshow()   ~ldclear()     → show / clear screen           │
│                                                             │
│  Example:                                                   │
│    ~ldsc(20, 8, "#", ".")                                   │
│    ~ldbox(0, 0, 19, 7, "#")                                 │
│    ~ldtext(6, 3, "TreeKraft")                               │
│    ~ldshow()                                                │
└─────────────────────────────────────────────────────────────┘

GRAPHICS MODULE KGame

┌─────────────────────────────────────────────────────────────┐
│  kg.init()              → enter graphics mode               │
│  kg.rect(x,y,w,h,color) → rectangle (ANSI background)       │
│  kg.circ(x,y,r,color)   → circle                            │
│  kg.text(x,y,"text",col,fg) → text with colors              │
│  kg.pixel(x,y,char,fg,bg) → character with colors           │
│  kg.clr(color)          → fill background                   │
│  kg.cur(x,y)            → move cursor                       │
│  kg.update()            → render frame                      │
│  kg.quit()              → exit graphics mode                │
│                                                             │
│  Colors: red green blue yellow white cyan magenta black     │
└─────────────────────────────────────────────────────────────┘

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

CREATING SCRIPTS
═══════════════════════════════════════════════════════════════
  1. Create a .tk file in any text editor
  2. Write TreeKraft code
  3. Run: treekraft filename.tk

EXAMPLE SCRIPTS
═══════════════════════════════════════════════════════════════

Hello world (hello.tk):

    ~green(=== WELCOME ===)
    name = ~in("What is your name? ")
    tx("Hello, {name}!")
    ~wait2
    ~red(Goodbye!)

Number guessing game:

    ~green(Guess a number from 1 to 10)
    secret = ~rnd(1, 10)
    while 1 == 1 do
        guess = ~in("Your answer: ")
        if guess == secret then
            ~green(You win!)
            break
        else
            if guess < secret then
                ~yellow(Higher)
            else
                ~yellow(Lower)
            end
        end
    end

ASCII game example:

    ~ldsc(40, 20, "#", ".")
    x = 5
    y = 5
    while 1 == 1 do
        key = ~key()
        if key == "w" then y = y - 1 end
        if key == "s" then y = y + 1 end
        if key == "a" then x = x - 1 end
        if key == "d" then x = x + 1 end
        ~ldc(x, y)
        ~ld(x, y, "@")
        ~ldshow()
    end

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

COMMON ERRORS

  "Unknown command"
    → Check command spelling
  "Not found"
    → Variable not declared. Use ~exists() to check
  "Not a number"
    → Math operations require numbers. Check with ~type()
  "Index out of range"
    → List index is out of bounds
  "File not found"
    → File does not exist. Check the path

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

SUPPORT

  Website: https://treekraft.tilda.ws/
  Email: treekraft@proton.me
  GitHub: https://github.com/Gdatree/TreeKraft

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

(c) 2026 TreeKraft Team. License: MIT / Apache 2.0