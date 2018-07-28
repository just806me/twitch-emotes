use clap;
use crossbeam_utils::thread;
use diesel::{self, prelude::*, PgConnection};
use reqwest;
use std::env;

use models::emoticon::Emoticon;
use schema::emoticons;

fn establish_connection() -> PgConnection {
    trace!("connecting to database");

    let database_url = env::var("DATABASE_URL").unwrap();

    trace!("database url is {}", database_url);

    PgConnection::establish(&database_url).unwrap()
}

fn fetch_emoticons() -> Vec<Emoticon> {
    #[derive(Deserialize)]
    struct Response {
        emoticons: Vec<Emoticon>,
    }

    trace!("fetching emoticons");

    let client_id: &str = &*env::var("TWITCH_CLIENT_ID").unwrap();

    trace!("twitch client id is {}", client_id);

    let url = format!(
        "https://api.twitch.tv/kraken/chat/emoticon_images?client_id={}",
        client_id
    );

    reqwest::get(&url)
        .and_then(|mut r| r.json::<Response>())
        .map(|r| r.emoticons)
        .map(|e| {
            trace!("fetched {} emoticons", e.len());
            e
        })
        .map_err(|e| error!("cound not fetch emoticons: {}", e))
        .unwrap()
}

pub fn fetch() {
    const CHUNK_SIZE: usize = 65535 / 2; // 2 is number of fields in Emoticon struct

    let emoticons = fetch_emoticons();

    thread::scope(|scope| {
        for chunk in emoticons.chunks(CHUNK_SIZE) {
            scope.spawn(move || {
                trace!("inserting chunk with len {}", chunk.len());

                diesel::insert_into(emoticons::table)
                    .values(chunk)
                    .execute(&establish_connection())
                    .unwrap();
            });
        }
    });

    trace!("done");

    // TODO: download images and convert them to jpeg
}

pub fn delete() {
    let conn = establish_connection();

    trace!("deleting emoticons");

    diesel::delete(emoticons::table).execute(&conn).unwrap();

    trace!("done");

    // TODO: delete images
}

pub fn run(matches: Option<&clap::ArgMatches>) {
    match matches.unwrap().subcommand() {
        ("fetch", _) => fetch(),
        ("delete", _) => delete(),
        _ => unimplemented!(),
    }
}
