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

  (def result (parser/parse contents))
  (pp result)
  (print)

  (def html (html/to-html result {}))
  # (pp html)
  # (print)

  (def output-file (file/open "out/output.html" :w))
  (:write output-file html)
  )
