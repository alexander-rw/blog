pub fn default_mdx_compile_options() -> markdown::Options {
    markdown::Options {
        compile: markdown::CompileOptions::gfm(),
        parse: markdown::ParseOptions {
            constructs: markdown::Constructs {
                frontmatter: true,
                code_indented: true,
                ..markdown::Constructs::mdx()
            },
            ..markdown::ParseOptions::mdx()
        },
    }
}
