-- Canonical Neovim adapter path for Ruff LSP.
-- Requires nvim-lspconfig.

require('lspconfig').ruff_lsp = {
  default_config = {
    cmd = { 'ruff', 'lsp' },
    filetypes = { 'ruff' },
    root_dir = function(fname)
      return require('lspconfig.util').find_git_ancestor(fname)
        or require('lspconfig.util').path.dirname(fname)
    end,
    single_file_support = true,
  },
}

require('lspconfig').ruff_lsp.setup({})
