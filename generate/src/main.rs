use anyhow::Result;
use chrono::{Datelike, Local};
use std::{iter::FromFn, ops::Range, time::Instant};

use nepospot_library::{get_nepos_csv_path, PersonData};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Fetching data...");

    let start = Instant::now();
    let mut people = Vec::new();
    // earliest known film actor is Max Linder, born in 1883
    for years in count_down_year_ranges(5, 1883) {
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

    let target_file_path = get_nepos_csv_path();
    println!("Writing to {target_file_path:?}...");
    let mut writer = csv::Writer::from_path(&target_file_path)?;
    for person in &people {
        writer.serialize(&person)?;
    }
    writer.flush()?;

    Ok(())
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
    ?father wdt:P345 ?fatherImdbId .
    ?fatherWikiLink schema:about ?father .
    ?fatherWikiLink schema:isPartOf <https://en.wikipedia.org/>
  }}
  OPTIONAL {{
    ?person wdt:P25 ?mother .
    ?mother wdt:P345 ?motherImdbId .
    ?motherWikiLink schema:about ?mother .
    ?motherWikiLink schema:isPartOf <https://en.wikipedia.org/>
  }}
  FILTER (?born >= \"{start_year}-01-01\"^^xsd:dateTime && ?born < \"{end_year}-01-01\"^^xsd:dateTime)
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"en\" . }}
}}
",
    start_year=years.start, end_year=years.end);

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
