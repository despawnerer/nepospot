# Nepospot

Finder of nepo babies.

## Prerequsites

* [Rust](https://www.rust-lang.org/)

## Structure

### [`data`](data)

- `nepos.csv` containing a CSV list of people with their parents information

### [`generate`](generate)

Binary crate that regenerates `data/nepos.csv` from wikidata.

### [`serve`](serve)

Lambda that uses information from `nepos.csv` to determine whether somebody is a nepo baby or not.

### [`library`](library)

Library containing common functions and data types used by `generate` and `serve` crates.

## Deployment

Refer to [`serve` module's README](serve/README.md)
