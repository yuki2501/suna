use std::env;

pub fn expand_env_vars(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }

        // ${VAR} 形式
        if matches!(chars.peek(), Some('{')) {
            chars.next(); // '{'

            let mut name = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '}' {
                    chars.next(); // '}'
                    break;
                }
                name.push(ch);
                chars.next();
            }

            if name.is_empty() {
                // "${}" はそのまま残す
                out.push_str("${}");
                continue;
            }

            let val = env::var(&name).unwrap_or_else(|_| {
                panic!("undefined env var {name} in {s:?}");
            });
            out.push_str(&val);
            continue;
        }

        // $VAR 形式
        // 先頭は [A-Za-z_] だけ許す
        let first = match chars.peek() {
            Some(&ch) if ch == '_' || ch.is_ascii_alphabetic() => ch,
            _ => {
                // "$" の直後にまともな名前がなければ "$" をそのまま出す
                out.push('$');
                continue;
            }
        };
        let mut name = String::new();
        name.push(first);
        chars.next(); // 先頭は自分で消費した

        while let Some(&ch) = chars.peek() {
            if ch == '_' || ch.is_ascii_alphanumeric() {
                name.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        let val = env::var(&name).unwrap_or_else(|_| {
            panic!("undefined env var {name} in {s:?}");
        });
        out.push_str(&val);
    }

    out
}
