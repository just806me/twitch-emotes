use clap;
use crossbeam_utils::thread;
use diesel::{self, prelude::*, PgConnection};
use reqwest;
use std::env;

use commands::error::Result;
use models::emoticon::Emoticon;
use schema::emoticons;

fn establish_connection() -> Result<PgConnection> {
    let database_url = env::var("DATABASE_URL")?;

    PgConnection::establish(&database_url).map_err(|e| e.into())
}

fn fetch_emoticons() -> Result<Vec<Emoticon>> {
    #[derive(Deserialize)]
    struct Response {
        emoticons: Vec<Emoticon>,
    }

    let client_id = env::var("TWITCH_CLIENT_ID")?;

    let url = reqwest::Url::parse_with_params(
        "https://api.twitch.tv/kraken/chat/emoticon_images",
        &[("client_id", client_id)],
    )?;

    reqwest::get(url)
        .and_then(|mut r| r.json::<Response>())
        .map(|r| r.emoticons)
        .map_err(|e| e.into())
}

fn insert_emoticons_chunk(chunk: &[Emoticon]) -> Result<()> {
    let conn = establish_connection()?;

    diesel::insert_into(emoticons::table)
        .values(chunk)
        .execute(&conn)?;

    Ok(())
}

pub fn fetch() -> Result<()> {
    const CHUNK_SIZE: usize = 65535 / 2; // 2 is number of fields in Emoticon struct

    let emoticons = fetch_emoticons()?;

    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(emoticons.len() / CHUNK_SIZE + 1);

        for chunk in emoticons.chunks(CHUNK_SIZE) {
            handles.push(scope.spawn(move || insert_emoticons_chunk(chunk)));
        }

        handles
            .into_iter()
            .map(|handle| match handle.join() {
                Ok(result) => result,
                Err(_) => Err("thread error".into()),
            })
            .collect()
    })

    // TODO: download images and convert them to jpeg
}

pub fn delete() -> Result<()> {
    let conn = establish_connection()?;

    diesel::delete(emoticons::table).execute(&conn)?;

    Ok(())

    // TODO: delete images
}

pub fn run(matches: &clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("fetch", _) => fetch(),
        ("delete", _) => delete(),
        _ => unimplemented!(),
    }
}
