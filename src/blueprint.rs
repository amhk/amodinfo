use anyhow::Result;
use regex::Regex;

#[allow(dead_code)]
pub fn find_module_source<'h>(haystack: &'h str, name: &str) -> Result<Option<&'h str>> {
    let regex_module = Regex::new(r"(?ms)[ \t]*[_a-zA-Z0-9]+\s*\{.*?^\}")?;
    let regex_name = Regex::new(&format!(r#"(?m)^\s*name:\s*"{}""#, name))?;
    for cap in regex_module.captures_iter(haystack) {
        let match_ = cap.get(0).unwrap();
        if regex_name.is_match(match_.as_str()) {
            return Ok(Some(&haystack[match_.range()]));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLUEPRINT: &str = include_str!("../tests/data/Android.bp");

    #[test]
    fn test_find_module_source() {
        assert!(find_module_source("", "").unwrap().is_none());
        assert!(find_module_source(BLUEPRINT, "none").unwrap().is_none());
        let source = find_module_source(BLUEPRINT, "idmap2").unwrap().unwrap();
        assert!(source.starts_with("cc_binary {\n    name: \"idmap2\",\n"));
        assert!(source.ends_with("},\n\n}"));
    }
}
