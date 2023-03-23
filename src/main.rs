use anyhow::Result;
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
                println!(
                    "Couldn't find a person with IMDB id {imdb_id}, are you sure that's correct?"
                )
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
    mother_wiki_link: Option<String>,
    father_wiki_link: Option<String>,
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
    if person_info.mother_wiki_link.is_some() && person_info.father_wiki_link.is_some() {
        Ok(NepoBabiness::Yes {
            father_wiki_link: person_info.father_wiki_link.clone().unwrap(),
            mother_wiki_link: person_info.mother_wiki_link.clone().unwrap(),
        })
    } else if person_info.mother_wiki_link.is_some() {
        Ok(NepoBabiness::OnlyMother {
            wiki_link: person_info.mother_wiki_link.clone().unwrap(),
        })
    } else if person_info.father_wiki_link.is_some() {
        Ok(NepoBabiness::OnlyFather {
            wiki_link: person_info.father_wiki_link.clone().unwrap(),
        })
    } else {
        Ok(NepoBabiness::No)
    }
}

async fn find_person_by_imdb_id(imdb_id: &str) -> Result<Option<PersonInfo>> {
    let api = mediawiki::api::Api::new("https://www.wikidata.org/w/api.php").await?; // Will determine the SPARQL API URL via site info data
    let query = format!(
        "SELECT ?person ?personLabel ?wikiLinkFather ?wikiLinkMother WHERE {{
  ?person wdt:P345 \"{imdb_id}\" .
  OPTIONAL {{
    ?person wdt:P22 ?father .
    ?wikiLinkFather schema:about ?father .
    ?wikiLinkFather schema:isPartOf <https://en.wikipedia.org/>
  }}
  OPTIONAL {{
    ?person wdt:P25 ?mother .
    ?wikiLinkMother schema:about ?mother .
    ?wikiLinkMother schema:isPartOf <https://en.wikipedia.org/>
  }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language \"en\". }}
}}
LIMIT 1"
    );

    let res = api.sparql_query(&query).await?;

    let people: Vec<_> = res
        .as_object()
        .and_then(|root| {
            Some(
                root.get("results")?
                    .get("bindings")?
                    .as_array()?
                    .iter()
                    .flat_map(|binding| {
                        let name = binding
                            .get("personLabel")?
                            .get("value")?
                            .as_str()?
                            .to_owned();
                        let father_wiki_link = binding
                            .get("wikiLinkFather")
                            .and_then(|x| x.get("value"))
                            .and_then(|x| x.as_str())
                            .map(|x| x.to_owned());
                        let mother_wiki_link = binding
                            .get("wikiLinkMother")
                            .and_then(|x| x.get("value"))
                            .and_then(|x| x.as_str())
                            .map(|x| x.to_owned());
                        Some(PersonInfo {
                            name,
                            mother_wiki_link,
                            father_wiki_link,
                        })
                    })
                    .collect(),
            )
        })
        .unwrap_or_default();

    Ok(people.into_iter().next())
}
