#!/usr/bin/env janet

(import ./acrylic)

(defn file-get-contents [path]
  (with [fd (file/open path :rn)]
    (:read fd math/int32-max)))

(defn main [_ path]
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
