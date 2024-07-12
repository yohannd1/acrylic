(defn- make-peg [options]
  (def opt-indent (-> options (in :indent 2)))

  (def indent-str
    (cond
      (-> opt-indent (type) (= :number))
      (string/repeat " " opt-indent)

      (= opt-indent "tab")
      "\t"

      (-> (string/format "Unknown value for indent option: %j" opt-indent) (error))
      ))

  (defn process-latex-math-inline [& args]
    [:latex-math-inline (string/join args)])

  (defn process-latex-math-line [& args]
    [:line-latex (string/join args)])

  (defn named-capture [name]
    (defn callback [& args] [name ;args])
    callback)

  (defn process-indent [indent trailing-space]
    # FIXME: trailing-space is not being detected. make it detect, and error out when it does happen
    (def size
      (cond
        # for tabs
        (= indent-str "\t") (length indent)

        # for spaces
        (-> (length indent) (/ (length indent-str)))
        )
      )
    [:indent size])

  (peg/compile
    ~{:main (* (some (* :indent (+ :line-comment
                                   :line-latex
                                   :line-spaced
                                   :line-normal)))
               (at-most 1 :tail))

      :tail (/ (<- (some 1)) ,(named-capture :tail))

      :indent (/ (* (<- (any ,indent-str)) (<- (any " \t"))) ,process-indent)
      :line-comment (/ (* "%%" :not-newline (any "\n")) ,(named-capture :line-comment))
      :line-spaced (* (/ :line-content ,(named-capture :line-spaced)) (at-least 2 "\n"))
      :line-normal (* (/ :line-content ,(named-capture :line-normal)) "\n")
      :line-latex (/ (* (+ (* "$$:" :not-newline)
                           (* "$${" :latex-math-block "}"))
                        (some "\n"))
                     ,process-latex-math-line)

      :line-content (any (+ :whitespace
                            :escaped
                            :line-content-bold-italic
                            :line-content-bold
                            :line-content-italic
                            :line-content-code
                            :line-content-latex
                            :line-content-latex-toend
                            :line-content-comment-toend
                            :line-content-word
                            ))

      :whitespace (/ (some (set " \t")) ,|" ")
      :escaped (* "\\" (<- (if-not "\n" 1)))
      :line-content-word (<- (any (if-not (set " \t%_*$`\\\n") 1)))

      :line-content-bold (/ (* "*" (<- (any (if-not (set "*\n") 1))) "*")
                            ,(named-capture :bold))
      :line-content-italic (/ (* "_" (<- (any (if-not (set "_\n") 1))) "_")
                              ,(named-capture :italic))
      :line-content-bold-italic (/ (+ (* "*_" (<- (any (if-not (+ "_*" "\n") 1))) "_*")
                                      (* "_*" (<- (any (if-not (+ "*_" "\n") 1))) "*_"))
                                   ,(named-capture :bold-italic))
      :line-content-code (/ (* "`" (<- (any (if-not (set "`\n") 1))) "`")
                            ,(named-capture :code))

      :line-content-comment-toend (/ (* "%%" :not-newline) ,(named-capture :comment))

      :line-content-latex (/ (* "${" :latex-math-block "}") ,process-latex-math-inline)
      :line-content-latex-toend (/ (* "$:" :not-newline) ,process-latex-math-inline)

      :latex-math-block (any (+ :latex-math-nest
                                :latex-math-text))
      :latex-bracket (set "{}")
      :latex-math-text (<- (any (+ "\\{" "\\}" (if-not :latex-bracket 1))))
      :latex-math-nest (* (<- "{") :latex-math-block (<- "}"))

      # utility patterns (?)
      :not-newline (<- (any (if-not "\n" 1)))
      }))

(def- header-peg
  (peg/compile
    ~{:main (* (any :entry) # a set of entries
               (any "\n") # many whitespaces(?)
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

  (def ast (:match body-peg body-str))
  (match (last ast)
    [:tail t]
    (->
      ```Content was not fully parsed.
      Could not parse: %j
      AST until then: %j
      ```
      (string/format t (front ast))
      (error))

    _
    nil)

  {:header options
   :ast ast})
