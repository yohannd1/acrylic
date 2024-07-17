(def- default-css
  ```
  body {
    font-family: sans-serif;
    font-size: 1.08em;
  }
  p.line-normal {
    margin-top: 0em;
    margin-bottom: 0.1em;
  }
  p.line-spaced {
    margin-top: 0em;
    margin-bottom: 1.5em;
  }
  .katex-display {
    margin: 0em 0em;
  }
  .katex-display.fleqn>.katex {
    padding-left: 0em;
  }
  ```)

(defn- make-katex-header [katex-path]
  (string/format
    ```
    <link rel="stylesheet" href="%skatex.min.css"/>
    <script src="%skatex.min.js" defer></script>
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
    katex-path
    katex-path
  ))

(defn to-html
  ```Analyze `ast` and return its HTML representation.

  Options:
    :css - the CSS stylesheet to be used, as CSS code. Vulnerable to HTML injection.
    :katex-path - the katex path prefix to use in the header.
  ```
  [parse-result opts]

  (def {:ast ast :header header} parse-result)

  (def css (-> opts (in :css default-css)))
  (def katex-path (-> opts (in :katex-path ""))) # FIXME: SHOULDN'T BE VULNERABLE TO HTML INJECTION!

  (def buf @"")
  (defn ps [& args]
    (loop [s :in args]
      (buffer/push-string buf (string s))))

  (defn process-unit [node]
    (match node
      [:latex-math-inline text]
      (ps `<span class="katex-inline">` text "</span>")

      [:bold text]
      (ps `<b>` text `</b>`)

      [:italic text]
      (ps `<i>` text `</i>`)

      [:bold-italic text]
      (ps `<b><i>` text `</i></b>`)

      [:code text]
      (ps `<code>` text `</code>`)

      other
      (ps other)
      ))

  (ps `<!DOCTYPE html>`)
  (ps `<html>`)
  (ps `<head>`)
  (ps `<meta charset="UTF-8"/>
       <meta name="viewport" content="width=device-width,initial-scale=1"/>`)
  (ps (make-katex-header katex-path))
  (ps `<style>`)
  (ps css)
  (ps `</style>`)
  (ps `</head>`)
  (ps `<body>`)

  (if-let [title (in header :title)]
    (ps `<h1>` title `</h1>`))

  (each node ast
    (def indent (-> node (in :indent) (* 1.25)))

    (def html-classes @[])
    (array/push html-classes (if (-> node (in :spacing) (= 'big))
                               "line-spaced" "line-normal"))

    (def before-content @[])
    (def after-content @[])

    (defn write-node "do the stuff hehe" []
      (ps `<p class="`)
      (loop [class :in html-classes]
        (ps class ` `))
      (ps `" style="margin-left: ` indent `em;">`)
      (loop [c :in before-content]
        (process-unit c))
      (loop [c :in (in node :content)]
        (process-unit c))
      (loop [c :in after-content]
        (process-unit c))
      (ps `</p>`)
      )

    (match (in node :type)
      'generic
      (write-node)

      'comment
      nil

      'latex
      (do
        (array/push html-classes "katex-display")
        (write-node))

      ['task type-]
      (do
        (match (string/ascii-lower type-)
          " " (array/push before-content `<input type="checkbox" disabled/>`)
          "x" (array/push before-content `<input type="checkbox" checked disabled/>`)
          "-" (do
                (array/push before-content `<input type="checkbox" checked disabled/><s>`)
                (array/push after-content `</s>`)
                ))
        (write-node))

      other
      (error (string/format "Unknown form: %j" other))
      )
    )

  (ps `</body>`)
  (ps `</html>`)

  buf)
