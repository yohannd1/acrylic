pub const DEFAULT_STYLE: &'static str = r#"
:root {
    --col-fg-alt: #555555;

    --col-bg: #FFFFFF;
    --col-bg-alt: #EEEEEE;

    --col-emphasis: #DF5273;
    --col-href: #2277DD;
}

html {
    background-color: var(--col-bg);
}

body {
    font-family: sans-serif;
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
}

span.acr-tag {
    color: var(--col-emphasis);
    font-size: 1.0em;
}

div.acr-spacing {
    margin-top: 0em;
    margin-bottom: 0.7em;
}

summary:hover {
    background-color: var(--col-bg-alt);
}

code, pre {
    white-space: pre-wrap;
    overflow-wrap: break-word;
    word-wrap: break-word;

    font-family: monospace;
    background: #f4f4f4;
    border: 1px solid #DDD;
    color: var(--col-fg-alt);
    max-width: 100%;
}
code {
    background-color: rgba(27, 31, 35, 0.05);
    border-radius: 2px;
    font-size: 85%;
    margin: 0;
    padding: 0.2em 0.4em;
    padding-top: 0.2em;
    padding-bottom: 0.1em;
}
pre {
    padding: 1em 1.5em;
    display: block;
    page-break-inside: avoid;
    line-height: 1.45;

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
"#;

pub const KATEX_INIT_JS: &'static str = r#"
document.addEventListener("DOMContentLoaded", function() {
    const macros = {};
    const opts = {
        throwOnError: false,
        macros: macros,
        globalGroup: true,
    };

    for (let e of document.querySelectorAll(".katex-inline")) {
        const text = e.textContent;
        e.textContent = "";
        katex.render(text, e, {displayMode: false, ...opts});
    }

    for (let e of document.querySelectorAll(".katex-display")) {
        const text = e.textContent;
        e.textContent = "";
        katex.render(text, e, {displayMode: true, fleqn: true, ...opts});
    }
});
"#;
