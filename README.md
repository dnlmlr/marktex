# MarkTex

> A lightweight (but much less powerful) single executable alternative to LaTeX, based on Markdown syntax that *just works*™. (disclaimer: It actually doesn't just work yet)

The goal of this project is not to replace LaTeX or get feature parity since the gigantic stack of LaTeX packages is pretty much impossible to compete with, much less to surpass it. But that gigantic stack together with the old and sluggish engines is also one of the biggest drawbacks of LaTeX:
- It's annoying to install
- It takes a lot of disk space (yeah I know disk space is not really a big problem but it annoys the hell out of me)
- Reproducible build can be hard due to the packages
- It's often quite slow to compile documents
- The syntax can be extremely annoying when actually just wanting to write text

This project aims to create decently looking (similar to the *professional / scientific* LaTeX look) PDF files from a Markdown superset. The base version is pretty much just creating prestyled PDF files from plaintext. More features will be gradually added, starting with Markdown features (headers, bold text, lists, ...) and after that more complex LaTeX features like math typesetting, bibtex-like functionalities for citation, plotting and more. Eventually some limited compile-time scripting features might also be added.

While adding features the focus will be kept on the user experience and especially on the lightweight single-executable. 

## Trying it out

This project is in a *very* early prototyping stage, but the current version can be tested out by installing it via: 
```
cargo install --git https://github.com/dnlmlr/marktex
```

After the installation, the program can be used by just calling `marktex` and the CLI help is of course available with `marktext --help`.

