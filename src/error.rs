use reqwest::StatusCode;

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Req(::reqwest::Error);
    }

    errors {
        NonSuccessStatus(code: StatusCode) {
            description("The status code of a received response was not success.")
            display("The status code of a received response was {} and not success.",
                     code)
        }

        ExtractionError(html: String) {
            description("Something went wrong while attempting to extract \
                         information from HTML.")
            display("Something went wrong while attempting to extract information \
                     from the following HTML: {}", html)
        }
    }
}
