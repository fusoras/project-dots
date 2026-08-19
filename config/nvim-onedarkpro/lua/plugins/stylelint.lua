-- Plugin: stylelint_lsp (nvim-lspconfig)
-- Qué hace: LSP de Stylelint para validar CSS/HTML/Astro; snippets para CSS/PostCSS
-- Estado: activo
-- Desactivar: pon enabled = false en el spec de abajo
return {
  {
    "neovim/nvim-lspconfig",
    desc = "Stylelint LSP (css/html/astro)",
    opts = {
      servers = {
        stylelint_lsp = {
          settings = {
            stylelint = {
              validate = { "css", "html", "astro" },
              snippet = { "css", "postcss" },
            },
          },
        },
      },
    },
  },
}
