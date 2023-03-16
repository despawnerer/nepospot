use anyhow::Result;
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let imdb_ids: Vec<String> = env::args().collect();
    for imdb_id in imdb_ids.iter().skip(1) {
        let judgement = get_nepo_babiness_judgment(imdb_id).await?;

        match judgement {
            Some(Judgement {
                person_info,
                nepo_babiness,
            }) => {
                println!(
                    "Person with IMDB id {imdb_id} appears to be {}",
                    person_info.name
                );
                match nepo_babiness {
                    NepoBabiness::No => println!("They're not a nepo baby: neither of their parents have a wikipedia page"),
                    NepoBabiness::OnlyMother { wiki_link } => println!("They might be a nepo baby. Their mother has a wiki page: {wiki_link}"),
                    NepoBabiness::OnlyFather { wiki_link } => println!("They might be a nepo baby. Their father has a wiki page: {wiki_link}"),
                    NepoBabiness::Yes { father_wiki_link, mother_wiki_link } => println!("They're a nepo baby.\nMeet the parents:\n- {mother_wiki_link}\n- {father_wiki_link}"),
                }
            }
            None => {
                println!("Couldn't find a person with IMDB id {imdb_id}, are you sure that's correct?")
            }
        }

    }

    Ok(())
}

#[derive(Debug)]
struct Judgement {
    person_info: PersonInfo,
    nepo_babiness: NepoBabiness,
}

#[derive(Debug)]
struct PersonInfo {
    name: String,
    mother_id: Option<String>,
    father_id: Option<String>,
    wiki_link: Option<String>,
}

#[derive(Debug)]
enum NepoBabiness {
    No,
    OnlyMother {
        wiki_link: String,
    },
    OnlyFather {
        wiki_link: String,
    },
    Yes {
        father_wiki_link: String,
        mother_wiki_link: String,
    },
}

async fn get_nepo_babiness_judgment(imdb_id: &str) -> Result<Option<Judgement>> {
    let maybe_person = find_person_by_imdb_id(imdb_id).await?;
    if maybe_person.is_none() {
        return Ok(None);
    }

    let person = maybe_person.unwrap();
    let nepo_babiness = determine_nepo_babiness(&person).await?;

    Ok(Some(Judgement {
        person_info: person,
        nepo_babiness,
    }))
}

async fn determine_nepo_babiness(person_info: &PersonInfo) -> Result<NepoBabiness> {
    // doesn't have a wiki link, not a nepo baby
    if person_info.wiki_link.is_none() {
        return Ok(NepoBabiness::No);
    }

    // parents don't have imdb ids, not a nepo baby
    if person_info.mother_id.is_none() && person_info.father_id.is_none() {
        return Ok(NepoBabiness::No);
    }

    let mother_wiki_link = if person_info.mother_id.is_some() {
        find_person_by_wikidata_id(&person_info.mother_id.as_ref().unwrap())
            .await?
            .and_then(|mother| mother.wiki_link)
    } else {
        None
    };

    let father_wiki_link = if person_info.father_id.is_some() {
        find_person_by_wikidata_id(&person_info.father_id.as_ref().unwrap())
            .await?
            .and_then(|father| father.wiki_link)
    } else {
        None
    };

    if mother_wiki_link.is_some() && father_wiki_link.is_some() {
        Ok(NepoBabiness::Yes {
            father_wiki_link: father_wiki_link.unwrap(),
            mother_wiki_link: mother_wiki_link.unwrap(),
        })
    } else if mother_wiki_link.is_some() {
        Ok(NepoBabiness::OnlyMother {
            wiki_link: mother_wiki_link.unwrap(),
        })
    } else if father_wiki_link.is_some() {
        Ok(NepoBabiness::OnlyFather {
            wiki_link: father_wiki_link.unwrap(),
        })
    } else {
        Ok(NepoBabiness::No)
    }
}

async fn find_person_by_imdb_id(imdb_id: &str) -> Result<Option<PersonInfo>> {
    let api = mediawiki::api::Api::new("https://www.wikidata.org/w/api.php").await?; // Will determine the SPARQL API URL via site info data
    let query = format!(
        "SELECT ?itemLabel ?mother ?father ?wikiLink WHERE {{
  ?item wdt:P345 \"{imdb_id}\".

  OPTIONAL{{ ?item wdt:P25 ?mother .}}
  OPTIONAL{{ ?item wdt:P22 ?father .}}
  OPTIONAL{{ ?wikiLink schema:about ?item . ?wikiLink schema:isPartOf <https://en.wikipedia.org/> }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"en\". }}
}}
LIMIT 1"
    );

    Ok(parse_person_info(api.sparql_query(&query).await?))
}

async fn find_person_by_wikidata_id(id: &str) -> Result<Option<PersonInfo>> {
    let api = mediawiki::api::Api::new("https://www.wikidata.org/w/api.php").await?; // Will determine the SPARQL API URL via site info data
    let query = format!(
        "SELECT ?itemLabel ?mother ?father ?wikiLink WHERE {{
  ?item ?p ?s

  OPTIONAL{{ ?item wdt:P25 ?mother .}}
  OPTIONAL{{ ?item wdt:P22 ?father .}}
  OPTIONAL{{ ?wikiLink schema:about ?item . ?wikiLink schema:isPartOf <https://en.wikipedia.org/> }}

  FILTER(?item = wd:{id})

  SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"en\". }}
}}
LIMIT 1"
    );

    Ok(parse_person_info(api.sparql_query(&query).await?))
}

fn parse_person_info(res: Value) -> Option<PersonInfo> {
    let people: Vec<_> = res
        .as_object()
        .and_then(|root| {
            Some(
                root.get("results")?
                    .get("bindings")?
                    .as_array()?
                    .iter()
                    .flat_map(|binding| {
                        let name = binding.get("itemLabel")?.get("value")?.as_str()?.to_owned();
                        let father_id = binding
                            .get("father")
                            .and_then(|x| x.get("value"))
                            .and_then(|x| x.as_str())
                            .and_then(|x| x.strip_prefix("http://www.wikidata.org/entity/"))
                            .map(|x| x.to_owned());
                        let mother_id = binding
                            .get("mother")
                            .and_then(|x| x.get("value"))
                            .and_then(|x| x.as_str())
                            .and_then(|x| x.strip_prefix("http://www.wikidata.org/entity/"))
                            .map(|x| x.to_owned());
                        let wiki_link = binding
                            .get("wikiLink")
                            .and_then(|x| x.get("value"))
                            .and_then(|x| x.as_str())
                            .map(|x| x.to_owned());
                        Some(PersonInfo {
                            name,
                            mother_id,
                            father_id,
                            wiki_link,
                        })
                    })
                    .collect(),
            )
        })
        .unwrap_or_default();

    people.into_iter().next()
}
