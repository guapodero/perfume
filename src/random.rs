use rand::distr::{Distribution, Uniform};
use rand_chacha::{ChaCha12Rng, rand_core::SeedableRng};

/// this function is idempotent. given the same parameters, always returns the same result
pub fn randomized<'a>(slices: &'a [&str], rng_seed: u64) -> Vec<&'a str> {
    let mut rng = ChaCha12Rng::seed_from_u64(rng_seed);
    let mut idxs = Uniform::new(0, slices.len()).unwrap().sample_iter(&mut rng);
    let mut randomized: Vec<&str> = Vec::with_capacity(slices.len());

    // idxs is from a uniform distribution, but can sample the same value more than once
    // therefore a loop is needed to ensure that every word is eventually used
    while randomized.len() < slices.len() {
        let idx = idxs.next().unwrap();
        let word = &slices[idx];
        if !randomized.contains(word) {
            randomized.push(word);
        }
    }

    randomized
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::read_lines;

    #[test]
    fn test_randomized_is_idempotent() {
        let words: Vec<String> = read_lines("data/gerunds.txt")
            .unwrap()
            .map_while(Result::ok)
            .take(500)
            .collect();
        let words: Vec<&str> = words.iter().map(|w| &w[..]).collect();
        let rng_seed = 1442565572658396971;
        let mut last_result = vec![];
        for _ in 0..50 {
            let this_result = randomized(words.as_slice(), rng_seed);
            if !last_result.is_empty() {
                assert_eq!(this_result, last_result);
            }
            last_result = this_result;
        }
    }
}
