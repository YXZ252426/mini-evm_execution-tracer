## 3.8
learn the usage of eyre, `wrap_err`, `wrap_err_with`

learn how a proj is construct gradually: code line by line by hand!!

when to define a new function: abstract as a sub problem while keep the main logic clear

`TxENV::builder`, `CacheDB::new(EmptyDB::new())`, `Context::mainnet()`, chain mode， different design pattern

why I not build this proj in person without refer, too many dependencies, the usage of rust std, revm primitives, different revm components, error handling, too many apis

some inspiration: It is actually to build and drive a big "truck", how to manage it with so many details, maintain a big proj, pr for opensource proj.

but actually do not need to worry too much, because different proj usually follower certain patterns to achieving easy-use, for example: `parse`, `to_string`, use of closure, different function classified with different impl, the archi design for api in a proj: `revm::{database, primitives, state}`

**type transition!**: from, into, as, parse, to...