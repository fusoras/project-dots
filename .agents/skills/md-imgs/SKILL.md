---
name: md-imgs
description: Image conventions for Markdown and Obsidian vaults. Use when editing .md files with images, emoji icons, twemoji SVGs, or Obsidian embeds, or when asked to size images with the Obsidian |width syntax (e.g. ![|10](), ![[file|10]]). Covers emoji images and local file embeds.
---

# Markdown / Obsidian Image Conventions

Standard image conventions for `.md` files in the vault, following Obsidian's
`|width` size syntax.

## Emoji images

Any image whose **alt text** or **URL** contains an emoji must include a
default width of `10`.

- Pattern: `![emoji|10](url)`
- Emoji detection applies to:
  - Alt text with an emoji character: `![❌]`, `![⭐]`, `![➡]`
  - Twemoji CDN URLs: `https://cdn.jsdelivr.net/gh/jdecked/twemoji@...`

### Before / after

```markdown
<!-- before -->
![❌](https://cdn.jsdelivr.net/gh/jdecked/twemoji@16.0.1/assets/svg/274c.svg)
![⭐](https://cdn.jsdelivr.net/gh/jdecked/twemoji@16.0.1/assets/svg/2b50.svg)

<!-- after -->
![❌|10](https://cdn.jsdelivr.net/gh/jdecked/twemoji@16.0.1/assets/svg/274c.svg)
![⭐|10](https://cdn.jsdelivr.net/gh/jdecked/twemoji@16.0.1/assets/svg/2b50.svg)
```

## Local embeds

Obsidian embeds of local files (`.avif`, `.png`, `.webp`, ...) use the embed
syntax. When a width is required, append it after a pipe inside the double
brackets:

- `![[file.avif]]` → `![[file.avif|10]]`

Do not convert embeds to `![...]()` markdown — keep the original embed form.

## Obsidian width syntax

| Form              | Example                        | Description                          |
| ----------------- | ------------------------------ | ------------------------------------ |
| Markdown image    | `![alt\|10](url)`              | Remote or URL-based image, width 10  |
| Embed (local file)| `![[file.avif\|10]]`           | Local file embed, width 10           |
| Embed (note)      | `![[Note name]]`               | Note embed, never add a width        |

Rules:
- The width unit is pixels (no `px` suffix).
- Only add `|10` to images that need the default emoji sizing; notes embeds
  (`![[Note]]`) never get a width.
