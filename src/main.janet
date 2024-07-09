#!/usr/bin/env janet

(import ../lib/parser)
(import ../lib/html)

(defn file-get-contents [path]
  (with [fd (file/open path :rn)]
    (:read fd math/int32-max)))

(defn main [_ path]
  (def contents (file-get-contents path))
  (pp contents)
  (print)

  (def ast (parser/parse contents))
  (pp ast)
  (print)

  (def html (html/to-html (in ast :body) {}))
  (pp html)
  (print)

  (def output-file (file/open "output.html" :w))
  (:write output-file html)
  )
