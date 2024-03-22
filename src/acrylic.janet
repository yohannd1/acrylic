(defn make-peg [options]
  (def opt-indent (-> options (in :indent 2)))
  (def indent-str
    (case
      (-> opt-indent (type) (= :number)) (string/repeat " " opt-indent)
      (= opt-indent :tab) "\t"
      (error (string "Unknown value for indent option: " opt-indent))))

  (peg/compile
    ~{:main (* (some (+ :line-spaced :line-normal)))

      :line-spaced (* (/ :line-content ,|[:line-spaced $]) (at-least 2 "\n"))
      :line-normal (* (/ :line-content ,|[:line-normal $]) "\n")
      :line-content (<- (any (if-not "\n" 1)))

      :indent ,indent-str
      }))

(defn parse
  ```
  Parse the acrylic text document in question.
  ```
  [text]
  (def p (make-peg {:indent 2}))
  (:match p text)
  )

(defn to-html [ast]
  (def buf @"")

  (each node ast
    (match node
      [:line-spaced text] (buffer/push-string buf (string/format "%s<br/><br/>" text))
      [:line-normal text] (buffer/push-string buf (string/format "%s<br/>" text))
      _ (error "TODO")
    ))

  buf)
