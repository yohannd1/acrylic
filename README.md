# acrylic

A little note/task format I made for myself, mostly based off Markdown,
Org-mode, Scribble and LaTeX. Its main difference is the focus on
indentation and line spacing to separate items.

This repository hosts most of the tools I use for my "acrylic" text
format, with some exceptions.

## related software

- [vim plugin](https://github.com/yohannd1/acrylic.vim) for this format;

- [related scripts in my
- dotfiles](https://github.com/yohannd1/dotfiles/blob/master/scripts)
- (they start with `acr-`);

## plans

See [the todo file](TODO.acr) and the TODOs & FIXMEs in the source code.

## building

The parser & HTML renderer was [originally made in
Janet](https://github.com/yohannd1/acrylic/tree/c4450ca1d20aa3dedb6253290a26030d62fab593),
but I ended up switching to Rust because I wanted the static typing
and... speed, perhaps? My Janet parser was really slow in bigger
documents.

```sh
cargo build
```

TODO: talk about the resulting binary, name it correctly (probably
`acr-parse` or something like that?), mention testing (which I didn't
start yet...)

## why "acrylic"?

I was thinking of a name and the word came to mind. It works as an
analogy where this format is acrylic and others - such as Markdown, Org
and LaTeX - are glass.

Glass is usually more robust, but it can also be heavier and more
expensive. In contrast, Acrylic is less resistant to scratches, UV rays
and germs, but it's light and cheap.

It's a really bad analogy. But the name's nice. Sorry.

> _[...] while acrylic has a small upfront cost, in the long run, it
> could cost just as much or more than glass._
>
> https://www.fgdglass.com/blog/acrylic-vs-glass/
