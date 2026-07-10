-- Esconde a pasta ".vscode" do nvim-tree apenas neste projeto
require("nvim-tree").setup({
  filters = {
    custom = { "^.git$", "^.vscode$" }
  }
})
