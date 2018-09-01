use clap::ArgMatches;
use hyper::rt::{self, Future};
use hyper::{service::service_fn_ok, Body, Request, Response, Server};
use regex::Regex;

use commands::shared::*;
use error::Result;
use models::emoticon::Emoticon;

lazy_static! {
    static ref PATH: Regex = Regex::new("\\A/emoticons/([0-9]+)\\z").unwrap();
}

fn get_image_for_request(request: Request<Body>) -> Result<Vec<u8>> {
    let id = PATH.captures(request.uri().path())
        .and_then(|captures| captures.get(1))
        .and_then(|id_match| id_match.as_str().parse::<i64>().ok());

    match id {
        Some(id) => {
            info!("requested emoticon {}", id);

            Emoticon::load_by_id(id, &establish_connection()?)?.get_image()
        },
        None => Err(format!("bad request path {}", request.uri().path()).into()),
    }
}

pub fn start(matches: &ArgMatches) -> Result<()> {
    let addr = matches.value_of("address").unwrap();

    info!("starting server at {}", addr);

    rt::run(
        Server::bind(&addr.parse()?)
            .serve(|| {
                service_fn_ok(|request| match get_image_for_request(request) {
                    Ok(data) => Response::builder()
                        .status(200)
                        .header("Content-Type", "image/jpeg")
                        .body(Body::from(data))
                        .unwrap(),
                    Err(e) => {
                        error!("{}", e);

                        Response::builder().status(500).body(Body::empty()).unwrap()
                    }
                })
            })
            .map_err(|e| error!("{}", e)),
    );

    Ok(())
}

pub fn run(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("start", Some(matches)) => start(matches),
        _ => unimplemented!(),
    }
}
