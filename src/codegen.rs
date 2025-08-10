//! Compile data to use for creating a [`crate::identity::Population`].

use std::cmp::max;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::RangeInclusive;
use std::path::Path;

use crate::random::randomized;
use crate::{Error, STORAGE_KEY_LENGTH, read_lines};

/// The number of possible identities, chosen only once.
/// This is necessary to ensure that each color ingredient is used in equal amount.
#[derive(Copy, Clone)]
pub enum PopulationSize {
    /// Up to 178 identities per storage blob.
    Bhutan = 727_145,
    /// Up to 2867 (191KB) per storage blob.
    Belgium = 11_742_796,
    /// Up to 49581 (3.2MB) per storage blob.
    Brazil = 203_080_756,
}

/// Compile words from `prefixes`, `colors` and `animals` files into `output` file.
/// The resulting static item will be named using `static_name`.
///
/// Returns a [`crate::Error::Codegen`] error if any of the input files contain an
/// insufficient number of words to generate a Population of size `size`.
pub fn ingredients<P1, P2>(
    static_name: &str,
    size: PopulationSize,
    prefixes: P1,
    colors: P1,
    animals: P1,
    output: P2,
) -> Result<(), Error>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let prefixes_path: &Path = prefixes.as_ref();
    let colors_path: &Path = colors.as_ref();
    let animals_path: &Path = animals.as_ref();
    let output_path: &Path = output.as_ref();

    // each prefix will be mapped to a different storage key (see storage.rs)
    let required_prefixes = 16u32.pow(STORAGE_KEY_LENGTH as u32);
    let prefix_count = count_lines(prefixes_path)?;
    if prefix_count < required_prefixes {
        return Err(Error::Codegen(format!(
            "insufficient seed words. {}. {}",
            format_args!("{prefixes_path:#?} ({prefix_count} words)"),
            format_args!(
                "{} words available, but {} needed",
                prefix_count, required_prefixes
            )
        )));
    }

    // within each storage blob,
    // each storage digest will be mapped to a different (color, animal)
    let required_color_animals = size as u32 / required_prefixes;
    let color_count = count_lines(colors_path)?;
    let animal_count = count_lines(animals_path)?;
    if required_color_animals > color_count * animal_count {
        return Err(Error::Codegen(format!(
            "insufficient seed words. {}. {}",
            format_args!(
                "{colors_path:#?} ({} words), {animals_path:#?} ({} words)",
                color_count, animal_count
            ),
            format_args!(
                "{} combinations available, but {} needed",
                color_count * animal_count,
                required_color_animals
            )
        )));
    }

    let mut output_writer = BufWriter::new(File::create(output_path).unwrap());
    writeln!(output_writer, "#[allow(dead_code)]")?;
    writeln!(output_writer, "pub static {}:", static_name.to_uppercase())?;
    // there are unit tests which depend on this generated code
    // which can not reference types which are also required by codegen (avoiding separate codegen crate)
    writeln!(
        output_writer,
        "(usize, phf::Map<&str, &str>, &[&str], &[&str]) = ("
    )?;
    writeln!(output_writer, "{},", size as usize)?;
    write_prefixes(prefixes_path, &mut output_writer)?;
    write_words(colors_path, &mut output_writer)?;
    write_words(animals_path, &mut output_writer)?;
    writeln!(output_writer, ");")?;

    Ok(())
}

fn write_prefixes(input: &Path, output: &mut BufWriter<File>) -> Result<(), Error> {
    // generate a list of all possible storage keys
    let hex_digits = "0123456789abcdef".chars().collect::<Vec<_>>();
    let mut hex_keys = vec![];
    find_combinations(
        STORAGE_KEY_LENGTH..=STORAGE_KEY_LENGTH,
        hex_digits.as_slice(),
        &mut hex_keys,
    );

    // randomly select a word to associate with each key
    // rng_seed is hardcoded here to prevent accidental misuse
    let rng_seed = 656437432927126634;
    let prefix_words = read_lines(input)?
        .map_while(Result::ok)
        .take(hex_keys.len())
        .collect::<Vec<String>>();
    let prefix_words = prefix_words.iter().map(|w| &w[..]).collect::<Vec<&str>>();
    let prefix_words = randomized(prefix_words.as_slice(), rng_seed);
    assert_eq!(hex_keys.len(), prefix_words.len());

    let mut map = &mut phf_codegen::Map::<&'static str>::new();
    for (k, v) in hex_keys.iter().zip(prefix_words.iter()) {
        map = map.entry(k, format!("\"{v}\""));
    }

    writeln!(output, "{},", map.build())?;

    Ok(())
}

fn write_words(input: &Path, output: &mut BufWriter<File>) -> Result<(), Error> {
    let input = read_lines(input)?.map_while(Result::ok);
    writeln!(output, "&[")?;
    for word in input {
        writeln!(output, "  \"{word}\",")?;
    }
    writeln!(output, "],")?;
    Ok(())
}

// update `results` with
// a list of all possible strings having a length from `lengths`, and characters from `chars`.
fn find_combinations(lengths: RangeInclusive<usize>, chars: &[char], results: &mut Vec<String>) {
    match (lengths.start(), lengths.end()) {
        (1, 1) => {
            results.append(&mut chars.iter().map(|c| c.to_string()).collect::<Vec<_>>());
        }
        (&start, &end) => {
            // for each desired length,
            // collect all combinations which are shorter by at least 1 character
            let mut seed_results = vec![];
            find_combinations(
                max(1, start - 1)..=max(1, end - 1),
                chars,
                &mut seed_results,
            );

            // for len < 2, keep combinations from seed_results
            // for len >= 2, combinations are created by extending each seed by 1 character
            let mut next_results: Vec<String> = if start == 1 {
                seed_results
                    .iter()
                    .filter_map(|s| if s.len() < 2 { Some(s.clone()) } else { None })
                    .collect()
            } else {
                vec![]
            };

            // create remaining combinations by
            // appending each character to each shorter combination
            for comb in seed_results.iter() {
                for c in chars {
                    let mut next = comb.clone();
                    next.push(*c);
                    next_results.push(next);
                }
            }

            results.append(&mut next_results);
        }
    }
}

fn count_lines(file: &Path) -> Result<u32, std::io::Error> {
    match count_lines::count_lines_exact(file) {
        Ok(count) => Ok(count as u32),
        Err(e) => Err(e
            .downcast::<std::io::Error>()
            .expect("count_lines_exact should produce io::Error")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_combinations_base() {
        let mut result = vec![];
        find_combinations(1..=1, &['a', 'b', 'c'], &mut result);
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_find_combinations_inductive() {
        let mut result = vec![];
        find_combinations(1..=2, &['a', 'b', 'c'], &mut result);
        assert_eq!(
            result,
            vec![
                "a", "b", "c", "aa", "ab", "ac", "ba", "bb", "bc", "ca", "cb", "cc"
            ]
        );
    }
}
