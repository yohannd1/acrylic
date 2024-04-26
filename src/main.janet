#!/usr/bin/env janet

(import ./acrylic)

(defn file-get-contents [path]
  (with [fd (file/open path :rn)]
    (:read fd math/int32-max)))

(defn main [_ path]
  # TODO: parse header
  # TODO: get indent option (default=2, and can be tab if chosen)
  # TODO: parse the rest of the document, with `indent` being the indent unit
  # TODO: analyze the lines and group figure out the tree, based off indentation
  # TODO: implement bold, italic and code (plain text only)
  # TODO: implement %tags
  # TODO: implement @functions() / @functions{}
  # TODO: implement @string-receiving-functions: ...
  # TODO: tests for this. would be hella useful.
  # TODO: ${a }} should error. ${a {}} shouldnt. would that be too annoing to implement? since inside the latex directive there shouldnt be any other constructs
  # TODO: Keywords arguments as [] and positional arguments as () and {}. Would make things easier for me. Scribble based.
  # TODO: raw args?
  #   @code:
  #   @code[lang (Bom dia)]#{{
  #     this is raw but left-trimmed data
  #   }}
  #   @end
  # TODO: multiline arg composition
  #   @code:
  #   @code->
  #     [lang (Bom dia)]
  #     {This is arg 1}
  #     {This is arg 2!!}
  #   @end

  (def contents (file-get-contents path))
  (pp contents)

  (def result (acrylic/parse contents))
  (pp result)

  (def html (acrylic/to-html (in result :body)))
  (pp html)

  (let [p (peg/compile ~{:main (* :s* (any :element))
                         :element (* (+ :word :latex) :s*)
                         :word (/ (<- :w+) ,|[:word $])
                         :latex (/ (* "${" (<- (any (if-not "}" 1))) "}") ,|[:latex $])
                         })
        i ```foo bar ${x_2 = 0} baz```]
    (pp (peg/match p i))
    )
  )

# TODO: option "inherit" - inherit settings such as "indentation" and libraries(?? idk) on the header
# TODO: implement @(raw-function-call) (BUT ONLY IF I FIGURE OUT SANDBOXING!!!!!!!)
