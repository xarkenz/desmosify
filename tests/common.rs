use std::path::Path;
use std::process::ExitCode;

pub fn run_test(
    source_paths: impl IntoIterator<Item = impl AsRef<Path>>,
    output_path: impl AsRef<Path>,
    target_name: impl AsRef<str>,
) {
    let args = desmosify::cli::DesmosifyArgs {
        source_paths: source_paths
            .into_iter()
            .map(|path| Path::new("tests").join(path))
            .collect(),
        output_path: Path::new("tests").join(output_path),
        target_name: target_name.as_ref().to_string(),
        fragile_strategy: Default::default(),
    };

    if desmosify::cli::invoke_wrapper(&args) != ExitCode::SUCCESS {
        panic!("compiler invocation failed")
    }
}

#[macro_export]
macro_rules! simple_test {
    ($test:ident, $name:literal, $target:expr) => {
        #[test]
        fn $test() {
            common::run_test(
                [concat!($name, "/main.desmos")],
                concat!($name, "/out/", $name, ".json"),
                $target,
            )
        }
    };
}
