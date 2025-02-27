local M = {}

-- OS detection
function M.detect_os()
  if vim.fn.has('win64') == 1 or vim.fn.has('win32') == 1 or vim.fn.has('win16') == 1 then
    return 'windows'
  elseif vim.fn.has('mac') == 1 then
    return 'darwin'
  else
    return 'linux'
  end
end

-- Architecture detection
function M.detect_arch()
  -- Check if uname is available
  if vim.fn.executable('uname') == 1 then
    local uname_output = vim.fn.system('uname -m')
    -- Check for ARM64 architectures
    if string.match(uname_output, 'arm64') or string.match(uname_output, 'aarch64') then
      return 'arm64'
    else
      -- Default to x86_64 for non-ARM systems
      return 'amd64'
    end
  else
    -- Fallback if uname isn't available
    if vim.fn.has('mac') == 1 and string.match(vim.fn.system('sysctl -n hw.optional.arm64'), '1') then
      return 'arm64'
    else
      return 'amd64'
    end
  end
end

-- Get the binary path
function M.get_binary_path()
  -- Get the OS and architecture information
  local os_name = M.detect_os()
  local arch = M.detect_arch()
  local binary_name = 'marv-' .. os_name .. '-' .. arch
  -- Get the directory of this script
  local script_path = debug.getinfo(1, "S").source:sub(2) -- Remove the '@' prefix
  local script_dir = vim.fn.fnamemodify(script_path, ':h')
  -- Check if the script directory contains a bin folder
  local bin_path = script_dir .. '/bin/' .. binary_name
  if vim.fn.filereadable(bin_path) == 1 then
    return bin_path
  end
  -- If not found, search in runtime paths
  for _, rtp in ipairs(vim.api.nvim_list_runtime_paths()) do
    local potential_path = rtp .. '/bin/' .. binary_name
    if vim.fn.filereadable(potential_path) == 1 then
      return potential_path
    end
  end
  return nil
end

-- Check if server is running
function M.is_server_running(binary_path, file_path)
  -- Ensure absolute file path
  local absolute_file_path = vim.fn.fnamemodify(file_path, ':p')
  -- Run test command
  local cmd = vim.fn.shellescape(binary_path) .. ' --stop ' .. vim.fn.shellescape(absolute_file_path) .. ' 2>&1'
  local output = vim.fn.system(cmd)
  -- Check for "No server found" message
  if string.match(output, 'No server found') then
    return false
  else
    return true
  end
end

-- Start server
function M.start_server(binary_path, file_path)
  -- Ensure absolute file path
  local absolute_file_path = vim.fn.fnamemodify(file_path, ':p')

  -- Construct command
  local cmd = vim.fn.shellescape(binary_path) .. ' --start ' .. vim.fn.shellescape(absolute_file_path)

  -- Execute command
  vim.fn.system(cmd)

  -- Give the server a moment to start
  vim.cmd('sleep 500m')
  vim.api.nvim_echo({{"marv: started preview server", "Normal"}}, false, {})
end

-- Stop server
function M.stop_server(binary_path, file_path)
  -- Ensure absolute file path
  local absolute_file_path = vim.fn.fnamemodify(file_path, ':p')

  -- Construct command
  local cmd = vim.fn.shellescape(binary_path) .. ' --stop ' .. vim.fn.shellescape(absolute_file_path)

  -- Execute command
  vim.fn.system(cmd)
  vim.api.nvim_echo({{"marv: stopped preview server", "Normal"}}, false, {})
end

-- Toggle server
function M.toggle()
  -- Detect OS
  local os_name = M.detect_os()

  -- Check if OS is supported
  if os_name == 'windows' then
    vim.api.nvim_echo({{"marv: windows support not implemented yet", "ErrorMsg"}}, false, {})
    return
  end

  -- Check if buffer is markdown
  if vim.fn.expand('%:e') ~= 'md' then
    vim.api.nvim_echo({{"marv: buffer is not a markdown file", "WarningMsg"}}, false, {})
    return
  end

  -- Get binary path
  local binary_path = M.get_binary_path()
  if not binary_path then
    vim.api.nvim_echo({{"marv: could not find binary for your platform", "ErrorMsg"}}, false, {})
    return
  end

  -- Get file path
  local file_path = vim.fn.expand('%:p')

  -- Check if server is running
  local is_running = M.is_server_running(binary_path, file_path)

  if is_running then
    M.stop_server(binary_path, file_path)
  else
    M.start_server(binary_path, file_path)
  end
end

-- Kill all Marv processes
function M.kill_all()
  local os_name = M.detect_os()
  if os_name == 'windows' then
    -- Windows command to kill processes
    vim.fn.system('taskkill /F /IM marv*.exe 2>&1')
  else
    -- Unix command to kill processes (Linux and macOS)
    vim.fn.system('pkill -f "marv-.*-.*" 2>&1')
  end
  vim.api.nvim_echo({{"marv: all processes terminated", "Normal"}}, false, {})
end

-- Setup function to create commands
function M.setup()
  vim.api.nvim_create_user_command('MarvToggle', function()
    M.toggle()
  end, {})
  vim.api.nvim_create_user_command('MarvKillAll', function()
    M.kill_all()
  end, {})
end

-- No auto-initialization here - we'll let the plugin/marv.lua handle that

return M