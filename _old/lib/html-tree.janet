(defn- buf-push-all [buf & data]
  (each x data
    (buffer/push-string buf (string x))))

(defn replacer
  ``Creates a peg that replaces instances of patt with subst.
  Source: https://janet-lang.org/docs/peg.html``
  [patt subst]

  (peg/compile ~(% (any (+ (/ (<- ,patt) ,subst) (<- 1))))))

(defn escape-string
  ``Escape dangerous characters in `str` that can be used for injection.``
  [str]

  (def replace-patterns
    '(("&" "&amp;")
      ("<" "&lt;")
      (">" "&gt;")
      (`"` "&quot;")
      (`'` "&#39;")
      ("`" "&#96;")))

  (var result str)
  (each [patt sub] replace-patterns
    (set result (-> (replacer patt sub)
                    (peg/match result)
                    (get 0))))
  result)

(varfn node->html [x buf] (error "Placeholder"))

(defn- text-to-html [text buf]
  (buf-push-all buf (escape-string text)))

(def- void-tags
  {'area true
   'base true
   'br true
   'col true
   'embed true
   'hr true
   'img true
   'input true
   'link true
   'meta true
   'param true
   'source true
   'track true
   'wbr true})

(defn- void-tag? [tag]
  (-> void-tags (in tag) (truthy?)))

(defn node-to-html [node buf &opt raw-]
  (var raw (truthy? raw-))

  (def len (length node))
  (assert (> len 0) "node should not be empty")

  (def tag (get node 0))
  (assert (symbol? tag) "tag should be a symbol")

  # get attributes if we have them. whether we have them will affect where the children index starts, so we have to calculate that as well.
  (def [children-start-idx attrs]
    (let [x (get node 1)]
      (if (or (struct? x) (table? x))
        [2 x]
        [1 {}])))

  (def self-close?
    (and (>= children-start-idx (length node))
         (void-tag? tag)))

  # tag and attributes
  (buf-push-all buf "<" tag)
  (each [k v] (pairs attrs)
    (cond
      (symbol? k)
      (case k
        'raw (set raw (truthy? v))
        (-> "Unknown node config: %j" (string/format k) (error)))

      (= v true) (buf-push-all buf " " k)
      (= v false) nil
      (buf-push-all buf " " k `="` (escape-string v) `"`)))
  (buf-push-all buf (if self-close? "/>" ">"))

  (unless self-close?
    # children
    (loop [i :range [children-start-idx (length node)]]
      (node->html (get node i) buf raw))

    # close tag
    (buf-push-all buf "</" tag ">"))
  )

(varfn node->html [x buf &opt raw]
  (cond
    (string? x) (if raw
                  (buf-push-all buf x)
                  (text-to-html x buf))
    (or (array? x) (tuple? x)) (node-to-html x buf raw)
    (error (string "Expected string, array or tuple, found " x))))

(defn generate-html
  ``Generate a html-doctype document based off the `root` node.
  Returns a buffer with said HTML document.``
  [root]

  (def buf @"<!DOCTYPE html>\n")
  (node->html root buf)
  buf)

