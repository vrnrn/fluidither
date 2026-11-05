Obel Mono 1.001
===============

A monospaced typeface for terminals, code and agents, from Typographic Systems.
Every build, each roman and italic, weights 100-900, in every format.

ObelMono/       Obel Mono. Masking (ss01), Handwritten (ss02, or cv02-cv09 one letter at a time), the dotted zero (cv01) and the alternates cv10-cv14 are opt-in.

ObelMonoMasked/ Obel Mono Masked. Secret masking is on by default. A key with a known prefix (ghp_, sk-ant-, AKIA ...) keeps four characters after the prefix and shows the rest as mask dots; a value after a name that says secret (GITHUB_TOKEN=, "password": ") keeps its first four characters, then mask dots, in Latin or Cyrillic letters (in a terminal; a browser shapes Cyrillic apart from the name before it, so there only a Latin value is masked). It is a visual veil, not security: copying the text still gives the real characters.

In each build's folder:
  ttf/variable/  the variable fonts, roman and italic - install these on a desktop
  ttf/static/    every named weight as a TTF, for apps without variable fonts
  otf/           every named weight as an OTF (CFF)
  woff2/         the variable WOFF2 for the web, and obel-mono.css to self-host them

Install one build's ttf/variable/ fonts (or both builds: their family names differ)
  macOS     double-click them
  Linux     copy them to ~/.local/share/fonts/obel-mono/ and run fc-cache -f
  Windows   right-click them, Install
  Or, on macOS and Linux, install both builds with one line:
    curl -fsSL https://typographic.systems/install.sh | sh

Web
  Host the WOFF2 files and obel-mono.css yourself (they're in the WOFF2 download), then:
  font-family: "Obel Mono" or "Obel Mono Masked", monospace;

License: SIL Open Font License 1.1 (OFL.txt), no Reserved Font Name.
Authors: AUTHORS.txt.  More: https://typographic.systems/obel-mono
