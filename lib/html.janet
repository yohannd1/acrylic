(import ./html-tree)

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
  (def inline-js
    ```
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
    ```)

  (def css-uri (string katex-path "katex.min.css"))
  (def js-uri (string katex-path "katex.min.js"))

  ~[(link {:rel "stylesheet" :href ,css-uri})
    (script {:src ,js-uri :defer true})
    (script {raw true} ,inline-js)])

(defn ast->html
  ```Analyze `ast` and return its HTML representation.

  Options:
    :css - the CSS stylesheet to be used, as CSS code. Vulnerable to HTML injection.
    :katex-path - the katex path prefix to use in the header.
  ```
  [parse-result opts]

  (def {:ast ast :header header} parse-result)

  (def css (-> opts (in :css default-css)))
  (def katex-path (-> opts (in :katex-path "")))

  (def head
    ~(head
       (meta {:name "viewport" :content "width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no"})
       (meta {:http-equiv "X-UA-Compatible" :content "IE=edge,chrome=1"})
       (meta {:name "HandheldFriendly" :content "true"})
       (meta {:charset "UTF-8"})
       ,;(make-katex-header katex-path)
       (style ,css)
       ))

  (def body @[])

  (if-let [title (in header :title)]
    (array/push body ~(h1 ,title)))

  (defn process-component
    ``Process the component of a ``
    [c]

    (match c
      [:latex-math-inline text] ~(span {:class "katex-inline"} ,text)
      [:bold text] ~(b ,text)
      [:italic text] ~(i ,text)
      [:bold-italic text] ~(b (i ,text))
      [:code text] ~(code ,text)
      [:comment _] ""
      other (do
              (assert (string? other) "Line component should be a string")
              (string other))
      ))

  (each node ast
    (defn process-content
      ``Process the content of the current node and output an array of HTML elements to be put inside the paragraph.``
      []

      (def content (-> node (in :content)))
      (map process-component content))

    (def indent (-> node (in :indent) (* 1.25)))

    (def line-class (if (-> node (in :spacing) (= 'big))
                      "line-spaced" "line-normal"))

    (def attrs @{:class line-class
                 :style (string/format "margin-left: %.2fem;" indent)})

    (match (in node :type)
      'generic
      (do
        (def line ~(p ,attrs ,;(process-content)))
        (array/push body line))

      'comment
      nil

      'latex
      (do
        (set (attrs :class) (string (in attrs :class) " katex-display"))
        (def line ~(p ,attrs ,;(process-content)))
        (array/push body line))

      ['task type-]
      (do
        (var line ~(p ,attrs))

        (def checkbox-attrs @{:type "checkbox" :disabled true})
        (def checkbox ~(input ,checkbox-attrs))

        (case (string/ascii-lower type-)
          # pending tasks
          " " (do
                (def line ~(p ,attrs ,checkbox ,;(process-content)))
                (array/push body line))

          # finished tasks
          "x" (do
                (set (checkbox-attrs :checked) true)
                (def line ~(p ,attrs ,checkbox ,;(process-content)))
                (array/push body line))

          # cancelled tasks
          "-" (do
                (set (checkbox-attrs :checked) true)
                (def line ~(p ,attrs ,checkbox (s ,;(process-content))))
                (array/push body line))
          ))

      other
      (-> "Unknown form: %j" (string/format other) (error))
      )

    )

  (html-tree/generate-html ~(html ,head (body ,;body))))
