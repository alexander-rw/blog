# Blog

This is my blog site written in rust, compiled to a single binary to run somewhere in the cloud. I currently host on DigitalOcean, with Cloudflare as the DNS Provider. 

## Local Development

You'll need `cargo` installed.

`cargo run` to run the project. `cargo build --release --bin blog` to just build it for release mode. The binary is a few mb (~3.5 at time of writing this initially, with a handful of md pages), so easy to upload and run with minimal memory footprint.

## Engineering decisions

All markdown files are installed within the binary. I wanted to do this for two reasons:
- speed (so i didn't have to load a file off the file system when requesting a page)
- ease of compilation (i wanted to fail a build if one of the md files was wrong, at build time - and if i'm validating at that point, then why _wouldn't_ I just include the files inside the binary?). This isn't the right choice if i'm hosting loads of pages or other assets. Seems fine for my scale though.

### Why rust?

I wanted to learn rust. I also maintain [https://github.com/alexander-rw/logs-of-war](https://github.com/alexander-rw/logs-of-war) for the same reason.

## License

This project is licensed under the MIT License.
