(def- default-css
  ```
  body {}
  p.line-normal {}
  p.line-spaced {}
  ```)

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
      (ps "($) {<code>" text "</code>}")

      other
      (ps other)
      ))

  (ps `<html>`)
  (ps `<head>`)
  (ps `<meta charset="UTF-8"/>
       <meta name="viewport" content="width=device-width,initial-scale=1"/>`)
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

      other
      (error (string/format "Unknown form: %j" other))
      ))

  (ps `</body>`)
  (ps `</html>`)

  buf)
