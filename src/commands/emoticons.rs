use clap::ArgMatches;
use crossbeam_utils::thread;
use diesel::{self, prelude::*};

use commands::shared::*;
use error::Result;
use models::emoticon::Emoticon;
use schema::emoticons;

fn collect_threads_results(handles: Vec<thread::ScopedJoinHandle<'_, Result<()>>>) -> Result<()> {
    fn convert(handle: thread::ScopedJoinHandle<'_, Result<()>>) -> Result<()> {
        match handle.join() {
            Ok(result) => result,
            Err(_) => Err("thread paniced".into()),
        }
    }

    handles.into_iter().map(convert).collect()
}

pub fn fetch() -> Result<()> {
    fn insert_chunk(chunk: &[Emoticon]) -> Result<()> {
        match diesel::insert_into(emoticons::table)
            .values(chunk)
            .execute(&establish_connection()?)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    const CHUNK_SIZE: usize = 65535 / 2; // 2 is number of fields in Emoticon struct

    let emoticons = Emoticon::load_from_twitch()?;

    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(emoticons.len() / CHUNK_SIZE + 1);

        for chunk in emoticons.chunks(CHUNK_SIZE) {
            handles.push(scope.spawn(move || insert_chunk(chunk)));
        }

        collect_threads_results(handles)
    })
}

pub fn delete() -> Result<()> {
    match diesel::delete(emoticons::table).execute(&establish_connection()?) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }

    // TODO: delete images
}

pub fn find(matches: &ArgMatches) -> Result<()> {
    let code = matches.value_of("code").unwrap();

    let limit = matches.value_of("limit").unwrap().parse::<i64>()?;

    let emoticons = Emoticon::load_by_code(code, &establish_connection()?, limit)?;

    println!("{:#?}", emoticons);

    Ok(())
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("fetch", _) => fetch(),
        ("delete", _) => delete(),
        ("find", Some(matches)) => find(matches),
        _ => unimplemented!(),
    }
}
