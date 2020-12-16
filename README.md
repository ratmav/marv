marv
====

(mar)kdown (v)im: markdown html and pdf previews

## installation

marv relies on pandoc to render html and pdf files. follow the [installation instructions](https://pandoc.org/installing.html) for your os.

use git or your plugin manager of choice to install marv.

### commands

* `MarvHTML`: renders markdown to a html tempfile and opens the tempfile with the system default web browser.
* `MarvPDF`: renders markdown to a pdf tempfile and opens the tempfile with the system default pdf viewer.

#### mapping example

add the following to your vim configuration to preview markdown in html and pdf, respectively:

```vimscript
" marv {{{
nnoremap <silent><Leader>h :execute 'MarvHTML'<CR>
nnoremap <silent><Leader>p :execute 'MarvPDF'<CR>
" }}}
```

## acknowledgements

* [the pandoc contributors and maintainers.](https://github.com/jgm/pandoc/graphs/contributors)
* [steve losh's](https://stevelosh.com/) book, [learn vimscript the hard way](https://learnvimscriptthehardway.stevelosh.com/), was a great and useful read.
