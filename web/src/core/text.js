// Text effects as pure functions. gpui/src/text_effects.rs is the same code.

const GLYPHS = '#%&*+=/<>?';

/**
 * `text` part way through a scramble: letters before `progress` (0 to 1) are settled
 * and the rest are stand-ins picked by `seed`. Spaces stay, so words keep their shape.
 */
export function scrambled(text, progress, seed) {
  const letters = [...text];
  const settled = Math.floor(letters.length * progress);
  return letters
    .map((letter, index) => {
      if (index < settled || letter === ' ') return letter;
      return GLYPHS[(Math.imul(index + 7, seed + 13) >>> 0) % GLYPHS.length];
    })
    .join('');
}
