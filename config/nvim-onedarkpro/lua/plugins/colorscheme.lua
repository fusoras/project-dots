-- Plugin: onedarkpro (colorscheme)
-- Qué hace: tema oscuro onedarkpro; fija el colorscheme de LazyVim a onedark_dark
-- Estado: activo
-- Desactivar: pon enabled = false en el primer spec y cambia colorscheme en LazyVim/LazyVim
return {
  {
    "olimorris/onedarkpro.nvim",
    name = "onedarkpro",
    lazy = false,
    priority = 1000,
    desc = "Tema oscuro onedarkpro",
  },
  {
    "LazyVim/LazyVim",
    opts = {
      colorscheme = "onedark_dark",
    },
  },
}
