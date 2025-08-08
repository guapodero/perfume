#[cfg(feature = "codegen")]
use perfume::codegen;

#[cfg(feature = "codegen")]
fn main() {
    // normally this is in build.rs
    // implemented for the purpose of automated testing
    codegen::ingredients(
        "PERFUME_INGREDIENTS",
        perfume::codegen::PopulationSize::Brazil,
        "data/gerunds.txt",
        "data/colors.txt",
        "data/animals.txt",
        "/tmp/perfume.rs",
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

#[cfg(not(feature = "codegen"))]
fn main() {}
