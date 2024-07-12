(declare-project
  :name "acrylic-parser"
  :description "an acrylic parser and html converter"
  :dependencies [])

(declare-executable
  :name "acr2html"
  :entry "src/acr2html.janet"
  :install true)
