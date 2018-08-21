use clap::ArgMatches;
use hyper::rt::{self, Future};
use hyper::{service::service_fn_ok, Body, Request, Response, Server};

use commands::shared::*;
use error::Result;
use models::emoticon::Emoticon;

fn get_image_for_request(request: Request<Body>) -> Result<Vec<u8>> {
    Emoticon::load_by_id(
        request.uri().path()[1..].parse::<i64>()?,
        &establish_connection()?,
    )?.get_image()
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
