use anyhow::{bail, Result};
use std::io::IsTerminal;

pub fn run() -> Result<()> {
    if !std::io::stdout().is_terminal() {
        bail!("cclens browse は対話端末でのみ利用できます");
    }
    Ok(())
}

pub fn head_lines(text: &str, n: usize) -> String {
    text.lines().take(n).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_head_lines_truncates() {
        let s = "a\nb\nc\nd\ne";
        assert_eq!(head_lines(s, 3), "a\nb\nc");
    }

    #[test]
    fn test_head_lines_fewer_than_n() {
        let s = "a\nb";
        assert_eq!(head_lines(s, 20), "a\nb");
    }
}
