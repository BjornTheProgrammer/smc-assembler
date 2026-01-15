use smc_assembler::{assembler::backends::Backend, compile, convert_to_tau};
use std::{fs, path::PathBuf};

pub fn test_compilation(program_name: &str) {
    use pretty_assertions::assert_eq;

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(format!("tests/tau/programs/{}.tasm", program_name));

    let mut expected_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    expected_path.push(format!("tests/tau/expected/{}.tau", program_name));

    let result =
        compile(path, Backend::TauAnalyzersNone, false).expect("compilation should succeed");
    let mc_output = convert_to_tau(result).expect("conversion to mc should succeed");

    assert_eq!(
        mc_output,
        fs::read_to_string(expected_path).expect("Should be able to read the expected output")
    );
}

#[test]
fn compiles_ball() {
    test_compilation("ball");
}

#[test]
fn compiles_tetris() {
    test_compilation("tetris");
}
