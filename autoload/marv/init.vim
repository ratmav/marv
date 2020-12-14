function! marv#init#Init() abort
  command! MarvHTML :call marv#preview#Preview('.html')
  command! MarvPDF :call marv#preview#Preview('.pdf')
endfunction
