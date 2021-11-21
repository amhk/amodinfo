mod integration {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::error::Error;
    use std::process::Command;

    #[test]
    fn missing_command() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("amodinfo")?
            .assert()
            .failure()
            .stderr(predicate::str::contains("USAGE"));
        Ok(())
    }

    #[test]
    fn command_help() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("amodinfo")?
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains("USAGE"));
        Ok(())
    }

    #[test]
    fn command_list() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("amodinfo")?
            .arg("--module-info")
            .arg("tests/data/module-info.json")
            .arg("list")
            .assert()
            .success()
            .stdout(predicate::str::contains("idmap2\n"))
            .stdout(predicate::str::contains("libziparchive\n"));
        Ok(())
    }

    #[test]
    fn command_show() -> Result<(), Box<dyn Error>> {
        // missing argument: show
        Command::cargo_bin("amodinfo")?
            .arg("--module-info")
            .arg("tests/data/module-info.json")
            .arg("show")
            .assert()
            .failure();

        // no such module: show does-not-exist
        Command::cargo_bin("amodinfo")?
            .arg("--module-info")
            .arg("tests/data/module-info.json")
            .arg("show")
            .arg("does-not-exist")
            .assert()
            .failure();

        // correct: show idmap2
        Command::cargo_bin("amodinfo")?
            .arg("--module-info")
            .arg("tests/data/module-info.json")
            .arg("show")
            .arg("idmap2")
            .assert()
            .success()
            .stdout(predicate::str::contains("frameworks/base/cmds/idmap2"));

        Ok(())
    }

    #[test]
    fn implicit_module_info_path() -> Result<(), Box<dyn Error>> {
        Command::cargo_bin("amodinfo")?
            .env("ANDROID_PRODUCT_OUT", "tests/data")
            .arg("show")
            .arg("idmap2")
            .assert()
            .success()
            .stdout(predicate::str::contains("frameworks/base/cmds/idmap2"));
        Ok(())
    }
}
