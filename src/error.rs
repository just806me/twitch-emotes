error_chain! {
    foreign_links {
        StdVarError(::std::env::VarError);

        StdParseIntError(::std::num::ParseIntError);

        StdIoError(::std::io::Error);

        StdAddrParseError(::std::net::AddrParseError);

        ReqwestError(::reqwest::Error);

        ReqwestUrlError(::reqwest::UrlError);

        DieselError(::diesel::result::Error);

        DieselConnectionError(::diesel::ConnectionError);

        HyperError(::hyper::Error);

        SerdeJsonError(::serde_json::Error);
    }
}
