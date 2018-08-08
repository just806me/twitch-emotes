use diesel::{prelude::*, PgConnection};
use error::Result;
use std::{convert::From, env::var};

pub fn establish_connection() -> Result<PgConnection> {
    PgConnection::establish(&var("DATABASE_URL")?).map_err(From::from)
}
