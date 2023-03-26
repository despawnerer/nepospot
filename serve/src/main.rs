use std::collections::HashMap;

use lambda_http::{
    http::{header::CONTENT_TYPE, Method, StatusCode},
    service_fn, Error, IntoResponse, Request, RequestExt, Response,
};
use once_cell::sync::OnceCell;
use serde_json::json;

use nepospot_library::{determine_nepo_babiness, PersonData};

static NEPOS_CSV: &[u8] = include_bytes!("../../data/nepos.csv");
static NEPOS_DATA: OnceCell<HashMap<String, PersonData>> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    initialize_nepos_data()?;

    lambda_http::run(service_fn(nepospot)).await?;

    Ok(())
}

fn initialize_nepos_data() -> Result<(), Error> {
    let mut data = HashMap::new();
    let mut reader = csv::Reader::from_reader(NEPOS_CSV);
    for record in reader.deserialize() {
        let record: PersonData = record?;
        data.insert(record.imdb_id.clone(), record);
    }
    NEPOS_DATA
        .set(data)
        .expect("Initial nepos data was set twice, what gives?");
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
        Some(imdb_id) => imdb_id,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("".to_owned())?)
        }
    };

    Ok(
        match NEPOS_DATA
            .get()
            .expect("Nepos data was not initialized, what gives?")
            .get(imdb_id)
        {
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
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("".to_string())?,
        },
    )
}

#[tokio::test]
async fn test_success() {
    initialize_nepos_data().unwrap();

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
