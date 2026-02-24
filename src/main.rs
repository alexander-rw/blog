fn main() {
    let mdx = std::fs::read_to_string("pages/index.mdx").expect("failed to read pages/index.mdx");
    let html = markdown::to_html(&mdx);
    println!("{html}");
}
