error_chain! {
    foreign_links {
        Var(::std::env::VarError);

        Reqwest(::reqwest::Error);

        ReqwestUrl(::reqwest::UrlError);

        Diesel(::diesel::result::Error);

        DieselConnection(::diesel::ConnectionError);
    }
}
