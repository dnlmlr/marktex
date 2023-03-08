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

## Current state

This is a non-exhaustive list of features that would be nice to have at some point and their current state:

- ✔️ Normal plain text
- ✔️ Justified text
- ❌ Anything other than justified text (not sure yet how to implement this in terms of syntax)
- ✔️ Hyphenation (curretly supposted: en, de)
- ✔️ Headings (h1 - h6): Not looking great due to the default styling, but supported
- ✔️ Paragraphs (clear separation between paragraphs)
- ✔️ **Bold / Strong** text 
- ✔️ *Italic / Emphasis* text
- 🛠️ ~~Strikethrough~~ text: Unoptimized and still buggy
- ✔️ Math formulas using latex syntax (codeblock with `math` as info)
  - ✔️ Block
  - ❌ Inline
- ✔️ Images (simply using the normal markdown image syntax)
  - ✔️ Scale images relative to page width (abusing the title field `![](./myimage.png "scale = 0.5")`)
  - ❌ Smart compression (I'm not yet sure how the PDF stack deals with the images, but `ps2pdf` can make it smaller. Maybe the images can be compressed more, or be prescaled to match the PPI or smth.)
  - ❌ Deduplicate images if the exact same image is used multiple times
- ✔️ Unordered lists
- ❌ Ordered (enumerated) lists
- ❌ Task lists
- 🛠️ Code blocks
  - 🛠️ Simple monospace rendering (Works, but not yet stylable. Also no overflows or line numberings)
  - 🛠️ Syntax highlighted rendering (Works, but not yet stylable. Also no overflows or line numberings)
  - ❌ Inline Code blocks
- ❌ Tables
- 🛠️ Block quotes: Currently just makes the text italic and slightly more gray
- ✔️ Page breaks (start a new page using `---`)
- ❌ Automatic table of content
- ❌ PDF table of content (not an actual rendered page, but the PDF embedded info)
- ❌ Bibliography
- ❌ Citation
- ❌ Footnotes
- ❌ Links
  - ❌ Hyperlinks
  - ❌ References to chapters / headings
  - ❌ References to images / tables / listings / ...
- ❌ Including other files
- ✔️ Automatically included default fonts
- ✔️ Font subsetting to reduce the output PDF size
  - ✔️ Remove fully unused fonts
  - ✔️ Subset the main text fonts according to glyphs occuring in the unparsed Markdown input
  - ✔️ Subset the math font
  - ✔️ Correctly subset all actually used glyphs (this is done using the subsetting implementation of the forked printpdf and genpdf crates)
- ❌ Configuration (style) via yaml frontmatter

## Trying it out

This project is in a *very* early prototyping stage, but the current version can be tested out by installing it via: 
```
cargo install --git https://github.com/dnlmlr/marktex
```

After the installation, the program can be used by just calling `marktex` and the CLI help is of course available with `marktext --help`.

