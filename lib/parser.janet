(defn- make-peg [options]
  (def opt-indent (-> options (in :indent 2)))

  (def indent-str
    (case
      (-> opt-indent (type) (= :number)) (string/repeat " " opt-indent)
      (= opt-indent "tab") "\t"
      (error (string "Unknown value for indent option: " opt-indent))))

  (defn process-latex-math-inline [& args]
    [:latex-math-inline (string/join args)])

  (defn process-latex-math-line [& args]
    [:line-latex (string/join args)])

  (defn named-capture [name]
    (defn callback [& args] [name ;args])
    callback)

  (peg/compile
    ~{:main (* (some (+ :line-latex :line-spaced :line-normal)))

      :line-spaced (* (/ :line-content ,(named-capture :line-spaced)) (at-least 2 "\n"))
      :line-normal (* (/ :line-content ,(named-capture :line-normal)) "\n")
      :line-latex (/ (* (+ (* "$$:" :not-newline)
                           (* "$${" :latex-math-block "}")
                           )
                        (some "\n"))
                     ,process-latex-math-line)

      :whitespace (/ (some (set " \t")) ,|" ")
      :line-content (any (+ :whitespace
                            :line-content-latex-inline
                            :line-content-latex-inline-to-end
                            :line-content-word
                            ))

      :not-newline (<- (any (if-not "\n" 1)))

      :line-content-word (<- (any (if-not (set " \n") 1)))
      :line-content-latex-inline (/ (* "${" :latex-math-block "}") ,process-latex-math-inline)
      :line-content-latex-inline-to-end (/ (* "$:" :not-newline) ,process-latex-math-inline)

      :latex-math-block (any (+ :latex-math-nest
                                :latex-math-text))
      :latex-bracket (set "{}")
      :latex-math-text (<- (any (+ "\\{" "\\}" (if-not :latex-bracket 1))))
      :latex-math-nest (* (<- "{") :latex-math-block (<- "}"))

      :indent ,indent-str
      }))

(def- header-peg
  (peg/compile
    ~{:main (* (any :entry) # a set of entries
               (some "\n") # many whitespaces(?)
               :body) # the rest of the document

      :entry (/ (* "%:" :identifier (some :s) :not-newline "\n")
                ,tuple)

      :identifier (<- (some (if-not (set "@%:") :S)))
      :not-newline (<- (any (if-not "\n" 1)))

      :body (/ (<- (any 1)) ,|[:body $])
      }))

(defn- front [arr]
  (let [len (length arr)]
    (array/slice arr 0 (dec len))))

(defn parse
  ```Parse the input string `str`, returning the AST with the parsed contents.```
  [str]

  (def header-result (:match header-peg str))
  (def options @{})
  (loop [[key val] :in (front header-result)]
    (set (options (keyword key)) val))

  (def body-peg (make-peg options))
  (def [_ body-str] (last header-result))

  {:header options
   :ast (:match body-peg body-str)})
