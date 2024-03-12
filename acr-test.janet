#!/usr/bin/env janet

(defn acrylic-make-peg [options]
  (def opt-indent (-> options (in :indent 2)))
  (def indent-str
    (case
      (-> opt-indent (type) (= :number)) (string/repeat " " opt-indent)
      (= opt-indent :tab) "\t"
      (error (string "Unknown value for indent option: " opt-indent))))

  (peg/compile
    ~{:main (* (any (* :line "\n")) (? :line))
      :line (<- (any (if-not "\n" 1)))
      :indent ,indent-str
      }))

(defn acrylic-parse [text]
  (def p (acrylic-make-peg {:indent 2}))
  (:match p text)
  )

(defn main [& args]
  # TODO: parse header
  # TODO: get indent option (default=2, and can be tab if chosen)
  # TODO: parse the rest of the document, with `indent` being the indent unit
  # TODO: analyze the lines and group figure out the tree, based off indentation
  # TODO: implement bold, italic and code (plain text only)
  # TODO: implement %tags
  # TODO: implement @functions()

  (def contents
    ```
    First line
    Second line
    ```)

  (pp contents)
  (pp (acrylic-parse contents))
  )

# TODO: option "inherit" - inherit settings such as "indentation" and libraries(?? idk) on the header
# TODO: implement @(raw-function-call) (BUT ONLY IF I FIGURE OUT SANDBOXING!!!!!!!)
