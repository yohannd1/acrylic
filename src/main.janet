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
  # TODO: tests for this. would be hella useful.
  # TODO: implement bold, italic and code (plain text only)
  # TODO: implement %tags
  # TODO: math support via ${}
  # TODO: implement $: syntax
  # TODO: implement $$: syntax
  # TODO: option "inherit" - inherit settings such as "indentation" and libraries(?? idk) on the header
  # TODO: implement @(raw-function-call) (BUT ONLY IF I FIGURE OUT SANDBOXING!!!!!!!)
  # TODO: implement @functions() / @functions{}
  # TODO: tests for this. would be hella useful.
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
  (print)

  (def ast (acrylic/parse contents))
  (pp ast)
  (print)

  (def html (acrylic/to-html (in ast :body)))
  (pp html)
  (print)

  (def output-file (file/open "output.html" :w))
  (:write output-file html)
  )
