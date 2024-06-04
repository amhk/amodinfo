use anyhow::{anyhow, bail, ensure, Result};
use serde::Deserialize;

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

    pub fn find(&self, name: &str) -> Option<Result<Module>> {
        let index = self.data.binary_search_by(|pair| pair.0.cmp(name)).ok()?;
        let json = self.data[index].1;
        let x = serde_json::from_str(json).map_err(|e| anyhow!("bad JSON: {}", e));
        Some(x)
    }
}

impl<'data> TryFrom<&'data str> for ModuleInfo<'data> {
    type Error = anyhow::Error;

    fn try_from(data: &'data str) -> Result<Self> {
        ensure!(data.starts_with("{\n"), "bad first line");
        let mut lines = data.split_terminator('\n').skip(1).collect::<Vec<_>>();
        if let Some(last) = lines.pop() {
            ensure!(last == "}", "bad last line");
        } else {
            bail!("too few lines");
        }

        // split ' "name": { ... }[,]' into ('name', '{ ... }')
        let mut data = Vec::with_capacity(lines.len());
        for (lineno, line) in lines.iter().enumerate().map(|pair| (pair.0 + 1, pair.1)) {
            ensure!(line.starts_with(" \""), "{}: no <name> element", lineno);
            let line = &line[2..];
            let name_end = line
                .find('"')
                .ok_or_else(|| anyhow!("{}: <name> element not terminated", lineno))?;
            let name = &line[..name_end];

            let line = &line[name_end..];
            ensure!(
                line.starts_with("\": {"),
                "{}: bad <name> terminator",
                lineno
            );
            let mut json = &line[3..];
            if json.ends_with(',') {
                json = &json[..json.len() - 1];
            }
            ensure!(
                json.ends_with('}'),
                "{}: <json> element not terminated",
                lineno
            );

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
        assert!(ModuleInfo::try_from("{\n foo\": { ... }\n}\n").is_err());
        assert!(ModuleInfo::try_from("{\n \"foo: { ... }\n}\n").is_err());
        assert!(ModuleInfo::try_from("{\n \"foo\": ... }\n}\n").is_err());
        assert!(ModuleInfo::try_from("{\n \"foo\": { ... \n}\n").is_err());

        // correct input
        let modinfo = ModuleInfo::try_from("{\n}\n").unwrap();
        assert_eq!(modinfo.data.len(), 0);

        let modinfo = ModuleInfo::try_from("{\n \"foo\": { ... }\n}\n").unwrap();
        assert_eq!(modinfo.data.len(), 1);

        let modinfo = ModuleInfo::try_from(MODULE_INFO).unwrap();
        assert_eq!(modinfo.data.len(), 64225);
    }

    #[test]
    fn test_module_names() {
        let modinfo = ModuleInfo::try_from(MODULE_INFO).unwrap();
        let names = modinfo.module_names();
        assert_eq!(names.len(), 64225);
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
        assert!(names.contains(&"idmap2"));
    }

    #[test]
    fn test_find() {
        let modinfo = ModuleInfo::try_from("{\n \"foo\": { ... }\n}\n").unwrap();
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
