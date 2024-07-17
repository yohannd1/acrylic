#!/usr/bin/env janet

(import ../lib/parser)
(import ../lib/html)
(import ./optparser)

(defn file-get-contents [path]
  (with [fd (file/open path :rn)]
    (:read fd math/int32-max)))

(defn show-help-and-exit []
  (eprint
    ``Usage: acr2html [OPTION]... <FILE>
    Options:
      -h, --help: show help and exit
      -k, --katex: specify katex path (relative to output file)
      -o, --output: output file (if not specified, will print to stdout)
    ``)
  (os/exit 2))

(defn main [_ & args]
  (def op (optparser/new args))
  (def {:is-over is-over
        :get-option get-option
        :get-value get-value} op)

  (var input-path nil)
  (var katex-path "")
  (var output-path nil)
  (var verbose false)

  (defn is-any-of [needle haystack]
    (-> (find |(= $ needle) haystack)
        (nil?) (not)))

  (while (not (is-over))
    (def opt (get-option))

    (cond
      (nil? opt)
      (let [val (get-value)]
        (unless (nil? input-path)
          (eprint "Error: filename already given.")
          (show-help-and-exit))
        (set input-path val))

      (is-any-of opt ["-h" "--help"])
      (do
        (show-help-and-exit))

      (is-any-of opt ["-k" "--katex"])
      (do
        (def path (get-value))
        (when (nil? path)
          (eprint "Error: expecting argument for -k/--katex")
          (show-help-and-exit))
        (cond
          (or (= path "")
              (string/has-suffix? "/" path))
          (set katex-path path)

          # add trailing / if it isn't present
          true
          (set katex-path (string path "/"))))

      (is-any-of opt ["-v" "--verbose"])
      (set verbose true)

      (is-any-of opt ["-o" "--output"])
      (do
        (def path (get-value))
        (when (nil? path)
          (eprint "Error: expecting argument for -o/--output")
          (show-help-and-exit))
        (set output-path path))
      )
    )

  (when (nil? input-path)
    (eprint "Error: input path not specified")
    (show-help-and-exit))

  (def contents (file-get-contents input-path))
  (def result (parser/parse contents))
  (when verbose
    (eprintf "Parse results: %j" result))

  (def html (html/to-html result {:katex-path katex-path}))

  (def output-file
    (if (nil? output-path)
      stdout
      (file/open output-path :wn)))

  (:write output-file html)
  )
