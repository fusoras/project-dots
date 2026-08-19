-- Plugin: goto-preview
-- Qué hace: previsualiza definiciones/tipos/implementaciones/referencias LSP en ventana flotante
-- Estado: activo
-- Desactivar: pon enabled = false en el spec de abajo
return {
  "rmagatti/goto-preview",
  desc = "Preview de definiciones LSP en ventana flotante",
  event = "BufEnter",
  config = true,
  keys = {
    { "gpd", "<cmd>lua require('goto-preview').goto_preview_definition()<CR>", desc = "Preview definition" },
    { "gpD", "<cmd>lua require('goto-preview').goto_preview_declaration()<CR>", desc = "Preview declaration" },
    { "gpi", "<cmd>lua require('goto-preview').goto_preview_implementation()<CR>", desc = "Preview implementation" },
    { "gpy", "<cmd>lua require('goto-preview').goto_preview_type_definition()<CR>", desc = "Preview type definition" },
    { "gpr", "<cmd>lua require('goto-preview').goto_preview_references()<CR>", desc = "Preview references" },
    { "gP", "<cmd>lua require('goto-preview').close_all_win()<CR>", desc = "Close all preview windows" },
  },
  opts = {
    border = { "↖", "─", "┐", "│", "┘", "─", "└", "│" },
    references = {
      provider = "snacks",
    },
  },
}
