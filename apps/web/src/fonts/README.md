# Local typography assets

Newsreader is the display face for landing headlines, section titles and report verdicts. Schibsted Grotesk is used for the wordmark, interface text and body copy. Red Hat Mono is used for digests, byte views, diagnostic codes, and exact integers. All three variable, Latin-subset WOFF2 files are bundled by Vite and served locally; there are no Google Fonts or other third-party font requests at runtime. Uncovered glyphs use the system fallback.

Downloaded from the official Google Fonts CDN on 2026-10-16:

- Schibsted Grotesk (`400–900`): https://fonts.gstatic.com/s/schibstedgrotesk/v7/Jqz55SSPQuCQF3t8uOwiUL-taUTtap9GayojdSFO.woff2
- Newsreader (upright): https://fonts.gstatic.com/s/newsreader/v26/cY9AfjOCX1hbuyalUrK4397yjIJFJpc.woff2, then instanced with fontTools 4.60.2 (`varLib.instancer wght=300:500 opsz=24:72`, re-saved as WOFF2 with brotli 1.1.0) to cut it from 132 kB to 86 kB. The OFL permits modified versions; Newsreader declares no Reserved Font Name.
- Red Hat Mono (`300–700`): https://fonts.gstatic.com/s/redhatmono/v16/jVyN7nDnA2uf2zVvFAhhzEsUWOxS3A_geQ.woff2

Catalogs: https://fonts.google.com/specimen/Newsreader, https://fonts.google.com/specimen/Schibsted+Grotesk and https://fonts.google.com/specimen/Red+Hat+Mono.

The accompanying SIL Open Font License files are copied from the corresponding `ofl/newsreader`, `ofl/schibstedgrotesk` and `ofl/redhatmono` directories in https://github.com/google/fonts. Retain all three license files when redistributing the fonts.
