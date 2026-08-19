-- Plugin: oil.nvim
-- Qué hace: explorador de archivos estilo editor (reemplazo de netrw). Keymaps:
--   - = abrir padre, <leader>- = cwd, <leader>o = flotante, <leader>. = dir del archivo actual
-- Estado: activo
-- Desactivar: pon enabled = false en el spec de abajo
return {
  "stevearc/oil.nvim",
  desc = "Explorador de archivos estilo editor",
  lazy = false,
  keys = {
    { "-", "<CMD>Oil<CR>", desc = "Open Oil (parent dir)" },
    { "<leader>-", "<CMD>Oil<CR>", desc = "Open Oil (cwd)" },
    { "<leader>o", "<CMD>Oil --float<CR>", desc = "Open Oil (floating)" },
  },
  opts = {
    default_file_explorer = true,
    restore_win_options = true,
    prompt_save_on_select_new_entry = true,
    keymaps = {
      ["g?"] = "actions.show_help",
      ["<CR>"] = "actions.select",
      ["<C-s>"] = { "actions.select", opts = { vertical = true } },
      ["<C-v>"] = { "actions.select", opts = { horizontal = true } },
      ["<C-t>"] = { "actions.select", opts = { tab = true } },
      ["<C-p>"] = "actions.preview",
      ["<C-c>"] = "actions.close",
      ["<C-r>"] = "actions.refresh",
      ["-"] = "actions.parent",
      ["_"] = "actions.open_cwd",
      ["`"] = "actions.cd",
      ["~"] = { "actions.cd", opts = { scope = "tab" } },
      ["gs"] = "actions.change_sort",
      ["gx"] = "actions.open_external",
      ["g."] = "actions.toggle_hidden",
      ["g\\"] = "actions.toggle_trash",
      ["q"] = "actions.close",
    },
    use_default_keymaps = false,
    view_options = {
      show_hidden = true,
      is_hidden_file = function(name)
        return vim.startswith(name, ".")
      end,
      is_always_hidden = function(name)
        return name == ".." or name == ".git"
      end,
      natural_order = true,
      sort = {
        { "type", "asc" },
        { "name", "asc" },
      },
    },
    float = { padding = 2, max_width = 100, max_height = 30, border = "rounded" },
  },
  dependencies = { "nvim-tree/nvim-web-devicons" },
  config = function(_, opts)
    require("oil").setup(opts)
    vim.api.nvim_create_autocmd("FileType", {
      pattern = "oil",
      callback = function()
        vim.opt_local.colorcolumn = ""
        vim.opt_local.signcolumn = "no"
      end,
    })
    vim.keymap.set("n", "<leader>.", function()
      local oil = require("oil")
      local file = vim.api.nvim_buf_get_name(0)
      if file ~= "" then
        oil.open(vim.fn.fnamemodify(file, ":h"))
      else
        oil.open()
      end
    end, { desc = "Open Oil in current file's directory" })
  end,
}
