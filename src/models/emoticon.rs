use schema::emoticons;

#[derive(Queryable, Insertable, Deserialize, Debug)]
pub struct Emoticon {
    pub id: i64,
    pub code: String,
}

impl Emoticon {
    pub fn url(&self) -> String {
        format!("https://static-cdn.jtvnw.net/emoticons/v1/{}/3.0", self.id)
    }
}
