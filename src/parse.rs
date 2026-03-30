/// Split a line into tokens, respecting single- and double-quoted spans (basic shell-like).
/// Does not implement escapes, globs, or redirection.
pub fn parse_line(line: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_dq = false;
    let mut in_sq = false;

    for ch in line.chars() {
        match ch {
            ' ' | '\t' if !in_dq && !in_sq => {
                if !cur.is_empty() {
                    out.push(cur);
                    cur = String::new();
                }
            }
            '"' if !in_sq => {
                if in_dq {
                    out.push(cur);
                    cur = String::new();
                    in_dq = false;
                } else {
                    in_dq = true;
                }
            }
            '\'' if !in_dq => {
                if in_sq {
                    out.push(cur);
                    cur = String::new();
                    in_sq = false;
                } else {
                    in_sq = true;
                }
            }
            c => cur.push(c),
        }
    }

    if in_dq || in_sq {
        return Err("unclosed quote".to_string());
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    Ok(out)
}
