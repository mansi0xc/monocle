# Monocle reference logo assets

This revision follows the feather-and-lens symbol supplied by the project owner. The surrounding reference card, AI badge, and “Feathered Lens” text are excluded. The product name stays Monocle. The symbol was cleaned/recoloured with the built-in image tool, then converted to native SVG outlines for consistent rendering and exact palette colours.

The palette is vintage burgundy `#AA2D3C`, deep navy `#28324F`, and cream `#F2E6D8`. The feather uses burgundy; lens, handle, reflection and wordmark use navy. Transparent negative space allows the cream page surface to show through.

- `monocle-mark.svg`: two-colour mark with transparent counters.
- `monocle-mark-inverse.svg`: cream mark for navy or burgundy backgrounds.
- `monocle-logo.svg`: horizontal logo with outlined Sora wordmark.
- `monocle-logo-inverse.svg`: cream horizontal logo.
- `monocle-mark.png`: 1024px transparent mark.
- `monocle-logo.png`: 1068×288 transparent horizontal logo.

The supplied reference was found online; a source URL was not provided. This documentation does not attribute the original symbol's authorship to Monocle. The wordmark uses the existing Sora font at weight 550, with its SIL Open Font License retained in `../../src/fonts/Sora-OFL.txt`.

The live component in `../../src/Brand.tsx` uses inline vector paths and local CSS palette tokens, so the logo adds no remote request to verification or offline replay. It remains decorative beside accessible link text. Retain the mark's proportions, feather gaps and open lens rim; do not fill its transparent counters or recolour isolated pieces arbitrarily.
