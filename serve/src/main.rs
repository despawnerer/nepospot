use std::collections::HashMap;

use lambda_http::{
    http::{header::CONTENT_TYPE, Method, StatusCode},
    service_fn, Error, IntoResponse, Request, RequestExt, Response,
};
use serde_json::json;

use nepospot_people::{StaticPersonData, PEOPLE};

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(nepospot)).await?;

    Ok(())
}

async fn nepospot(request: Request) -> Result<impl IntoResponse, Error> {
    if request.method() != Method::GET {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body("".to_owned())?);
    }

    let params = request.query_string_parameters();
    let imdb_id = match params.first("imdb_id") {
        Some(imdb_id) if imdb_id.starts_with("nm") => imdb_id,
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("".to_owned())?)
        }
    };

    Ok(match PEOPLE.get(imdb_id) {
        Some(person) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(
                json!({
                    "person": &person,
                    "is_nepo_baby": determine_nepo_babiness(&person)
                })
                .to_string(),
            )?,
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("".to_string())?,
    })
}

#[derive(Debug, serde::Serialize)]
enum NepoBabiness {
    No,
    OnlyMother,
    OnlyFather,
    Yes,
}

fn determine_nepo_babiness(person: &StaticPersonData) -> NepoBabiness {
    if person.mother_wiki_link.is_some() && person.father_wiki_link.is_some() {
        NepoBabiness::Yes
    } else if person.mother_wiki_link.is_some() {
        NepoBabiness::OnlyMother
    } else if person.father_wiki_link.is_some() {
        NepoBabiness::OnlyFather
    } else {
        NepoBabiness::No
    }
}

#[tokio::test]
async fn test_success() {
    let mut hash = HashMap::new();
    hash.insert("imdb_id".to_owned(), vec!["nm0001774".to_owned()]);

    let request = lambda_http::http::Request::builder()
        .method(Method::GET)
        .body(lambda_http::Body::Empty)
        .unwrap()
        .with_query_string_parameters(hash);

    let response = nepospot(request).await.unwrap().into_response().await;
    assert_eq!(response.body(), &lambda_http::Body::Text("".to_string()));
    assert_eq!(response.status(), StatusCode::OK);
}
