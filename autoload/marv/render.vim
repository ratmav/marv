" marv#render#Render {{{
function! marv#render#Render(extension) abort
  " detect os.
  let os = marv#os#Detect()

  " are we on a supported os?
  if os !=# 'windows'
    " is pandoc installed?
    if executable('pandoc')
      " is the extension supported?
      if a:extension == '.pdf' || a:extension == '.html'
        "is the buffer in markdown?
        if expand("%:e") != "md"
          echo 'marv: buffer is not a markdown file.'
        else
          let sourcefile = expand('%:t')
          let title = fnamemodify(sourcefile, ':r')
          let targetfile = '/tmp/' . title . a:extension

          " only set the title metadata attribute for html
          if a:extension == '.pdf'
            let prefix = ':! pandoc -s -V geometry:margin=1in -o'
          else
            let prefix = ':! pandoc --metadata title="' . title . '" -s -V geometry:margin=1in -o'
          endif

          " clean up old tempfiles, then build new tempfile
          silent execute ':! rm -f ' . targetfile
          silent execute prefix . ' ' . targetfile . ' ' . sourcefile

          " open the tempfile
          if os ==# 'darwin'
            silent execute ':! open ' . targetfile
          elseif os ==# 'linux'
            silent execute ':! xdg-open ' . targetfile
          endif
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
