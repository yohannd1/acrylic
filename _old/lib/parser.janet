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

  (defn capture-inner-line [name]
    (defn callback [& args]
      {:type name :content args})
    callback)

  (defn capture-inner-line-latex [& args]
    (def f (capture-inner-line 'latex))
    (f (string/join args)))

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

  (defn process-inner-task [type- & args]
    {:type ['task type-] :content args}
    )

  (defn process-line [& args]
    (def [indent-tup inner-line spacing] args)
    (def [_ indent] indent-tup)
    (def {:type type- :content content} inner-line)
    (def tags @[])

    (each c content
      (match c
        [:tag t] (array/push tags t)))

    {:indent indent
     :type type-
     :content content
     :tags tags
     })

  (defn spacing-line []
    {:type 'spacing})

  (peg/compile
    ~{:main (* (some :line) (? :tail))
      :line (+ (/ (some "\n") ,spacing-line)
               (/ (* :indent :line-inner "\n") ,process-line))
      :tail (/ (<- (some 1)) ,(named-capture :tail))

      :indent (/ (* (<- (any ,indent-str)) (<- (any " \t"))) ,process-indent)

      # resulting format: struct with keys type,content
      :line-inner (+ :line-inner-comment
                     :line-inner-latex-display
                     :line-inner-task
                     :line-inner-generic)

      :line-inner-comment (/ (* "%%" :not-newline) ,(capture-inner-line 'comment))
      :line-inner-latex-display (/ (+ (* "$$:" :not-newline)
                                      (* "$${" :latex-math-block "}"))
                                   ,capture-inner-line-latex)
      :line-inner-task (/ (* :checkbox :line-content) ,process-inner-task)
      :checkbox (+ (* "(" (<- (set "x -")) ")")
                   (* "[" (<- (set "x -")) "]"))
      :line-inner-generic (/ (* :line-content) ,(capture-inner-line 'generic))

      :line-content (any (+ :line-content-whitespace
                            :escaped
                            :line-content-url
                            :line-content-tag
                            :line-content-bold-italic
                            :line-content-bold
                            :line-content-italic
                            :line-content-code
                            :line-content-latex
                            :line-content-latex-toend
                            :line-content-comment-toend
                            :line-content-stray-character
                            :line-content-word
                            ))

      :line-content-whitespace (/ (some :set-whitespace) ,|" ")

      :escaped (* "\\" (<- (if-not "\n" 1)))
      :line-content-word (<- (any (if-not (set " \t%_*$`\\\n") 1)))

      :set-whitespace (set " \t")

      :line-content-url (/ (<- (* :a+ "://" (some (if-not (set " \t\n") 1))))
                           ,(named-capture :url))

      :line-content-tag (/ (* "%" (<- (some (if-not (set " \t%\n") 1))))
                           ,(named-capture :tag))

      # TODO: make inline-styles require either zero characters or any amount, as long as the first one isn't a space character (I did that with bold already here but it seems frail)
      :line-content-bold (/ (* "*" (<- (* (if-not (set " \t*") 1) (any (if-not (set "*\n") 1)))) "*")
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

      :line-content-stray-character (<- (set "*$%_"))

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
  (when (nil? ast)
    (error ```Failed to parse the content at all.```))

  (match (last ast)
    [:tail t]
    (->
      ```Content was not fully parsed.
      Could not parse: %j
      ```
      (string/format t)
      (error))

    _
    nil)

  {:header options
   :ast ast})
