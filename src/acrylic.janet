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

  (each node ast
    (match node
      [:line-spaced text] (buffer/push-string buf (string/format "%s<br/><br/>" text))
      [:line-normal text] (buffer/push-string buf (string/format "%s<br/>" text))
      _ (error "TODO")
    ))

  buf)
