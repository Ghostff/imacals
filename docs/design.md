---
version: alpha
name: Graphite
description: Cool greys, one lime signal.
colors:
  primary: "#ECEDEE"
  secondary: "#9CA3AF"
  tertiary: "#B4FF39"
  neutral: "#0E1013"
  surface: "#17191C"
  on-primary: "#0E1013"
typography:
  display:
    fontFamily: Inter Tight
    fontSize: 4rem
    fontWeight: 600
    letterSpacing: "-0.03em"
  h1:
    fontFamily: Inter Tight
    fontSize: 2.25rem
    fontWeight: 600
  body:
    fontFamily: Inter
    fontSize: 0.95rem
    lineHeight: 1.55
  label:
    fontFamily: JetBrains Mono
    fontSize: 0.75rem
    letterSpacing: "0.02em"
rounded:
  sm: 6px
  md: 10px
  lg: 14px
spacing:
  sm: 8px
  md: 16px
  lg: 32px
components:
  button-primary:
    backgroundColor: "{colors.tertiary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.md}"
    padding: 12px 20px
  card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.primary}"
    rounded: "{rounded.lg}"
    padding: 24px
---
## Overview

An engineering-grade dark palette. Carefully tuned greys across 10 stops, with a single lime-green for focus and CTAs.

## Colors

The palette is built around high-contrast neutrals and a single accent that drives interaction.

- **Primary (`#ECEDEE`):** Headlines and core text.
- **Secondary (`#9CA3AF`):** Borders, captions, and metadata.
- **Tertiary (`#B4FF39`):** The sole driver for interaction. Reserve it.
- **Neutral (`#0E1013`):** The page foundation.

## Typography

- **display:** Inter Tight 4rem
- **h1:** Inter Tight 2.25rem
- **body:** Inter 0.95rem
- **label:** JetBrains Mono 0.75rem

## Derived tokens

Hairlines, dividers, and modal scrims are **derived from the palette**, never hardcoded, so a theme
swap carries them along. The same token block is defined at the top of **both** apps'
stylesheets — `imacals-dashboard/src/style.css` and `imacals-web/src/style.css`. They are one
contract: change a value in one and change it in the other, or the storefront and the back office
drift apart.

| Token | Derivation | Use |
|---|---|---|
| `--color-on-primary` | palette `on-primary` | Text/icons on a Primary- or Tertiary-filled surface |
| `--color-border` | `color-mix(secondary 28%, transparent)` | Input borders, card and section hairlines |
| `--color-divider` | `color-mix(secondary 16%, transparent)` | Table row separators |
| `--color-overlay` | `color-mix(primary 35%, transparent)` | Modal backdrop |

## Light and dark

Both palettes ship. `:root` carries the light set ("Heritage": paper neutrals, `#B8422E` tertiary);
`:root[data-theme="dark"]` carries the Graphite set specified in this file's frontmatter. Only the
colours differ — type, spacing and radii are shared, and the derived tokens above recompute from the
new values, so a component built from tokens needs no dark-specific rules.

`useTheme()` (`src/composables/useTheme.ts`, mirrored in both apps) owns the switch: it stores the
choice under `localStorage.theme`, defaults to the OS `prefers-color-scheme` on a first visit, and
`initTheme()` runs in `main.ts` before mount so a dark session never flashes light. In the dashboard
the toggle lives in the account menu at the right of the top nav; in the storefront it sits in the
site header.

> **Caveat:** the token-driven views switch cleanly — every view in `imacals-web`, plus
> `AuthView.vue`, `UserProfileView.vue` and `AppTopNav.vue` in the dashboard. `UsersAllView.vue`
> still hardcodes light-palette hexes for borders and hovers, so it reads as light-on-dark until it
> gets a token pass.

## Do's and Don'ts

- **Do** use Tertiary for exactly one action per screen.
- **Do** let Neutral carry the composition — negative space is a feature.
- **Don't** introduce gradients. This system is flat on purpose.
- **Don't** mix Tertiary with alternate accents; the single-accent rule is load-bearing.
