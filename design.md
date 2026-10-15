# Frontend design guide

## Direction

Build the interface as a dark editorial security system: precise, technical, slightly futuristic, and restrained. The reference combines a serious information architecture with expressive visual surfaces. The page should feel like a security lab or protocol control room rather than a conventional SaaS landing page.

Use the reference as a visual language, not as a source for its brand, copy, logos, or exact artwork. Replace those elements with Slotbook content and product concepts.

The design should communicate:

- trust and technical competence;
- clear monitoring and verification;
- controlled access to complex information;
- a premium research-oriented product experience.

## Visual principles

1. Use a near-black canvas with pale technical text.
2. Establish hierarchy through scale, spacing, and panel placement rather than heavy shadows.
3. Combine rigid geometry with a small number of soft, luminous elements.
4. Treat panels as modular “cells” that can be rearranged into editorial compositions.
5. Keep controls sparse and deliberate. Every button should look like an instrument or labelled control.
6. Use bright red and violet only as directional accents, active states, and focal gradients.

## Color system

Use CSS variables so the visual system can be adjusted centrally.

```css
:root {
  --color-ink: #08090b;
  --color-ink-soft: #111318;
  --color-panel: #15171c;
  --color-panel-raised: #1b1e25;
  --color-white: #f4f5f1;
  --color-muted: #9a9da5;
  --color-faint: #5d626b;
  --color-line: rgba(244, 245, 241, 0.22);
  --color-line-strong: rgba(244, 245, 241, 0.48);
  --color-red: #f04455;
  --color-red-hot: #ff5361;
  --color-violet: #7352cf;
  --color-blue: #4d63ff;
  --color-cyan: #80d7df;
  --color-success: #92d6aa;
  --color-warning: #e8b768;
}
```

### Usage

- `--color-ink` is the page background.
- `--color-panel` and `--color-panel-raised` separate cards from the canvas without making them look glossy.
- `--color-white` is for display headings and primary labels.
- `--color-muted` is for descriptions and metadata.
- `--color-line` creates the thin technical borders visible throughout the reference.
- `--color-red` is the main product accent and hero gradient anchor.
- `--color-violet` and `--color-blue` support depth in gradients and active states.
- Green, amber, and red status colors should be reserved for system state, never used as decoration.

Avoid pure white backgrounds, saturated rainbow palettes, large drop shadows, and gradients behind every component.

## Typography

The reference uses a wide geometric display face for headlines and a compact neutral sans for utility text.

### Font roles

- **Display:** wide, squared, futuristic sans with distinctive numerals. Use for hero headings, section titles, cell numbers, and large metrics.
- **Body:** neutral grotesk sans with high legibility. Use for paragraphs, navigation, labels, tables, and controls.
- **Mono:** technical monospace for addresses, slots, transaction signatures, timestamps, and code-like metadata.

Recommended fallback stacks:

```css
--font-display: "Space Grotesk", "Arial Narrow", sans-serif;
--font-body: Inter, ui-sans-serif, system-ui, sans-serif;
--font-mono: "IBM Plex Mono", "SFMono-Regular", Consolas, monospace;
```

### Type scale

Use a fluid scale, with clamp values adjusted to the application shell:

```css
--text-xs: 0.68rem;
--text-sm: 0.78rem;
--text-md: 0.95rem;
--text-lg: clamp(1.1rem, 1.4vw, 1.45rem);
--text-xl: clamp(1.8rem, 3vw, 3.25rem);
--text-hero: clamp(3rem, 8vw, 8.5rem);
```

Display headings should be uppercase or title case with tight line height around `0.86–0.98`. Body text should use `1.45–1.65` line height. Utility labels can be uppercase with `0.08–0.16em` letter spacing.

Do not use the display face for long paragraphs or dense data tables.

## Layout system

Use a 12-column editorial grid on desktop.

```css
.page-shell {
  width: min(100% - 2rem, 1440px);
  margin-inline: auto;
}

.editorial-grid {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr));
  gap: 1px;
}
```

The one-pixel gaps are intentional when panels sit beside one another. They create the technical frame visible in the reference.

### Spacing

Use a compact, consistent scale:

```css
--space-1: 0.25rem;
--space-2: 0.5rem;
--space-3: 0.75rem;
--space-4: 1rem;
--space-6: 1.5rem;
--space-8: 2rem;
--space-12: 3rem;
--space-16: 4rem;
--space-24: 6rem;
```

Use large vertical space between major sections, but keep labels, borders, and metadata close to their associated content.

### Responsive behavior

- At `1200px` and above, use the full 12-column composition and overlapping feature panels.
- Between `768px` and `1199px`, reduce the grid to 8 columns and remove decorative overlaps that make reading difficult.
- Below `768px`, use a single column. Stack panels in the order of importance, preserve the grid lines, and let the hero artwork become a contained background or smaller inline visual.
- Navigation should collapse into a compact menu while keeping the primary action visible.
- Never allow large display headings to overflow horizontally. Use `text-wrap: balance` and controlled line breaks.

## Navigation

The header is a thin bordered strip over the dark canvas.

- Keep the brand mark at the far left.
- Place primary links in a compact horizontal group.
- Use small chevrons for menus such as “Services” or “Explore”.
- Keep the main action on the right in a clipped control.
- Separate the header from the hero with a one-pixel border, not a filled bar.
- On scroll, the header may become sticky with a translucent `backdrop-filter: blur(16px)` and a slightly darker background.

The primary action should be short and explicit: `View demo`, `Open dashboard`, `Get started`, or `Connect wallet` depending on the screen.

## Buttons and controls

The signature control is a rectangular button with one or two corners cut at an angle.

Implement the shape with a pseudo-element or `clip-path: polygon(...)`. Keep the clickable element rectangular in layout so keyboard focus and hit testing remain reliable.

```css
.control-button {
  min-height: 2.75rem;
  padding: 0.65rem 1.35rem;
  border: 1px solid var(--color-line-strong);
  color: var(--color-ink);
  background: var(--color-white);
  clip-path: polygon(10% 0, 100% 0, 90% 100%, 0 100%);
  font: 600 var(--text-sm) / 1 var(--font-body);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
```

Provide a visible focus ring outside the clipped shape. Hover states should shift the accent color or translate the control by `1–2px`; avoid large scale animations.

## Hero composition

The hero is the strongest visual moment.

Use a large panel with:

- a red-to-violet radial gradient concentrated toward the left and center;
- a dark fade toward the lower and right edges;
- a short eyebrow such as `SLOTBOOK / DISTRIBUTION RECORDS`;
- a large, multi-line display heading;
- one primary action near the heading;
- a translucent geometric illustration on the right or upper right;
- subtle grid lines and one or two thin connection lines.

The artwork should support the product concept: blocks, account nodes, a merkle tree, a slot timeline, a vault, or a distribution graph. Keep it abstract and low contrast so text remains dominant.

The hero should feel layered. Use absolutely positioned decorative elements inside an `overflow: hidden` container, but keep the content in normal flow for accessibility and responsive behavior.

## Panels and cells

The reference repeatedly uses framed content cells.

Each cell should have:

- a thin border;
- a small index such as `01`, `02`, or `[02]`;
- a short heading;
- a concise explanatory paragraph;
- an icon or abstract visual;
- an optional corner marker or small circular indicator.

```css
.cell {
  position: relative;
  min-height: 15rem;
  padding: var(--space-6);
  border: 1px solid var(--color-line);
  background: var(--color-panel);
  overflow: hidden;
}

.cell::after {
  position: absolute;
  right: 0.75rem;
  bottom: 0.75rem;
  width: 0.45rem;
  height: 0.45rem;
  border: 1px solid var(--color-faint);
  border-radius: 50%;
  content: "";
}
```

Use two to four cells per row. Vary their width and height slightly to create an editorial rhythm, but keep the underlying grid aligned.

For Slotbook, useful cell types include:

- current record slot;
- distribution status;
- merkle root status;
- holder count;
- frozen or pending accounts;
- challenge window;
- claim vault balance;
- replay verification status.

## Data and protocol surfaces

The application should use the same visual language for operational data as for marketing content.

- Use monospace text for public keys, slots, signatures, roots, and timestamps.
- Truncate long values visually but provide a copy action and accessible full value.
- Use small uppercase labels above values.
- Present status with a dot, short label, and optional timestamp.
- Use red for blocked or frozen states, green for verified or claimable states, amber for pending or challenge states.
- Use thin horizontal rules to separate event rows.
- Highlight the active record slot with a small red or violet bar rather than a large filled card.

Example status labels:

```text
REPLAY VERIFIED
PENDING CHALLENGE
FROZEN ACCOUNT
CLAIM OPEN
RECORD SLOT 509633005
```

## Imagery and 3D graphics

The reference uses translucent cubes and soft-lit geometric forms.

For product graphics:

- prefer low-poly or glass-like primitives;
- use pale lavender, white, and violet surfaces;
- add soft bloom sparingly;
- keep edges and connection lines crisp;
- use depth through opacity and blur rather than photorealistic texture;
- animate slowly with a small rotation or parallax response.

The graphics should remain secondary to data. Provide a reduced-motion mode that removes rotation and parallax while preserving the image.

## Background treatments

Use a small set of repeatable textures:

- a one-pixel grid with low opacity;
- a faint crosshair or corner registration mark;
- a blurred radial glow behind a hero object;
- subtle vertical or horizontal rules dividing editorial regions;
- sparse code-like or hexadecimal texture at very low opacity.

Textures must not interfere with contrast or text selection. Keep them decorative with `pointer-events: none` and `aria-hidden="true"`.

## Motion

Motion should communicate system activity, not decoration.

- Fade and translate sections into view over `400–700ms`.
- Use staggered reveals for cells with a delay of `40–80ms` per item.
- Animate status pulses only while a live operation is active.
- Use a slow `8–16s` movement for hero artwork.
- Use a short `150–220ms` transition for hover and focus states.
- Respect `prefers-reduced-motion: reduce` by disabling parallax, rotation, looping glows, and stagger delays.

Avoid scroll hijacking, constant flashing, and animations that alter layout.

## Page composition

A typical Slotbook page can follow this order:

1. Thin global header.
2. Hero statement with a live protocol or distribution visual.
3. Trust or system metrics row.
4. Editorial explanation of how a distribution moves from declaration to claim.
5. Four-cell feature grid for replay, merkle verification, challenge, and claims.
6. Live activity panel with transactions or holder state.
7. CTA panel with the next action.
8. Minimal footer with protocol links and network status.

Keep one strong focal point per viewport. If a dashboard is the primary surface, replace the marketing hero with a smaller command header and let the record state occupy the first panel.

## Accessibility

- Maintain WCAG AA contrast for all text and status indicators.
- Never communicate state through color alone; include text and, where useful, an icon.
- Keep all clipped controls keyboard accessible.
- Provide visible `:focus-visible` states.
- Use semantic headings in document order.
- Give decorative 3D art empty alt text or mark it `aria-hidden`.
- Ensure monospace addresses can be copied without relying on hover.
- Make tables and event lists readable at 200% zoom.
- Provide a reduced-motion mode.

## Implementation guidance

Create reusable primitives rather than styling each page independently:

- `PageShell`
- `Header`
- `ClippedButton`
- `HeroPanel`
- `SectionLabel`
- `MetricCell`
- `StatusBadge`
- `DataValue`
- `EventRow`
- `GridTexture`
- `ProtocolGraphic`

Keep content, data state, and decorative artwork separate. A component should be able to render loading, empty, success, pending, frozen, and error states without changing the overall layout.

Prefer CSS for borders, grids, gradients, clipping, and simple glow effects. Use canvas or WebGL only for a visual that cannot be expressed with layered DOM elements. Lazy-load heavy 3D assets and provide a static fallback.

## Quality checks

Before considering a screen complete, check:

- the hero heading remains readable over the artwork;
- every panel aligns to the editorial grid;
- borders remain visible on dark displays;
- mobile layout does not depend on overlapping artwork;
- status colors have text labels;
- controls have keyboard focus states;
- reduced motion removes continuous animation;
- long wallet addresses and transaction signatures do not break the layout;
- empty and error states use the same visual system as successful states.
