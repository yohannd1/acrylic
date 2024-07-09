(defn to-html
  ```Analyze `ast` and return its HTML representation.
  Options: TODO
  ```
  [ast opts]

  (def buf @"")
  (defn ps [& args]
    (loop [s :in args]
      (buffer/push-string buf s)))

  (defn process-unit [node]
    (match node
      [:latex-math-inline text]
      (ps "LATEX{<code>" text "</code>}")

      other
      (ps other)
      ))

  (each node ast
    (match node
      [:line-spaced & contents]
      (do
        (loop [c :in contents] (process-unit c))
        (ps "<br/><br/>"))

      [:line-normal & contents]
      (do
        (loop [c :in contents] (process-unit c))
        (ps "<br/>"))

      other
      (error (string/format "Unknown form: %j" other))
      ))

  buf)
