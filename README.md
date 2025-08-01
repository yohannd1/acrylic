# acrylic

This repository hosts most of the tools I use for my "acrylic" text
format.

The parser & HTML renderer was [originally made in Janet](https://github.com/yohannd1/acrylic/tree/c4450ca1d20aa3dedb6253290a26030d62fab593), but I ended up switching to Rust because I wanted the static typing and... speed, perhaps? My Janet parser was really slow in bigger documents.

## related software

- [vim plugin](https://github.com/yohannd1/acrylic.vim) for this format;

- [related scripts in my dotfiles](https://github.com/yohannd1/dotfiles/blob/master/scripts) (they start with `acr-`);

## plans

See [the todo file](TODO.acr) and the TODOs & FIXMEs in the source code.

## building

```sh
cargo build
```

TODO: talk about the resulting binary, name it correctly (probably
`acr-parse` or something like that?), mention testing (which I didn't
start yet...)
