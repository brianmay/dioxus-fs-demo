use static_file_util::process_file;

fn main() {
    process_file("assets/header.svg", "header_svg_HASH");
    process_file("assets/main.css", "main_css_HASH");
    process_file("assets/favicon.ico", "favicon_HASH");
}
