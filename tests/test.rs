use std::env;
use std::fs::read_dir;
use std::io;
use std::path::PathBuf;

use pretty_assertions::assert_str_eq;

struct TestFixture {
    fixture: String,
    code_file: PathBuf,
    ast_file: PathBuf,
    error_file: PathBuf,
}

struct ExpectedTestResult {
    ast: String,
    error: String,
}

impl TestFixture {
    fn new(entry: PathBuf) -> Self {
        Self {
            fixture: entry.to_string_lossy().to_string(),
            code_file: entry.join("code.php"),
            ast_file: entry.join("ast.txt"),
            error_file: entry.join("error.txt"),
        }
    }

    fn code(&self) -> String {
        std::fs::read_to_string(&self.code_file).unwrap_or_default()
    }

    fn validate(&self) -> io::Result<()> {
        if !self.code_file.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Code file {} does not exist",
                    self.code_file.to_string_lossy()
                ),
            ));
        }

        if !self.ast_file.exists() && !self.error_file.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Test fixture {} must have either an ast.txt or error.txt file",
                    self.fixture
                ),
            ));
        }

        Ok(())
    }

    fn expected(&self) -> ExpectedTestResult {
        let ast = std::fs::read_to_string(&self.ast_file).unwrap_or_default();
        let error = std::fs::read_to_string(&self.error_file).unwrap_or_default();

        ExpectedTestResult { ast, error }
    }
}

#[test]
fn test_fixtures() -> io::Result<()> {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let tests = manifest.join("tests/fixtures");

    let mut entries = read_dir(tests)?
        .flatten()
        .map(|entry| entry.path())
        .filter(|entry| entry.is_dir())
        .collect::<Vec<PathBuf>>();

    entries.sort();

    for entry in entries {
        let test_fixture = TestFixture::new(entry);

        test_fixture.validate()?;

        run_test(&test_fixture)?;
    }

    Ok(())
}

fn run_test(test_fixture: &TestFixture) -> io::Result<()> {
    let code = test_fixture.code();
    let expected = test_fixture.expected();

    if !expected.ast.is_empty() {
        let ast = php_parser_rs::parse(&code).unwrap();
        assert_str_eq!(
            expected.ast.trim(),
            format!("{:#?}", ast),
            "ast mismatch for fixture `{}`",
            test_fixture.fixture
        );
    }

    if !expected.error.is_empty() {
        let error = php_parser_rs::parse(&code).err().unwrap();

        assert_str_eq!(
            expected.error.trim(),
            (error.report(&code, Some("code.php"), false, true)?).trim(),
            "error mismatch for fixture `{}`",
            test_fixture.fixture
        );
    }

    Ok(())
}
