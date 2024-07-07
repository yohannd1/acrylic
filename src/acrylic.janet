(defn make-peg [options]
  (def opt-indent (-> options (in :indent 2)))
  (def indent-str
    (case
      (-> opt-indent (type) (= :number)) (string/repeat " " opt-indent)
      (= opt-indent :tab) "\t"
      (error (string "Unknown value for indent option: " opt-indent))))

  (defn process-latex-math-inline [& args]
    [:latex-math-inline (string/join args)])

  (defn named-capture [name]
    (defn callback [& args] [name ;args])
    callback)

  (peg/compile
    ~{:main (* (some (+ :line-spaced :line-normal)))
      :line-spaced (* (/ :line-content ,(named-capture :line-spaced)) (at-least 2 "\n"))
      :line-normal (* (/ :line-content ,(named-capture :line-normal)) "\n")

      :line-content (any (+ (/ (some " ") ,|" ")
                            :line-content-latex-inline
                            :line-content-word
                            ))
      :line-content-word (<- (any (if-not (set " \n") 1)))
      :line-content-latex-inline (/ (* "${" :latex-math-block "}") ,process-latex-math-inline)

      :latex-math-block (any (+ :latex-math-nest
                                :latex-math-text))
      :latex-bracket (set "{}")
      :latex-math-text (<- (any (+ "\\{" "\\}" (if-not :latex-bracket 1))))
      :latex-math-nest (* (<- "{") :latex-math-block (<- "}"))

      :indent ,indent-str
      }))

(def- header-peg
  (peg/compile
    ~{:main (* (any :header)
               (/ (<- :tail) ,|['tail $]))
      :header (<- (* "%:" :identifier (some :s) (any (if-not "\n" 1)) (some "\n")))
      :tail (any 1)
      :identifier (some (if-not (set "@%:") :S))
      }))

(defn- front [arr]
  (let [len (length arr)]
    (array/slice arr 0 (dec len))))

(defn parse [str]
  (def body-peg (make-peg {:indent 2}))

  # TODO: extract key-value pairs out of header
  (def header-and-body (:match header-peg str))

  {:header (front header-and-body)
   :body (let [[_ body-str] (last header-and-body)]
           (:match body-peg body-str))
   })

(defn to-html [ast]
  (def buf @"")
  (defn ps [& args]
    (loop [s :in args]
      (buffer/push-string buf s)))

  (defn process-unit [node]
    (match node
      [:latex-math-inline text]
      (ps "LATEX{<code>" text "}")

      other
      (ps other)
      )
    )

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
