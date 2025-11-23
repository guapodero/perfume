#[cfg(feature = "codegen")]
use perfume::codegen;

#[cfg(feature = "codegen")]
fn main() {
    let tmp_dir = std::env::var("TMPDIR").unwrap_or("/tmp".to_string());
    let output_path = format!("{tmp_dir}/perfume.rs");

    // normally this is in build.rs
    // implemented for the purpose of automated testing
    codegen::ingredients(
        "PERFUME_INGREDIENTS",
        perfume::codegen::PopulationSize::Brazil,
        "data/gerunds.txt",
        "data/colors.txt",
        "data/animals.txt",
        output_path,
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

#[cfg(not(feature = "codegen"))]
fn main() {}
