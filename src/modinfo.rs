use crate::error::ParseError;

pub struct ModuleInfo<'data> {
    data: Vec<(&'data str, &'data str)>,
}

impl<'data> ModuleInfo<'data> {
    pub fn module_names(&self) -> Vec<&'data str> {
        self.data.iter().map(|line| line.0).collect()
    }
}

impl<'data> TryFrom<&'data str> for ModuleInfo<'data> {
    type Error = ParseError;

    fn try_from(data: &'data str) -> Result<Self, Self::Error> {
        if !data.starts_with("{\n") {
            return Err(ParseError {
                lineno: 1,
                message: "bad first line",
            });
        }
        let mut lines = data.split_terminator('\n').skip(1).collect::<Vec<_>>();
        let n_lines = lines.len();
        if let Some(last) = lines.pop() {
            if last != "}" {
                return Err(ParseError {
                    lineno: n_lines,
                    message: "bad last line",
                });
            }
        } else {
            return Err(ParseError {
                lineno: 1,
                message: "too few lines",
            });
        }

        // check that each line is '  "name": { ... }[,]'
        // and split into "name" and "{ ... }"
        let mut data = Vec::with_capacity(lines.len());
        for (lineno, line) in lines.iter().enumerate().map(|pair| (pair.0 + 2, pair.1)) {
            if !line.starts_with("  \"") {
                return Err(ParseError {
                    lineno,
                    message: "no <name> element",
                });
            }
            let line = &line[3..];
            let name_end = line.find('"').ok_or(ParseError {
                lineno,
                message: "<name> element not terminated",
            })?;
            let name = &line[..name_end];

            let line = &line[name_end..];
            if !line.starts_with("\": {") {
                return Err(ParseError {
                    lineno,
                    message: "bad <name> terminator",
                });
            }
            let mut json = &line[3..];
            if json.ends_with(',') {
                json = &json[..json.len() - 1];
            }
            if !json.ends_with('}') {
                return Err(ParseError {
                    lineno,
                    message: "<json> element not terminated",
                });
            }

            data.push((name, json));
        }

        Ok(ModuleInfo { data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODULE_INFO: &str = include_str!("../tests/data/module-info.json");

    #[test]
    fn test_try_from() {
        // entirely wrong input
        assert!(ModuleInfo::try_from("").is_err());
        assert!(ModuleInfo::try_from("foo").is_err());
        assert!(ModuleInfo::try_from("{").is_err());

        // corrupt input
        assert!(ModuleInfo::try_from("{\n  foo\": { ... }\n}\n").is_err());
        assert!(ModuleInfo::try_from("{\n  \"foo: { ... }\n}\n").is_err());
        assert!(ModuleInfo::try_from("{\n  \"foo\": ... }\n}\n").is_err());
        assert!(ModuleInfo::try_from("{\n  \"foo\": { ... \n}\n").is_err());

        // correct input
        let modinfo = ModuleInfo::try_from("{\n}\n").unwrap();
        assert_eq!(modinfo.data.len(), 0);

        let modinfo = ModuleInfo::try_from("{\n  \"foo\": { ... }\n}\n").unwrap();
        assert_eq!(modinfo.data.len(), 1);

        let modinfo = ModuleInfo::try_from(MODULE_INFO).unwrap();
        assert_eq!(modinfo.data.len(), 52638);
    }

    #[test]
    fn test_module_names() {
        let modinfo = ModuleInfo::try_from(MODULE_INFO).unwrap();
        let names = modinfo.module_names();
        assert_eq!(names.len(), 52638);
        assert!(names.contains(&"idmap2"));
    }
}
