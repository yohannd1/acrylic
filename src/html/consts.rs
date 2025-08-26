pub const DEFAULT_STYLE: &'static str = r#"
:root {
    --col-fg-alt: #555555;

    --col-bg: #FFFFFF;
    --col-bg-alt: #EEEEEE;

    --col-emphasis: #DF5273;

    --col-href: #2277DD;
    --col-href-hover: #66CCEE;

    /* https://systemfontstack.com/ */
    --font-sans-serif: -apple-system, BlinkMacSystemFont, avenir next, avenir, segoe ui, helvetica neue, Adwaita Sans, Cantarell, Ubuntu, roboto, noto, helvetica, arial, sans-serif;
    --font-serif: Iowan Old Style, Apple Garamond, Baskerville, Times New Roman, Droid Serif, Times, Source Serif Pro, serif, Apple Color Emoji, Segoe UI Emoji, Segoe UI Symbol;
    --font-monospace: Menlo, Consolas, Monaco, Adwaita Mono, Liberation Mono, Lucida Console, monospace;
}

html {
    background-color: var(--col-bg);
}

body {
    font-family: var(--font-sans-serif);
    font-size: 1.08em;
}

p {
    margin-top: 0em;
    margin-bottom: 0.1em;
}

b {
    color: var(--col-emphasis);
}

a {
    color: var(--col-href);
    font-weight: bold;
    text-decoration-line: underline;
}

a:hover {
    color: var(--col-href-hover);
}

/* TODO: signify this is clickable (when I manage to link this stuff), or just make it an <a> tag too */
.acr-href {
    color: var(--col-emphasis);
    text-decoration-line: underline;
}

span.acr-tag {
    color: var(--col-emphasis);
    font-size: 1.0em;
}

div.acr-spacing {
    margin-top: 0em;
    margin-bottom: 0.85em;
}

summary:hover {
    background-color: var(--col-bg-alt);
}

.acr-inline-code, pre {
    white-space: pre-wrap;
    overflow-wrap: break-word;
    word-wrap: break-word;

    font-family: var(--font-monospace);
    background: #f4f4f4;
    border: 1px solid #DDD;
    color: var(--col-fg-alt);
    max-width: 100%;
}

.acr-inline-code {
    background-color: rgba(27, 31, 35, 0.05);
    border-radius: 2px;
    font-size: 85%;
    margin: 0;
    padding: 0.2em 0.4em;
    padding-top: 0.2em;
    padding-bottom: 0.1em;
}

pre {
    padding: 0.5em;
    /* display: block; */
    page-break-inside: avoid;

    background-color: var(--col-bg-alt);
    border-radius: 3px;
}

/* katex display */
.katex-display {
    margin: 0.5em 0em;
}
.katex-display.fleqn>.katex {
    padding-left: 0em;
}

@media (prefers-color-scheme: dark) {
    /* TODO: this is not the right way, but it works for now */
    svg g {
        fill: #FFF;
    }
}
"#;

pub const KATEX_INIT_JS: &'static str = r#"
document.addEventListener("DOMContentLoaded", function() {
    const macros = {};
    const opts = {
        throwOnError: false,
        macros: macros,
        globalGroup: true,
    };

    for (let e of document.querySelectorAll(".katex-inline, .katex-display")) {
        const text = e.textContent;
        const opts_selected = e.classList.contains("katex-display")
            ? {displayMode: true, fleqn: true, ...opts}
            : opts;

        katex.render(text, e, opts_selected);
    }
});
"#;
