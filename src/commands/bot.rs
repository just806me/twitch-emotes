use clap::ArgMatches;
use hyper::rt::{self, Future, Stream};
use hyper::{self, service::service_fn, Body, Request, Response, Server};
use reqwest::{header::ContentType, Client, Url};
use serde_json;
use std::{env::var, thread, time::Duration};

use commands::shared::*;
use error::Result;
use models::emoticon::Emoticon;

lazy_static! {
    static ref TELEGRAM_TOKEN: String = var("TELEGRAM_TOKEN").unwrap_or_default();
    static ref IMAGE_SERVER_URL: String = var("IMAGE_SERVER_URL").unwrap_or_default();
}

macro_rules! telegram_api_url {
    ($($arg:tt)*) => {
        format!(
            "https://api.telegram.org/bot{}/{}",
            *TELEGRAM_TOKEN, ($($arg)*)
        )
    };
}

#[derive(Serialize, Deserialize, Debug)]
struct Update {
    #[serde(rename = "update_id")]
    id: i32,
    inline_query: Option<InlineQuery>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InlineQuery {
    id: String,
    query: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AnswerInlineQuery {
    method: String,
    inline_query_id: String,
    results: Vec<EmoticonResult>,
    cache_time: i32,
    is_personal: bool,
}

impl AnswerInlineQuery {
    pub fn new(inline_query_id: &str, results: Vec<EmoticonResult>) -> Self {
        Self {
            method: "answerInlineQuery".to_string(),
            inline_query_id: inline_query_id.to_string(),
            results,
            cache_time: 0 * 60 * 60,
            is_personal: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct EmoticonResult {
    #[serde(rename = "type")]
    type_: String,
    id: String,
    photo_url: String,
    thumb_url: String,
    caption: String,
}

impl EmoticonResult {
    pub fn new(url: &str, emoticon: &Emoticon) -> Self {
        Self {
            type_: "photo".to_owned(),
            id: emoticon.id.to_string(),
            photo_url: url.to_string(),
            thumb_url: url.to_string(),
            caption: emoticon.code.to_string(),
        }
    }
}

fn get_response(body: hyper::Chunk) -> Result<Response<Body>> {
    let update = serde_json::from_slice::<Update>(&body)?;

    info!("new update {}", update.id);

    let body = match update.inline_query {
        Some(inline_query) => {
            debug!("inline query {}", inline_query.query);

            let emoticons =
                Emoticon::load_by_code(&inline_query.query, &establish_connection()?, 12)?;

            let mut results = Vec::with_capacity(emoticons.len());

            for emoticon in emoticons {
                let result = EmoticonResult::new(
                    &((*IMAGE_SERVER_URL).clone() + &emoticon.id.to_string()),
                    &emoticon,
                );

                results.push(result);
            }

            let response = AnswerInlineQuery::new(&inline_query.id, results);

            serde_json::to_vec(&response)
                .map(Body::from)
                .map_err(|e| e.into())
        }
        None => {
            debug!("not inline query");

            Ok(Body::empty())
        }
    };

    info!("done {}", update.id);

    body.map(|b| {
        Response::builder()
            .header("Content-Type", "application/json")
            .status(200)
            .body(b)
            .unwrap()
    })
}

fn process_request(
    request: Request<Body>,
) -> impl Future<Item = Response<Body>, Error = hyper::Error> {
    request
        .into_body()
        .concat2()
        .map(|body| match get_response(body) {
            Ok(response) => response,
            Err(e) => {
                error!("{}", e);

                Response::builder().status(500).body(Body::empty()).unwrap()
            }
        })
}

pub fn start(matches: &ArgMatches) -> Result<()> {
    let addr = matches.value_of("address").unwrap();

    info!("starting bot at {}", addr);

    rt::run(
        Server::bind(&addr.parse()?)
            .serve(|| service_fn(process_request))
            .map_err(|e| error!("{}", e)),
    );

    Ok(())
}

fn process_proxy_update(update: Update, client: &Client, offset: i32, url: &str) -> Result<i32> {
    info!("new update {}", update.id);

    let bot_response = client
        .post(url)
        .body(serde_json::to_vec(&update)?)
        .send()?
        .json::<AnswerInlineQuery>();

    match bot_response {
        Ok(bot_response) => {
            let api_url = telegram_api_url!(bot_response.method);

            let api_response = client
                .post(&api_url)
                .header(ContentType::json())
                .body(serde_json::to_vec(&bot_response)?)
                .send()?
                .text()?;

            debug!("url {}", api_url);

            debug!(
                "bot response {}",
                serde_json::to_string_pretty(&bot_response)?
            );

            debug!("api response {}", api_response);
        }
        Err(e) => warn!("{}", e),
    }

    info!("done {}", update.id);

    Ok(offset.max(update.id + 1))
}

pub fn proxy(matches: &ArgMatches) -> Result<()> {
    #[derive(Deserialize, Debug)]
    struct ApiResponse {
        result: Option<Vec<Update>>,
    }

    let client = Client::new();

    let mut offset = 0;

    let bot_url = matches.value_of("bot").unwrap();

    let timeout = Duration::from_millis(matches.value_of("timeout").unwrap().parse()?);

    info!("starting proxy for {}", bot_url);

    loop {
        let api_url = Url::parse_with_params(
            &telegram_api_url!("getUpdates"),
            &[("offset", offset.to_string())],
        )?;

        let api_response = client.get(api_url).send()?.json::<ApiResponse>()?;

        match api_response.result {
            Some(updates) => {
                for update in updates {
                    offset = process_proxy_update(update, &client, offset, bot_url)?;
                }
            }
            None => return Err(format!("bad telegram response: {:?}", api_response).into()),
        }

        thread::sleep(timeout);
    }
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("start", Some(matches)) => start(matches),
        ("proxy", Some(matches)) => proxy(matches),
        _ => unimplemented!(),
    }
}
