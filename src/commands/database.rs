use crate::main::vm::{VM, Value};
use std::cell::RefCell;

thread_local! {
    static DB: RefCell<Option<rusqlite::Connection>> = const { RefCell::new(None) };
    static DB_PATH: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn with_db<F, R>(f: F) -> Result<R, String> where F: FnOnce(&rusqlite::Connection) -> Result<R, String> {
    DB.with(|db| {
        let db = db.borrow();
        match &*db {
            Some(conn) => f(conn),
            None => Err("No database open. Use ~db_open(path)".to_string()),
        }
    })
}

fn valid_ident(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn escape_sql(s: &str) -> String {
    s.replace('\'', "''")
}

fn with_db_mut<F, R>(f: F) -> Result<R, String> where F: FnOnce(&mut rusqlite::Connection) -> Result<R, String> {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        match &mut *db {
            Some(conn) => f(conn),
            None => Err("No database open. Use ~db_open(path)".to_string()),
        }
    })
}

// SELECT отдаем как пару заголовки плюс строки где все значения уже приведены к тексту чтобы скрипт всегда получал одинаковую структуру
fn run_query(conn: &rusqlite::Connection, sql: &str) -> Result<Value, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count).map(|i| stmt.column_name(i).unwrap_or("?").to_string()).collect();
    let rows = stmt.query_map([], |row| {
        let mut values = Vec::new();
        for i in 0..col_count {
            let val: rusqlite::Result<String> = row.get::<_, String>(i)
                .or_else(|_| row.get::<_, i64>(i).map(|n| n.to_string()))
                .or_else(|_| row.get::<_, f64>(i).map(|n| Value::fmt_num(n)))
                .or_else(|_| row.get::<_, Option<String>>(i).map(|s| s.unwrap_or_default()));
            values.push(val.unwrap_or_default());
        }
        Ok(Value::List(values.into_iter().map(Value::Text).collect()))
    }).map_err(|e| e.to_string())?;
    let result: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
    let result = Value::List(vec![Value::List(col_names.into_iter().map(Value::Text).collect()), Value::List(result)]);
    Ok(result)
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~db_open(") && code.ends_with(')') {
        let path = vm.eval_value(&code[9..code.len()-1]).ok()?.as_text()?;
        match rusqlite::Connection::open(&path) {
            Ok(conn) => {
                DB.with(|db| *db.borrow_mut() = Some(conn));
                DB_PATH.with(|p| *p.borrow_mut() = Some(path.clone()));
                println!("Database opened: {}", path);
                Some(Ok(Value::None))
            }
            Err(e) => Some(Err(e.to_string())),
        }
    } else if code.starts_with("~db_close") && (code.len() == 9 || (code.starts_with("~db_close(") && code.ends_with(')'))) {
        DB.with(|db| *db.borrow_mut() = None);
        DB_PATH.with(|p| *p.borrow_mut() = None);
        println!("Database closed");
        Some(Ok(Value::None))
    } else if code.starts_with("~db_exec(") && code.ends_with(')') {
        let sql = vm.eval_value(&code[9..code.len()-1]).ok()?.as_text()?;
        match with_db_mut(|conn| { conn.execute_batch(&sql).map_err(|e| e.to_string()) }) {
            Ok(_) => { println!("OK"); Some(Ok(Value::None)) }
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~db_query(") && code.ends_with(')') {
        let sql = vm.eval_value(&code[10..code.len()-1]).ok()?.as_text()?;
        match with_db(|conn| run_query(conn, &sql)) { Ok(r) => Some(Ok(r)), Err(e) => Some(Err(e)) }
    } else if code.starts_with("~db_query_row(") && code.ends_with(')') {
        let sql = vm.eval_value(&code[14..code.len()-1]).ok()?.as_text()?;
        match with_db(|conn| {
            let all = run_query(conn, &sql)?;
            if let Value::List(items) = all {
                let headers = items.first().cloned().unwrap_or(Value::List(vec![]));
                let rows = items.get(1).cloned().unwrap_or(Value::List(vec![]));
                if let Value::List(rows_list) = rows {
                    if let Some(first) = rows_list.first() {
                        vm.echo(format!("{}", first.display()));
                        return Ok(first.clone());
                    }
                }
                vm.echo(format!("{}", headers.display()));
                return Ok(headers);
            }
            Err("Bad query".to_string())
        }) { Ok(r) => Some(Ok(r)), Err(e) => Some(Err(e)) }
    } else if code.starts_with("~db_query_value(") && code.ends_with(')') {
        let sql = vm.eval_value(&code[16..code.len()-1]).ok()?.as_text()?;
        match with_db(|conn| {
            let all = run_query(conn, &sql)?;
            if let Value::List(items) = all {
                if let Value::List(rows) = items.get(1).cloned().unwrap_or(Value::List(vec![])) {
                    if let Some(Value::List(first_row)) = rows.first() {
                        if let Some(v) = first_row.first() {
                            vm.echo(format!("{}", v.display()));
                            return Ok(v.clone());
                        }
                    }
                }
            }
            Ok(Value::None)
        }) { Ok(r) => Some(Ok(r)), Err(e) => Some(Err(e)) }
    } else if code.starts_with("~db_execute(") && code.ends_with(')') {
        // Значения подставляем параметрами а не склейкой строк чтобы кавычки не ломали запрос и не давали инъекцию
        let p = VM::split_top_level(&code[12..code.len()-1], ',');
        if p.len() >= 1 {
            let sql = vm.eval_value(&p[0]).ok()?.as_text()?;
            let mut args: Vec<rusqlite::types::Value> = Vec::new();
            for a in &p[1..] {
                let v = vm.eval_value(a).unwrap_or(Value::None);
                if let Some(n) = v.as_number() {
                    if n == n.trunc() && n.abs() < 9_000_000_000_000_000_000.0 { args.push(rusqlite::types::Value::Integer(n as i64)); }
                    else { args.push(rusqlite::types::Value::Real(n)); }
                } else { args.push(rusqlite::types::Value::Text(v.display())); }
            }
            match with_db_mut(|conn| {
                let rows = conn.execute(&sql, rusqlite::params_from_iter(args.iter())).map_err(|e| e.to_string())?;
                vm.echo(format!("{} rows affected", rows));
                Ok(Value::Number(rows as f64))
            }) { Ok(r) => Some(Ok(r)), Err(e) => Some(Err(e)) }
        } else { None }
    } else if code.starts_with("~db_insert(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 2 {
            let table = vm.eval_value(&p[0]).ok()?.as_text()?;
            let vals = vm.eval_value(&p[1]).ok()?;
            if !valid_ident(&table) {
                return Some(Err(format!("db_insert: invalid table name '{}'", table)));
            }
            let values = match &vals {
                Value::List(l) => l.iter().filter_map(|v| v.as_text()).map(|s| escape_sql(&s)).collect::<Vec<_>>().join("','"),
                _ => escape_sql(&vals.as_text().unwrap_or_default()),
            };
            let sql = format!("INSERT INTO {} VALUES('{}')", table, values);
            match with_db_mut(|conn| conn.execute_batch(&sql).map_err(|e| e.to_string())) {
                Ok(_) => { println!("OK"); Some(Ok(Value::None)) }
                Err(e) => Some(Err(e)),
            }
        } else { None }
    } else if code.starts_with("~db_create(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[11..code.len()-1], ',');
        if p.len() >= 2 {
            let table = vm.eval_value(&p[0]).ok()?.as_text()?;
            let cols = vm.eval_value(&p[1]).ok()?.as_text()?;
            if !valid_ident(&table) {
                return Some(Err(format!("db_create: invalid table name '{}'", table)));
            }
            if cols.contains(';') {
                return Some(Err("db_create: column definition must not contain ';'".to_string()));
            }
            let sql = format!("CREATE TABLE IF NOT EXISTS {} ({})", table, cols);
            match with_db_mut(|conn| conn.execute_batch(&sql).map_err(|e| e.to_string())) {
                Ok(_) => { println!("Table '{}' created", table); Some(Ok(Value::None)) }
                Err(e) => Some(Err(e)),
            }
        } else { None }
    } else if code.starts_with("~db_tables") && (code.len() == 10 || (code.starts_with("~db_tables(") && code.ends_with(')'))) {
        match with_db(|conn| {
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").map_err(|e| e.to_string())?;
            let names: Vec<Value> = stmt.query_map([], |row| row.get::<_, String>(0)).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).map(Value::Text).collect();
            let r = Value::List(names);
            vm.echo(r.display());
            Ok(r)
        }) { Ok(r) => Some(Ok(r)), Err(e) => Some(Err(e)) }
    } else if code.starts_with("~db_columns(") && code.ends_with(')') {
        let table = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?;
        if !valid_ident(&table) {
            return Some(Err(format!("db_columns: invalid table name '{}'", table)));
        }
        match with_db(|conn| {
            let mut stmt = conn.prepare(&format!("PRAGMA table_info(\"{}\")", table.replace('"', "\"\""))).map_err(|e| e.to_string())?;
            let names: Vec<Value> = stmt.query_map([], |row| row.get::<_, String>(1)).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).map(Value::Text).collect();
            let r = Value::List(names);
            vm.echo(r.display());
            Ok(r)
        }) { Ok(r) => Some(Ok(r)), Err(e) => Some(Err(e)) }
    } else if code.starts_with("~db_transaction") || code == "~db_begin" {
        match with_db_mut(|conn| conn.execute_batch("BEGIN TRANSACTION").map_err(|e| e.to_string())) {
            Ok(_) => { println!("Transaction started"); Some(Ok(Value::None)) }
            Err(e) => Some(Err(e)),
        }
    } else if code == "~db_commit" || code.starts_with("~db_commit(") {
        match with_db_mut(|conn| conn.execute_batch("COMMIT").map_err(|e| e.to_string())) {
            Ok(_) => { println!("Transaction committed"); Some(Ok(Value::None)) }
            Err(e) => Some(Err(e)),
        }
    } else if code == "~db_rollback" || code.starts_with("~db_rollback(") {
        match with_db_mut(|conn| conn.execute_batch("ROLLBACK").map_err(|e| e.to_string())) {
            Ok(_) => { println!("Transaction rolled back"); Some(Ok(Value::None)) }
            Err(e) => Some(Err(e)),
        }
    } else if code == "~db_last_insert_id" || code.starts_with("~db_last_insert_id(") {
        match with_db(|conn| {
            let id: i64 = conn.last_insert_rowid();
            vm.echo(id);
            Ok(Value::Number(id as f64))
        }) { Ok(r) => Some(Ok(r)), Err(e) => Some(Err(e)) }
    } else if code == "~db_changes" || code.starts_with("~db_changes(") {
        match with_db(|conn| {
            let n: u64 = conn.changes();
            vm.echo(n);
            Ok(Value::Number(n as f64))
        }) { Ok(r) => Some(Ok(r)), Err(e) => Some(Err(e)) }
    } else if code.starts_with("~db_backup(") && code.ends_with(')') {
        let path = vm.eval_value(&code[11..code.len()-1]).ok()?.as_text()?;
        let quoted = format!("'{}'", path.replace('\'', "''"));
        match with_db_mut(|conn| conn.execute_batch(&format!("VACUUM INTO {}", quoted)).map_err(|e| e.to_string())) {
            Ok(_) => { println!("Backup saved: {}", path); Some(Ok(Value::Bool(true))) }
            Err(e) => Some(Err(e)),
        }
    } else if code.starts_with("~db_restore(") && code.ends_with(')') {
        let path = vm.eval_value(&code[12..code.len()-1]).ok()?.as_text()?;
        // Перед копией бэкапа базу закрываем и потом открываем заново чтобы SQLite не продолжал держать старые данные в памяти
        match with_db(|_| Ok(())) {
            Ok(_) => {
                let current = DB_PATH.with(|p| p.borrow().clone());
                match (current, std::fs::read(&path)) {
                    (Some(cur), Ok(data)) => {
                        DB.with(|db| *db.borrow_mut() = None);
                        if std::fs::write(&cur, &data).is_err() {
                            return Some(Err("Failed to restore file".to_string()));
                        }
                        match rusqlite::Connection::open(&cur) {
                            Ok(conn) => { DB.with(|db| *db.borrow_mut() = Some(conn)); println!("Database restored from {}", path); Some(Ok(Value::Bool(true))) }
                            Err(e) => Some(Err(e.to_string())),
                        }
                    }
                    _ => Some(Err("No open database or backup not found".to_string())),
                }
            }
            Err(_) => Some(Err("No database open".to_string())),
        }
    } else {
        None
    }
}
