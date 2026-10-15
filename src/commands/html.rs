use crate::main::vm::{VM, Value};

fn get_text(vm: &mut VM, s: &str) -> Option<String> {
    vm.eval_value(s).ok().and_then(|v| v.as_text())
}

pub fn resolve(vm: &mut VM, code: &str) -> Option<Result<Value, String>> {
    if code.starts_with("~html_tag(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() == 3 {
            let r = format!("<{} {}>{}</{}>", get_text(vm, &p[0])?, get_text(vm, &p[1])?, get_text(vm, &p[2])?, get_text(vm, &p[0])?);
            vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
        } else if p.len() == 2 {
            let r = format!("<{}>{}</{}>", get_text(vm, &p[0])?, get_text(vm, &p[1])?, get_text(vm, &p[0])?);
            vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
        } else { None }
    } else if code.starts_with("~html_div(") && code.ends_with(')') {
        let s = get_text(vm, &code[10..code.len()-1])?;
        vm.echo(format!("<div>{}</div>", s)); Some(Ok(Value::Text(format!("<div>{}</div>", s))))
    } else if code.starts_with("~html_span(") && code.ends_with(')') {
        let s = get_text(vm, &code[11..code.len()-1])?;
        vm.echo(format!("<span>{}</span>", s)); Some(Ok(Value::Text(format!("<span>{}</span>", s))))
    } else if code.starts_with("~html_p(") && code.ends_with(')') {
        let s = get_text(vm, &code[8..code.len()-1])?;
        vm.echo(format!("<p>{}</p>", s)); Some(Ok(Value::Text(format!("<p>{}</p>", s))))
    } else if code.starts_with("~html_a(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[8..code.len()-1], ',');
        if p.len() == 2 {
            let r = format!("<a href=\"{}\">{}</a>", get_text(vm, &p[0])?, get_text(vm, &p[1])?);
            vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
        } else { None }
    } else if code.starts_with("~html_img(") && code.ends_with(')') {
        let p = VM::split_top_level(&code[10..code.len()-1], ',');
        if p.len() >= 2 {
            let alt = if p.len() >= 3 { format!(" alt=\"{}\"", get_text(vm, &p[2])?) } else { String::new() };
            let r = format!("<img src=\"{}\"{}>", get_text(vm, &p[0])?, alt);
            vm.echo(format!("{}", r)); Some(Ok(Value::Text(r)))
        } else { None }
    } else if code.starts_with("~html_table(") && code.ends_with(')') {
        let s = get_text(vm, &code[12..code.len()-1])?;
        vm.echo(format!("<table><tr>{}</tr></table>", s)); Some(Ok(Value::Text(format!("<table><tr>{}</tr></table>", s))))
    } else if code.starts_with("~html_tr(") && code.ends_with(')') {
        let s = get_text(vm, &code[9..code.len()-1])?;
        vm.echo(format!("<tr>{}</tr>", s)); Some(Ok(Value::Text(format!("<tr>{}</tr>", s))))
    } else if code.starts_with("~html_td(") && code.ends_with(')') {
        let s = get_text(vm, &code[9..code.len()-1])?;
        vm.echo(format!("<td>{}</td>", s)); Some(Ok(Value::Text(format!("<td>{}</td>", s))))
    } else if code.starts_with("~html_br(") || code == "~html_br" {
        vm.echo("<br>"); Some(Ok(Value::Text("<br>".to_string())))
    } else if code.starts_with("~html_h1(") && code.ends_with(')') {
        let s = get_text(vm, &code[9..code.len()-1])?;
        vm.echo(format!("<h1>{}</h1>", s)); Some(Ok(Value::Text(format!("<h1>{}</h1>", s))))
    } else if code.starts_with("~html_h2(") && code.ends_with(')') {
        let s = get_text(vm, &code[9..code.len()-1])?;
        vm.echo(format!("<h2>{}</h2>", s)); Some(Ok(Value::Text(format!("<h2>{}</h2>", s))))
    } else if code.starts_with("~html_ul(") && code.ends_with(')') {
        let s = get_text(vm, &code[9..code.len()-1])?;
        vm.echo(format!("<ul>{}</ul>", s)); Some(Ok(Value::Text(format!("<ul>{}</ul>", s))))
    } else if code.starts_with("~html_li(") && code.ends_with(')') {
        let s = get_text(vm, &code[9..code.len()-1])?;
        vm.echo(format!("<li>{}</li>", s)); Some(Ok(Value::Text(format!("<li>{}</li>", s))))
    } else if code.starts_with("~html_h3(") && code.ends_with(')') { let s = get_text(vm, &code[9..code.len()-1])?; vm.echo(format!("<h3>{}</h3>", s)); Some(Ok(Value::Text(format!("<h3>{}</h3>", s)))) }
    else if code.starts_with("~html_h4(") && code.ends_with(')') { let s = get_text(vm, &code[9..code.len()-1])?; vm.echo(format!("<h4>{}</h4>", s)); Some(Ok(Value::Text(format!("<h4>{}</h4>", s)))) }
    else if code.starts_with("~html_h5(") && code.ends_with(')') { let s = get_text(vm, &code[9..code.len()-1])?; vm.echo(format!("<h5>{}</h5>", s)); Some(Ok(Value::Text(format!("<h5>{}</h5>", s)))) }
    else if code.starts_with("~html_h6(") && code.ends_with(')') { let s = get_text(vm, &code[9..code.len()-1])?; vm.echo(format!("<h6>{}</h6>", s)); Some(Ok(Value::Text(format!("<h6>{}</h6>", s)))) }
    else if code.starts_with("~html_code(") && code.ends_with(')') { let s = get_text(vm, &code[11..code.len()-1])?; vm.echo(format!("<code>{}</code>", s)); Some(Ok(Value::Text(format!("<code>{}</code>", s)))) }
    else if code.starts_with("~html_pre(") && code.ends_with(')') { let s = get_text(vm, &code[10..code.len()-1])?; vm.echo(format!("<pre>{}</pre>", s)); Some(Ok(Value::Text(format!("<pre>{}</pre>", s)))) }
    else if code.starts_with("~html_strong(") && code.ends_with(')') { let s = get_text(vm, &code[13..code.len()-1])?; vm.echo(format!("<strong>{}</strong>", s)); Some(Ok(Value::Text(format!("<strong>{}</strong>", s)))) }
    else if code.starts_with("~html_em(") && code.ends_with(')') { let s = get_text(vm, &code[9..code.len()-1])?; vm.echo(format!("<em>{}</em>", s)); Some(Ok(Value::Text(format!("<em>{}</em>", s)))) }
    else if code.starts_with("~html_blockquote(") && code.ends_with(')') { let s = get_text(vm, &code[17..code.len()-1])?; vm.echo(format!("<blockquote>{}</blockquote>", s)); Some(Ok(Value::Text(format!("<blockquote>{}</blockquote>", s)))) }
    else if code.starts_with("~html_hr(") && code.ends_with(')') || code == "~html_hr" { vm.echo("<hr>"); Some(Ok(Value::Text("<hr>".to_string()))) }
    else if code.starts_with("~html_nav(") && code.ends_with(')') { let s = get_text(vm, &code[10..code.len()-1])?; vm.echo(format!("<nav>{}</nav>", s)); Some(Ok(Value::Text(format!("<nav>{}</nav>", s)))) }
    else if code.starts_with("~html_section(") && code.ends_with(')') { let s = get_text(vm, &code[14..code.len()-1])?; vm.echo(format!("<section>{}</section>", s)); Some(Ok(Value::Text(format!("<section>{}</section>", s)))) }
    else if code.starts_with("~html_header(") && code.ends_with(')') { let s = get_text(vm, &code[13..code.len()-1])?; vm.echo(format!("<header>{}</header>", s)); Some(Ok(Value::Text(format!("<header>{}</header>", s)))) }
    else if code.starts_with("~html_footer(") && code.ends_with(')') { let s = get_text(vm, &code[13..code.len()-1])?; vm.echo(format!("<footer>{}</footer>", s)); Some(Ok(Value::Text(format!("<footer>{}</footer>", s)))) }
    else if code.starts_with("~html_main(") && code.ends_with(')') { let s = get_text(vm, &code[11..code.len()-1])?; vm.echo(format!("<main>{}</main>", s)); Some(Ok(Value::Text(format!("<main>{}</main>", s)))) }
    else if code.starts_with("~html_aside(") && code.ends_with(')') { let s = get_text(vm, &code[12..code.len()-1])?; vm.echo(format!("<aside>{}</aside>", s)); Some(Ok(Value::Text(format!("<aside>{}</aside>", s)))) }
    else if code.starts_with("~html_article(") && code.ends_with(')') { let s = get_text(vm, &code[14..code.len()-1])?; vm.echo(format!("<article>{}</article>", s)); Some(Ok(Value::Text(format!("<article>{}</article>", s)))) }
    else if code.starts_with("~html_figure(") && code.ends_with(')') { let s = get_text(vm, &code[13..code.len()-1])?; vm.echo(format!("<figure>{}</figure>", s)); Some(Ok(Value::Text(format!("<figure>{}</figure>", s)))) }
    else if code.starts_with("~html_figcaption(") && code.ends_with(')') { let s = get_text(vm, &code[17..code.len()-1])?; vm.echo(format!("<figcaption>{}</figcaption>", s)); Some(Ok(Value::Text(format!("<figcaption>{}</figcaption>", s)))) }
    else if code.starts_with("~html_mark(") && code.ends_with(')') { let s = get_text(vm, &code[11..code.len()-1])?; vm.echo(format!("<mark>{}</mark>", s)); Some(Ok(Value::Text(format!("<mark>{}</mark>", s)))) }
    else {
        None
    }
}
