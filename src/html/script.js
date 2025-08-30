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
