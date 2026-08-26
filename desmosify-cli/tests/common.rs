use std::path::Path;
use std::process::ExitCode;

pub fn run_test(
    source_paths: impl IntoIterator<Item = impl AsRef<Path>>,
    output_path: impl AsRef<Path>,
    target_name: impl Into<String>,
) {
    let args = desmosify_cli::CommandLineArguments {
        source_paths: source_paths
            .into_iter()
            .map(|path| Path::new("tests").join(path))
            .collect(),
        output_path: Path::new("tests").join(output_path),
        compile_options: desmosify::CompileOptions::default_for_target(target_name),
    };

    if desmosify_cli::invoke(&args) != ExitCode::SUCCESS {
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
