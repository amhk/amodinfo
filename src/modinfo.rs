use anyhow::{anyhow, ensure, Result};
use serde::Deserialize;

pub struct ModuleInfo<'data> {
    // vector of (module-name, json-data), sorted by module-name
    data: Vec<(&'data str, &'data str)>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Module<'data> {
    #[serde(rename = "module_name")]
    pub name: &'data str,
    pub path: Vec<&'data str>,
    pub installed: Vec<&'data str>,
    pub dependencies: Option<Vec<&'data str>>,
    pub class: Vec<&'data str>,
    pub supported_variants: Option<Vec<&'data str>>,
    pub shared_libs: Option<Vec<&'data str>>,
    pub static_libs: Option<Vec<&'data str>>,
    pub system_shared_libs: Option<Vec<&'data str>>,
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
        ensure!(
            data.starts_with("{\n"),
            "unexpected start of module-info.json"
        );
        ensure!(data.ends_with("}\n"), "unexpected end of module-info.json");
        let data = &data[2..data.len() - 2];

        #[derive(PartialEq)]
        enum State {
            BeforeName,
            InsideName,
            BeforeJson,
            InsideJson,
        }

        let mut state = State::BeforeName;
        let mut delimiter = '"';
        let mut offsets = (0, 0, 0, 0);
        let mut out = Vec::with_capacity(10_000);
        for (i, ch) in data.chars().enumerate() {
            if ch != delimiter {
                continue;
            }
            match state {
                State::BeforeName => {
                    offsets.0 = i + 1;
                    state = State::InsideName;
                    delimiter = '"';
                }
                State::InsideName => {
                    offsets.1 = i - 1;
                    state = State::BeforeJson;
                    delimiter = '{';
                }
                State::BeforeJson => {
                    offsets.2 = i;
                    state = State::InsideJson;
                    delimiter = '}';
                }
                State::InsideJson => {
                    offsets.3 = i;
                    state = State::BeforeName;
                    delimiter = '"';

                    ensure!(
                        offsets.0 < offsets.1 && offsets.1 < offsets.2 && offsets.2 < offsets.3,
                        "module-json: internal error: {:?}",
                        offsets
                    );

                    let name = &data[offsets.0..=offsets.1];
                    let json = &data[offsets.2..=offsets.3];
                    out.push((name, json));
                }
            }
        }
        ensure!(state == State::BeforeName, "module-json: corrupt data");
        Ok(ModuleInfo { data: out })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODULE_INFO_LITE: &str = include_str!("../tests/data/module-info.json.lite");

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
        let modinfo = ModuleInfo::try_from(MODULE_INFO_LITE).unwrap();
        assert_eq!(modinfo.data.len(), 4);
    }

    #[test]
    fn test_module_names() {
        let modinfo = ModuleInfo::try_from(MODULE_INFO_LITE).unwrap();
        let names = modinfo.module_names();
        assert_eq!(names.len(), 4);
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
        assert!(names.contains(&"zxing-core"));
    }

    #[test]
    fn test_find() {
        let modinfo = ModuleInfo::try_from(MODULE_INFO_LITE).unwrap();
        let module = modinfo.find("zxing-core").unwrap().unwrap();
        assert_eq!(module.name, "zxing-core");
        assert_eq!(module.path, ["external/zxing"]);

        let module = modinfo.find("does-not-exist");
        assert!(module.is_none());
    }
}
