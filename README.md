# Blog

This is my blog site, written in Rust. Every page is rendered to HTML at build time and served as static files by a Cloudflare Worker at [alexander-rw.com](https://alexander-rw.com). Cloudflare is also the DNS provider.

## Local Development

You'll need `cargo` installed. `rust-toolchain.toml` pins the Rust version, and rustup installs it automatically.

- `cargo run` starts a local axum server on [localhost:3084](http://localhost:3084). Use it to iterate quickly on pages and styles.
- `cargo run -- export` writes the static site to `dist/`. This is what gets deployed. It uses the same rendering as the server, so the output matches.
- `npx wrangler dev` serves `dist/` exactly as Cloudflare does in production, including trailing-slash redirects, the 404 page and `_redirects`. Run the export first.

## Deployment

Cloudflare Workers Builds is connected to this repo and deploys every push to `main`:

1. The build command (set in the Cloudflare dashboard) installs Rust with rustup and runs `cargo run -- export`.
2. The deploy command, `npx wrangler deploy`, uploads `dist/` as a static-assets-only Worker using `wrangler.jsonc`.

Notes:

- Settings in the Workers Builds dashboard override the repo. If a build runs an unexpected command, check the dashboard first.
- The Worker's `name` in `wrangler.jsonc` must match the Worker name in the dashboard (`blog`).
- `wrangler.jsonc` attaches the `alexander-rw.com` custom domain, and Cloudflare manages its DNS record. `www` redirects to the apex through a rule in the Cloudflare dashboard, which isn't part of this repo.
- Redirects are defined once in `src/redirects.rs`, and the export turns them into `dist/_redirects`.
- Pages marked `draft: true` are left out of the post listing and search, but are still served at their URL.
- GitHub Actions (`.github/workflows/ci.yml`) runs fmt, clippy, tests and the export on every push and pull request. It doesn't deploy.

## Engineering decisions

All markdown files are embedded in the binary and rendered once, then exported as static HTML. I wanted to do this for two reasons:
- speed (nothing is rendered per request, and Cloudflare serves the files straight from its edge)
- ease of compilation (i wanted to fail a build if one of the md files was wrong, at build time - and if i'm validating at that point, then why _wouldn't_ I just include the files inside the binary?). This isn't the right choice if i'm hosting loads of pages or other assets. Seems fine for my scale though.

### Why rust?

I wanted to learn rust. I also maintain [https://github.com/alexander-rw/logs-of-war](https://github.com/alexander-rw/logs-of-war) for the same reason.

## License

This project is licensed under the MIT License.
