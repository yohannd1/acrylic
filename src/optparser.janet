(defn new
  ```A simple argument parser.```
  [args-list]

  (def len (length args-list))
  (var index 0)

  (defn is-over []
    (>= index len))

  (defn get-option
    ```Returns the next argument if it is an option.```
    []

    (label -result
      (when (is-over) (return -result nil))
      (def arg (in args-list index))
      (unless (string/has-prefix? "-" arg)
        (return -result nil))

      (++ index)
      (return -result arg))
    )

  (defn get-value
    ```Return the next argument if it is a value.```
    []

    (label -result
      (when (is-over) (return -result nil))
      (def arg (in args-list index))
      (when (string/has-prefix? "-" arg)
        (return -result nil))

      (++ index)
      (return -result arg))
    )

  {:get-option get-option
   :get-value get-value
   :is-over is-over})
