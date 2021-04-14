" marv#preview#Render {{{
function! marv#preview#Render(extension) abort
  let sourcefile = expand('%:p')
  let tempfile = '/tmp/' . expand('%:t:r') . a:extension

  " only set the title metadata attribute for html
  if a:extension ==# '.pdf'
    let prefix = ':! pandoc -s -V geometry:margin=1in -o'
  else
    let prefix = ':! pandoc --metadata title="marv preview" -s -V geometry:margin=1in -o'
  endif

  " clean up old tempfiles, then build new tempfile
  silent execute ':! rm -f ' . tempfile
  execute prefix . ' "' . tempfile . '" "' . sourcefile . '"'
  echo 'marv: rendered ' . a:extension . ' preview.'

  return tempfile
endfunction
" }}}

" marv#preview#Open {{{
function! marv#preview#Open(os, tempfile) abort
  if a:os ==# 'darwin'
    silent execute ':! open ' . a:tempfile
  elseif a:os ==# 'linux'
    silent execute ':! xdg-open ' . a:tempfile
  endif
endfunction
" }}}

" marv#preview#Preview {{{
function! marv#preview#Preview(extension) abort
  " detect os.
  let os = marv#os#Detect()

  " are we on a supported os?
  if os !=# 'windows'
    " is pandoc installed?
    if executable('pandoc')
      " is the extension supported?
      if a:extension ==# '.pdf' || a:extension ==# '.html'
        "is the buffer in markdown?
        if expand('%:e') !=# 'md'
          echo 'marv: buffer is not a markdown file.'
        else
          " render the tempfile.
          let tempfile = marv#preview#Render(a:extension)

          " open the tempfile
          call marv#preview#Open(os, tempfile)
        endif
      else
        echo 'marv: only conversion to pdf or html is supported.'
      endif
    else
      echo 'marv: pandoc is not installed'
    endif
  else
    echo 'marv: windows support not implemented yet.'
  endif
endfunction
" }}}
