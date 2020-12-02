marv
====

**alpha software. work in progress.**

(mar)kdown (v)im: syntax highlighting, folding, and pdf or html previews for markdown

## installation

marv relies on pandoc to render html and pdf files. follow the [installation instructions](https://pandoc.org/installing.html) for your os.

once pandoc is installed, use your plugin manager of choice to install marv. for example, with [vim-plug](https://github.com/junegunn/vim-plug):

    * add the following to your vim configuration: `Plug 'ratmav/marv'`
    * run `:PlugInstall`

## use

* call or map `MarvHTML` to render markdown to html and open using the default browser.
* call or map `MarvPDF` to render markdown to pdf and open using the defaul pdf viewer.

## roadmap

### windows support

marv will hopefully "just work" on windows; just need to sort out the commands to open the tempfiles in the default applications.

# acknowledgements

* [ben williams'](https://plasticboy.com/) plugin, [vim-markdown](https://github.com/plasticboy/vim-markdown), was a great resource.
* [steve losh's](https://stevelosh.com/) book, [learn vimscript the hard way](https://learnvimscriptthehardway.stevelosh.com/), was a great and useful read.
