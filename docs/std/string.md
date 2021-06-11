String
======

Functions
---------

`show :: (Any a) => a -> String`

Transforms a value into its literal form.

```clojure
(show "a") ;; "'a'"
(show 44) ;; "44"
```