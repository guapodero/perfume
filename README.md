# perfume
Impromptu conversion of sensitive metadata to persistent random names.

[![Crates.io](https://img.shields.io/crates/v/perfume.svg)](https://crates.io/crates/perfume)
[![Docs.rs](https://docs.rs/perfume/badge.svg)](https://docs.rs/perfume)
[![CI](https://github.com/guapodero/perfume/workflows/CI/badge.svg)](https://github.com/guapodero/perfume/actions)
[![Rust GitHub Template](https://img.shields.io/badge/Rust%20GitHub-Template-blue)](https://rust-github.github.io/)

## Motivation

Most web applications need to distinguish between users in order to provide services, which is usually accomplished by assigning a unique number to each user. However, because it is impractical for humans to refer to such numbers, it becomes necessary to choose a unique username before using an application. The task of selecting yet another username is time consuming and distracting, which provides an incentive for users to compromise their own privacy by choosing a name that can be easily associated with them.

This library solves the problem by generating identifiers which are both unique and human readable. It also helps application providers mitigate the cost of data breaches by storing only ciphertext.

## Usage

See the [documentation](https://docs.rs/perfume) for an example to get started with. An implementation of the `ConnectionBridge` trait is necessary so that the generated values are persistent.

There is also some code generation involved, which relies on the use of a build script: 
https://doc.rust-lang.org/cargo/reference/build-scripts.html

## Example

```sh
export TMPDIR=/tmp
cargo run -F codegen

cargo run --example remote_store_ureq
# unraking-teal-muskrat
# outpleasing-rose-gelding
# reifying-navy-lab

export PERFUME_SECRET=51fX7DcodQ3C0hQQMYSp1W4jU05UEoNi
cargo run --example remote_store_ureq
# embruting-aqua-weevil
# curtsying-lime-cardinal
# lampblacking-purple-whitefly
```

### Word Lists

Although you are encouraged to create your own unique lists of seed words, this can consume a significant amount of time. There are some word lists in this repository to start with. If you choose to open a pull request containing a word list that you found useful, please update the list below with a detailed description.

* data/animals.txt 
  1057 small and medium nouns taken from the [petname](https://crates.io/crates/petname) crate.

* data/colors.txt 
  50 distinct, easily pronounced and recognized color names which are known to web browsers.
  
* data/gerunds.txt 
  4237 english [gerunds](https://en.wikipedia.org/wiki/Gerund) (ex. flowering bicycling freelancing). 
  Each gerund has 3 syllables, up to 15 letters, and is distinct by a [Levenshtein distance](https://en.wikipedia.org/wiki/Levenshtein_distance) of at least 3.
  
* data/words.txt
  370105 english words taken from the https://github.com/dwyl/english-words repo.

## Limitations

The persistence mechanism uses an application secret to generate a seeded hash value which always refers to the same random name. Therefore there is not a way to generate a list of random names to choose from without using multiple `Population` instances, each of which requires a unique application secret.

Only triplet *first-middle-last* names are generated. To ensure that names are not clustered into a small subset of possible *middle* names, the size of each `Population` must be declared. There are comments in the code about this, and you are welcome to open a pull request if you find a simple way to remove this restriction.

The generated names are not guaranteed to be unique, and collisions are possible. There is a unit test to check for collisions, which can probably be improved. After generating 1000 names, `test_distinct_names` reveals several clusters of size 3 which differ by only the *last* name.

This won't work well in distributed environments, as it relies on iteration through a single list of prepared names. This can be fixed in a future release. One option would be to assume that requests are split evenly over all nodes, it which case it becomes practical to shard the list of names.

## Related Work

There are a variety of projects on [crates.io](https://crates.io) dealing with random name generation. What makes this one different is that the functionality is exposed as a library. This library is designed specifically for the obfuscation of sensitive user information, and is mature enough for production use.

## Security

The security of this library is based on that of the [BLAKE3](https://crates.io/crates/blake3) cryptographic hash function. If an attacker gains access to the stored data that this library depends on, there will not be any way to determine the input or manipulate the output without access to the 256-bit encryption key which was used to generate the data. **If the encryption key is compromised, there will not be any way to rebuild the map using a different key.**

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
