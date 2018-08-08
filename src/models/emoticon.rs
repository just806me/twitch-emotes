use diesel::prelude::*;

use error::Result;
use schema::emoticons;

#[derive(Queryable, Insertable, Deserialize, Debug)]
pub struct Emoticon {
    pub id: i64,
    pub code: String,
}

impl Emoticon {
    fn image_url(&self) -> String {
        format!("https://static-cdn.jtvnw.net/emoticons/v1/{}/3.0", self.id)
    }

    fn image_path(&self) -> String {
        format!("images/{}.jpg", self.id)
    }

    fn fetch_image(&self) -> Result<Vec<u8>> {
        use reqwest::{get, header::ContentLength};

        let mut response = get(&self.image_url())?;

        let len = response
            .headers()
            .get::<ContentLength>()
            .map(|ct_len| **ct_len)
            .unwrap_or(0) as usize;

        let mut image = Vec::with_capacity(len);

        response.copy_to(&mut image)?;

        Ok(image)
    }

    fn convert_image(image: Vec<u8>) -> Result<Vec<u8>> {
        use std::io::prelude::*;
        use std::process::{Command, Stdio};

        let mut process = Command::new("convert")
            .args(&["-", "jpg:-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        process.stdin.as_mut().unwrap().write_all(&image[..])?;

        process.wait()?;

        let mut image = Vec::new();

        process.stdout.as_mut().unwrap().read_to_end(&mut image)?;

        Ok(image)
    }

    pub fn get_image(&self) -> Result<Vec<u8>> {
        Self::convert_image(self.fetch_image()?)
    }

    pub fn load_from_twitch() -> Result<Vec<Self>> {
        use reqwest::{get, Url};
        use std::env::var;

        #[derive(Deserialize)]
        struct Response {
            emoticons: Vec<Emoticon>,
        }

        let url = Url::parse_with_params(
            "https://api.twitch.tv/kraken/chat/emoticon_images",
            &[("client_id", var("TWITCH_CLIENT_ID")?)],
        )?;

        Ok(get(url)?.json::<Response>()?.emoticons)
    }

    pub fn load_by_id(emoticon_id: i64, connection: &PgConnection) -> Result<Self> {
        use schema::emoticons::dsl::*;

        emoticons
            .filter(id.eq(emoticon_id))
            .first::<Self>(connection)
            .map_err(|e| e.into())
    }
}
