use anyhow::Result;
use chrono::{Datelike, Local};
use std::io::BufWriter;
use std::path::PathBuf;
use std::{fs::File, io::Write, iter::FromFn, ops::Range, time::Instant};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Fetching data...");

    let start = Instant::now();
    let mut people = Vec::new();

    /*
    Notes:
    1. Earliest known film actor is Max Linder, born in 1883, hence the year here
    2. For chunks, 4 was chosen experimentally. 5 was causing deserialization problems for reasons I don't (and don't want to) understand.
    */
    for years in count_down_year_ranges(4, 1883) {
        let this_chunk_of_people = collect_personal_data(&years).await?;
        println!(
            "Fetched {count} people for years {years:?}",
            count = this_chunk_of_people.len()
        );
        people.extend(this_chunk_of_people);
    }
    let duration = start.elapsed();
    println!("Done fetching. Took: {duration:?}");
    println!("Received total {} people.", people.len());

    // NOTE: sorting is here mostly to make diffs in the resulting generated file nicer on each regeneration
    println!("Keeping only items with imdb id starting with nm, and sorting the lot");
    let start = Instant::now();
    people.retain(|p| p.imdb_id.starts_with("nm"));
    people.sort_by_key(|p| p.imdb_id.as_str()[2..].parse::<usize>().unwrap());
    people.dedup_by_key(|p| p.imdb_id.as_str()[2..].parse::<usize>().unwrap());
    let duration = start.elapsed();
    println!("Done sorting. Took: {duration:?}");

    let target_file_path = get_people_library_module_path();
    let file = File::create(&target_file_path)?;
    let mut writer = BufWriter::new(file);

    println!("Writing to {target_file_path:?}...");
    let start = Instant::now();
    writeln!(&mut writer, "use phf::phf_map;")?;
    writeln!(&mut writer, "")?;
    writeln!(&mut writer, "use crate::StaticPersonData;")?;
    writeln!(&mut writer, "")?;
    writeln!(
        &mut writer,
        "pub static PEOPLE: phf::Map<&'static str, StaticPersonData> = phf_map! {{"
    )?;
    for person in &people {
        // debug repr of PersonData happens to be valid Rust code, so we can abuse that
        writeln!(
            &mut writer,
            "    {:?} => Static{:?},",
            person.imdb_id, person
        )?;
    }
    writeln!(&mut writer, "}};")?;
    writer.flush()?;

    let duration = start.elapsed();
    println!("Done writing. Took: {duration:?}");

    Ok(())
}

#[allow(dead_code)]
#[derive(Debug)]
struct PersonData {
    imdb_id: String,
    name: String,
    wiki_link: Option<String>,
    father_name: Option<String>,
    father_imdb_id: Option<String>,
    father_wiki_link: Option<String>,
    mother_name: Option<String>,
    mother_imdb_id: Option<String>,
    mother_wiki_link: Option<String>,
}

async fn collect_personal_data(years: &Range<i32>) -> Result<Vec<PersonData>> {
    let api = mediawiki::api::Api::new("https://www.wikidata.org/w/api.php").await?; // Will determine the SPARQL API URL via site info data
    let query = format!(
        "SELECT
  DISTINCT ?imdbId
  ?personLabel
  ?wikiLink
  ?fatherLabel
  ?fatherImdbId
  ?fatherWikiLink
  ?motherLabel
  ?motherImdbId
  ?motherWikiLink
WHERE
{{
  ?person wdt:P31 wd:Q5;
    wdt:P345 ?imdbId;
    wdt:P569 ?born .
  OPTIONAL {{
    ?wikiLink schema:about ?person .
    ?wikiLink schema:isPartOf <https://en.wikipedia.org/>
  }}
  OPTIONAL {{
    ?person wdt:P22 ?father .
    ?fatherWikiLink schema:about ?father .
    ?fatherWikiLink schema:isPartOf <https://en.wikipedia.org/>
    OPTIONAL {{ ?father wdt:P345 ?fatherImdbId . }}
  }}
  OPTIONAL {{
    ?person wdt:P25 ?mother .
    ?motherWikiLink schema:about ?mother .
    ?motherWikiLink schema:isPartOf <https://en.wikipedia.org/>
    OPTIONAL {{ ?mother wdt:P345 ?motherImdbId . }}
  }}
  FILTER (YEAR(?born) >= {start_year} && YEAR(?born) < {end_year})
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"en\" . }}
}}
",
        start_year = years.start,
        end_year = years.end
    );

    println!("Using query:\n{query}");

    Ok(api
        .sparql_query(&query)
        .await?
        .as_object()
        .and_then(|root| {
            Some(
                root.get("results")?
                    .get("bindings")?
                    .as_array()?
                    .iter()
                    .flat_map(|binding| {
                        let maybe_get_string =
                            |key| Some(binding.get(key)?.get("value")?.as_str()?.to_owned());

                        Some(PersonData {
                            name: maybe_get_string("personLabel")?,
                            imdb_id: maybe_get_string("imdbId")?,
                            wiki_link: maybe_get_string("wikiLink"),
                            mother_name: maybe_get_string("motherLabel"),
                            mother_imdb_id: maybe_get_string("motherImdbId"),
                            mother_wiki_link: maybe_get_string("motherWikiLink"),
                            father_name: maybe_get_string("fatherLabel"),
                            father_imdb_id: maybe_get_string("fatherImdbId"),
                            father_wiki_link: maybe_get_string("fatherWikiLink"),
                        })
                    })
                    .collect(),
            )
        })
        .unwrap_or_default())
}

fn get_people_library_module_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("people");
    path.push("src");
    path.push("people.rs");
    return path;
}

fn count_down_year_ranges(
    chunk_size: i32,
    last_year_to_be_included: i32,
) -> FromFn<impl FnMut() -> Option<Range<i32>>> {
    let mut year = Local::now().year() + 1;
    std::iter::from_fn(move || {
        year -= chunk_size;

        if year + chunk_size > last_year_to_be_included {
            Some(year..year + chunk_size)
        } else {
            None
        }
    })
}
