use serde::Deserialize;

use crate::error::ParseError;

pub struct ModuleInfo<'data> {
    data: Vec<(&'data str, &'data str)>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Module<'data> {
    #[serde(rename = "module_name")]
    pub name: &'data str,
    pub path: Vec<&'data str>,
    pub installed: Vec<&'data str>,
    pub dependencies: Vec<&'data str>,
    pub class: Vec<&'data str>,
    pub tags: Vec<&'data str>,
    pub test_config: Vec<&'data str>,
}

impl<'data> ModuleInfo<'data> {
    pub fn module_names(&self) -> Vec<&'data str> {
        self.data.iter().map(|line| line.0).collect()
    }

    pub fn find(&self, name: &str) -> Option<Result<Module, ParseError>> {
        let index = self.data.binary_search_by(|pair| pair.0.cmp(name)).ok()?;
        let json = self.data[index].1;
        let x = serde_json::from_str(json).map_err(|e| ParseError {
            lineno: index + 2, // offset by two: omitted inital line + start counting from 1
            message: format!("bad JSON: {}", e),
        });
        Some(x)
    }
}

impl<'data> TryFrom<&'data str> for ModuleInfo<'data> {
    type Error = ParseError;

    fn try_from(data: &'data str) -> Result<Self, Self::Error> {
        if !data.starts_with("{\n") {
            return Err(ParseError {
                lineno: 1,
                message: "bad first line".to_string(),
            });
        }
        let mut lines = data.split_terminator('\n').skip(1).collect::<Vec<_>>();
        let n_lines = lines.len();
        if let Some(last) = lines.pop() {
            if last != "}" {
                return Err(ParseError {
                    lineno: n_lines,
                    message: "bad last line".to_string(),
                });
            }
        } else {
            return Err(ParseError {
                lineno: 1,
                message: "too few lines".to_string(),
            });
        }

        // split '  "name": { ... }[,]' into ('name', '{ ... }')
        let mut data = Vec::with_capacity(lines.len());
        for (lineno, line) in lines.iter().enumerate().map(|pair| (pair.0 + 2, pair.1)) {
            if !line.starts_with("  \"") {
                return Err(ParseError {
                    lineno,
                    message: "no <name> element".to_string(),
                });
            }
            let line = &line[3..];
            let name_end = line.find('"').ok_or(ParseError {
                lineno,
                message: "<name> element not terminated".to_string(),
            })?;
            let name = &line[..name_end];

            let line = &line[name_end..];
            if !line.starts_with("\": {") {
                return Err(ParseError {
                    lineno,
                    message: "bad <name> terminator".to_string(),
                });
            }
            let mut json = &line[3..];
            if json.ends_with(',') {
                json = &json[..json.len() - 1];
            }
            if !json.ends_with('}') {
                return Err(ParseError {
                    lineno,
                    message: "<json> element not terminated".to_string(),
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

    #[test]
    fn test_find() {
        let modinfo = ModuleInfo::try_from("{\n  \"foo\": { ... }\n}\n").unwrap();
        let module = modinfo.find("foo");
        assert!(module.is_some());
        let module = module.unwrap();
        assert!(module.is_err());

        let modinfo = ModuleInfo::try_from(MODULE_INFO).unwrap();
        let module = modinfo.find("idmap2").unwrap().unwrap();
        assert_eq!(module.name, "idmap2");

        let module = modinfo.find("does-not-exist");
        assert!(module.is_none());
    }
}
