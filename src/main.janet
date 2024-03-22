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
  # TODO: implement @functions()
  # TODO: tests for this. would be hella useful.

  (def contents (file-get-contents path))
  (pp contents)

  (def ast (acrylic/parse contents))
  (pp ast)

  (def html (acrylic/to-html ast))
  (pp html)
  )

# TODO: option "inherit" - inherit settings such as "indentation" and libraries(?? idk) on the header
# TODO: implement @(raw-function-call) (BUT ONLY IF I FIGURE OUT SANDBOXING!!!!!!!)
