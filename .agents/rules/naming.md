# Rule: Category and Configuration Naming Convention

## Principles

1. **No Generic Names**:
   - Never use generic terms like `shell`, `editor`, `terminal`, `config`, or `dotfiles` as canonical category names or folder names in `config/`.
   
2. **Distinctive & Specific Naming**:
   - Combine the component/tool type with its specific variant, theme, style, or preset (e.g., `shell-tokyonight`, `lazyvim-minimal`, `tmux-dracula`).
   - Keep names simple, clear, and hyphenated (`kebab-case`).

3. **Collision Prevention**:
   - Multiple configurations for the same tool/component will exist (e.g., different shell themes or neovim presets). Specific naming ensures each configuration lives in a dedicated, collision-free subdirectory under `config/<category-name>/`.

4. **Explicit Alias Governance**:
   - Aliases must ONLY be created when explicitly requested and defined by the user.
   - Agents must NEVER generate, infer, or automatically add aliases without explicit user instruction.
