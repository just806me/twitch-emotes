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

    pub fn get_image(&self) -> Result<Vec<u8>> {
        use reqwest::{get, header::ContentLength};

        let mut response = get(&self.image_url())?;

        let len = response
            .headers()
            .get::<ContentLength>()
            .map(|ct_len| **ct_len)
            .unwrap_or(0) as usize;

        let mut buffer = Vec::with_capacity(len);

        response.copy_to(&mut buffer)?;

        Ok(buffer)

        // TODO: convert to jpeg

        // TODO: caching

        // use std::{io::Write, process};

        // fn process_image(emoticon: &Emoticon) -> Result<()> {
        //     let image = fetch_image(emoticon)?;

        //     let mut process = process::Command::new("ffmpeg")
        //         .args(&["-i", "-", &emoticon.path()])
        //         .stdin(process::Stdio::piped())
        //         .spawn()?;

        //     process.stdin.as_mut().unwrap().write_all(&image)?;

        //     process.wait()?;

        //     Ok(())
        // }
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
}
