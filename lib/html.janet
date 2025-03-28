(import ./html-tree)

(def- default-css
  ```
  :root {
    --col-fg-alt: #555;
    --col-bg-alt: #FAFAFC;
  }

  body {
    font-family: sans-serif;
    font-size: 1.08em;
  }

  p {
    margin-top: 0em;
    margin-bottom: 0.1em;
  }

  div.acr-spacing {
    margin-top: 0em;
    margin-bottom: 1.5em;
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
  ```)

(defn- make-katex-header [katex-path]
  (def inline-js
    ```
    document.addEventListener("DOMContentLoaded", function() {
      const macros = {};
      const opts = {
        throwOnError: false,
        macros: macros,
        globalGroup: true,
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
  (def title (in header :title))

  (def head
    ~(head
       (meta {:name "viewport" :content "width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no"})
       (meta {:http-equiv "X-UA-Compatible" :content "IE=edge,chrome=1"})
       (meta {:name "HandheldFriendly" :content "true"})
       (meta {:charset "UTF-8"})
       ,;(if title [~(title ,title)] [])
       ,;(make-katex-header katex-path)
       (style {raw true} ,css)
       ))

  (def body @[])

  # fold stack entry format: {:indent <int> :elements <array>}
  (def fold-stack @[])

  (var pop-fold-stack nil)

  (defn add-element [elem fold-opts]
    (def {:indent indent :is-fold is-fold} (or fold-opts {}))

    (defn add-or-fold []
      (if is-fold
        # it's a fold! begin a fold with the desired element
        (array/push fold-stack {:indent indent :elements @[elem]})

        # it's not a fold - just add it to wherever it should be
        (if (empty? fold-stack)
          (-> body (array/push elem))
          (-> fold-stack (last) (in :elements) (array/push elem)))
        )
      )

    # this part is responsible for deciding if we need to clean up
    # folds or if we can already add the element, hence why the add-or-fold function is called in a few situations
    (if (empty? fold-stack)
      # normal flow
      (add-or-fold)

      (let [{:indent ts-indent :elements ts-elems} (last fold-stack)]
        (if (<= indent ts-indent)
          # whoops! the topmost fold entry is finished.
          # let's pop and try again.
          (do
            (pop-fold-stack)
            (add-element elem fold-opts))

          # normal flow
          (add-or-fold))
          )
        )
      )

  (defn pop-fold-stack-f
    ``Pop the topmost entry from the fold-stack, adding its fold structure as an element.``
    []

    (when (empty? fold-stack)
      (error "The fold stack is empty"))

    (def {:indent indent :elements ts-elems} (array/pop fold-stack))

    (def [summ-tag & summ-rest] (first ts-elems))
    (assert (= summ-tag 'p) "Soon-to-be-summary element should've been a paragraph")

    (add-element
      ~(details
         (summary ,;summ-rest)
         ,;(array/slice ts-elems 1))
      {:indent indent :is-fold false}
      )
    )
  (set pop-fold-stack pop-fold-stack-f)

  (when title
    (add-element ~(h1 ,title) nil))

  (defn process-component
    ``Process the component of a [??? TODO(finish sentence)]``
    [c]

    (match c
      [:latex-math-inline text] ~(span {:class "katex-inline"} ,text)
      [:bold text] ~(b ,text)
      [:italic text] ~(i ,text)
      [:bold-italic text] ~(b (i ,text))
      [:code text] ~(code ,text)
      [:url text] ~(a {:href ,text} ,text)
      [:comment _] ""

      [:tag tag]
      (do
        # TODO: process the tag if it begins with -
        ~(small "%" ,tag)
        )

      other
      (if (string? other)
        other
        (-> "Unknown line component: %j" (string/format other) (error)))

      ))

  (each node ast
    (defn process-content
      ``Process the content of the current node and output an array of HTML elements to be put inside the paragraph.``
      []

      (def content (-> node (in :content)))
      (map process-component content))

    # FIXME: this feels kinda hacky. the 'spacing type doesn't have indent, and doesn't make use of line-class and the like.

    (def indent (-> node (in :indent 0) (* 1.25)))

    (def is-fold
      (do
        (def indent (in node :indent 0))
        (def tags (in node :tags []))
        (def has-fold-tag (truthy? (find |(= "-fold" $) tags)))
        has-fold-tag
        ))

    (def attrs @{:style (string/format "margin-left: %.2fem;" indent)})

    (def fold-opts {:indent indent :is-fold is-fold})

    (match (in node :type)
      'spacing
      (add-element ~(div {:class "acr-spacing"}) nil)

      'generic
      (do
        (def line ~(p ,attrs ,;(process-content)))
        (add-element line fold-opts))

      'comment
      nil # do nothing

      'latex
      (do
        (set (attrs :class) (string (in attrs :class "") " katex-display"))
        (def line ~(p ,attrs ,;(process-content)))
        (add-element line fold-opts))

      ['task type-]
      (do
        (var line ~(p ,attrs))

        (def checkbox-attrs @{:type "checkbox" :disabled true})
        (def checkbox ~(input ,checkbox-attrs))

        (case (string/ascii-lower type-)
          # pending tasks
          " " (do
                (def line ~(p ,attrs ,checkbox ,;(process-content)))
                (add-element line fold-opts))

          # finished tasks
          "x" (do
                (set (checkbox-attrs :checked) true)
                (def line ~(p ,attrs ,checkbox ,;(process-content)))
                (add-element line fold-opts))

          # cancelled tasks
          "-" (do
                (set (checkbox-attrs :checked) true)
                (def line ~(p ,attrs ,checkbox (s ,;(process-content))))
                (add-element line fold-opts))
          ))

      other
      (-> "Unknown form: %j" (string/format other) (error))
      )
    )
  (while (not (empty? fold-stack))
    (pop-fold-stack))

  (html-tree/generate-html ~(html ,head (body ,;body))))
