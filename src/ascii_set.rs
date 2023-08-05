pub enum AsciiCharset {
    LowDetails(String),
    BasicDetails(String),
}

static LOW: AsciiCharset = AsciiCharset::LowDetails("@%#*+=-:. ".to_owned());
static BASIC: AsciiCharset = AsciiCharset::BasicDetails(
    "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ".to_owned(),
);
