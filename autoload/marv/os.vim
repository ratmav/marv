" marv#os#Detect {{{
function! marv#os#Detect() abort
  if has("win64") || has("win32") || has("win16")
      let os = "windows"
  else
      if has("mac")
        let os = "darwin"
      else
        let os = "linux"
      endif
  endif

  return os
endfunction
" }}}
