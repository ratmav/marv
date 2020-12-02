" DetectOS {{{
function! marv#running#DetectOS()
  if has("win64") || has("win32") || has("win16")
      return "windows"
  else
      if has("mac")
        return "darwin"
      else
        return "linux"
      endif
  endif
endfunction
" }}}

" Preview {{{
function! s:marv#running#Preview(extension, os)
  let sourcefile = expand("%:t")
  let title = fnamemodify(sourcefile, ":r")
  let targetfile = "/tmp/" . title . a:extension

  " only set the title metadata attribute for html
  if a:extension == ".pdf"
    let prefix = ':! pandoc -s -V geometry:margin=1in -o'
  else
    let prefix = ':! pandoc --metadata title="' . title . '" -s -V geometry:margin=1in -o'
  endif

  " clean up old tempfiles and build new ones
  execute ":! rm -f " . targetfile
  execute prefix . " " . targetfile . " " . sourcefile

  " open the tempfile
  if a:os ==# "darwin"
    execute ":! open " . targetfile
  elseif a:os ==# "linux"
    execute ":! xdg-open " . targetfile
  endif
endfunction
" }}

" CheckedPreview {{{
function! s:marv#running#CheckedPreview(extension)
  let os = marv#running#DetectOS()

  " are we on windows?
  if os ==# "windows"
    echo "windows support not implemented yet."
  else
    " is pandoc installed?
    if executable("pandoc")
      " is the extension supported?
      if a:extension == ".pdf" || a:extension == ".html"
        "is the buffer in markdown?
        if expand("%:e") != "md"
          echo "buffer is not a markdown file"
        else
          call s:marv#running#Preview(a:extension, os)
        endif
      else
        echo "only conversion to pdf or html is supported."
      endif
    else
      echo "pandoc is not installed"
    endif
  endif
endfunction
" }}}

command! -buffer MarvHTML call s:marv#running#CheckedPreview('html')
command! -buffer MarvPDF call s:marv#running#CheckedPreview('pdf')
