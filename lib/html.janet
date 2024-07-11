(def- default-css
  ```
  body {}
  p.line-normal {
    margin-top: 0em;
    margin-bottom: 0.1em;
  }
  p.line-spaced {
    margin-top: 0em;
    margin-bottom: 1.5em;
  }
  .katex-display.fleqn>.katex {
    padding-left: 0em;
  }
  ```)

(def- katex-header
  ```
  <link rel="stylesheet" href="katex.min.css"/>
  <script src="katex.min.js" defer></script>
  <script>
    document.addEventListener("DOMContentLoaded", function() {
      const macros = {};
      const opts = {
        throwOnError: false,
        macros: macros,
      };

      for (let e of document.querySelectorAll(".katex-inline")) {
        const text = e.innerText;
        e.innerText = "";
        katex.render(text, e, {displayMode: false, ...opts});
      }

      for (let e of document.querySelectorAll(".katex-display")) {
        const text = e.innerText;
        e.innerText = "";
        katex.render(text, e, {displayMode: true, fleqn: true, ...opts});
      }
    });
  </script>
  ```
  )

(defn to-html
  ```Analyze `ast` and return its HTML representation.

  Options:
    :css - the CSS stylesheet to be used, as CSS code. Vulnerable to HTML injection.
  ```
  [ast opts]

  (def css (-> opts (in :css) (or default-css)))

  (def buf @"")
  (defn ps [& args]
    (loop [s :in args]
      (buffer/push-string buf s)))

  (defn process-unit [node]
    (match node
      [:latex-math-inline text]
      (ps `<span class="katex-inline">` text "</span>")

      other
      (ps other)
      ))

  (ps `<!DOCTYPE html>`)
  (ps `<html>`)
  (ps `<head>`)
  (ps `<meta charset="UTF-8"/>
       <meta name="viewport" content="width=device-width,initial-scale=1"/>`)
  (ps katex-header)
  (ps `<style>`)
  (ps css)
  (ps `</style>`)
  (ps `</head>`)
  (ps `<body>`)

  (each node ast
    (match node
      [:line-spaced & contents]
      (do
        (ps `<p class="line-spaced">`)
        (loop [c :in contents] (process-unit c))
        (ps `</p>`))

      [:line-normal & contents]
      (do
        (ps `<p class="line-normal">`)
        (loop [c :in contents] (process-unit c))
        (ps `</p>`))

      [:line-latex & contents]
      (do
        (ps `<p class="line-normal katex-display">`)
        (loop [c :in contents] (process-unit c))
        (ps `</p>`))

      other
      (error (string/format "Unknown form: %j" other))
      ))

  (ps `</body>`)
  (ps `</html>`)

  buf)
