# typomania-crates

An example of using [typomania][typomania] with a crates.io database dump.

## Prerequisites

### PostgreSQL

You'll need a [crates.io database dump][dump] loaded into a running PostgreSQL
instance. You'll also need to set the `DATABASE_URL` environment variable so
that `typomania-crates` can connect to Postgres: the easiest way to do that will
be to copy `.envrc.sample` to `.envrc` or `.env` (depending on whether you're
using `direnv` or `dotenv`, respectively), and change it to point to the right
Postgres.

### Configuration

You can edit `typomania.toml` to tinker with the options used when running the
typosquatting checks, although the defaults should be reasonable (and basically
match Dan Gardner's [typogard-crates][typogard-crates]).

### spaCy

By default, this uses [spaCy][spacy] to perform additional checks on the
description of each possibly typosquatted crate, which requires a Python
environment with spaCy enabled.

The easiest way to do this is with [Poetry][poetry], which this repo includes
configuration for:

```bash
poetry install
```

## Running

### With spaCy

```bash
poetry run cargo run
```

### Without spaCy

```bash
cargo run
```

## [Code of Conduct][code-of-conduct]

The [Rust Foundation][rust-foundation] has adopted a Code of Conduct that we
expect project participants to adhere to. Please read [the full
text][code-of-conduct] so that you can understand what actions will and will not
be tolerated.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Licenses

Rust is primarily distributed under the terms of both the MIT license and the
Apache License (Version 2.0), with documentation portions covered by the
Creative Commons Attribution 4.0 International license..

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT), 
[LICENSE-documentation](LICENSE-documentation), and 
[COPYRIGHT](COPYRIGHT) for details.

You can also read more under the Foundation's [intellectual property
policy][ip-policy].

## Other Policies

You can read about other Rust Foundation policies in the footer of the Foundation
[website][foundation-website].

[code-of-conduct]: https://foundation.rust-lang.org/policies/code-of-conduct/
[dump]: https://crates.io/data-access#database-dumps
[foundation-website]: https://foundation.rust-lang.org
[ip-policy]: https://foundation.rust-lang.org/policies/intellectual-property-policy/
[media-guide and trademark]: https://foundation.rust-lang.org/policies/logo-policy-and-media-guide/
[poetry]: https://python-poetry.org/
[rust-foundation]: https://foundation.rust-lang.org/
[spacy]: https://spacy.io/
[typogard-crates]: https://github.com/dangardner/typogard
[typomania]: https://crates.io/crates/typomania
