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
  # TODO: implement @functions()
  # TODO: math support via ${}
  # TODO: implement $: syntax
  # TODO: implement $$: syntax
  # TODO: option "inherit" - inherit settings such as "indentation" and libraries(?? idk) on the header
  # TODO: implement @(raw-function-call) (BUT ONLY IF I FIGURE OUT SANDBOXING!!!!!!!)

  (def contents (file-get-contents path))
  (pp contents)

  (def result (acrylic/parse contents))
  (pp result)

  (def html (acrylic/to-html (in result :body)))
  (pp html)
  )
