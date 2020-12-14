function! marv#init#Init() abort
  command! MarvHTML :call marv#render#Render('.html')
  command! MarvPDF :call marv#render#Render('.pdf')
endfunction
