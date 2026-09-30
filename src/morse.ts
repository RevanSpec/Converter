/** Bip Morse, en unités de temps (1 unité = la durée d'un point). */
export interface Beep {
  start: number;
  duration: number;
}

/**
 * Programme les bips d'un texte Morse selon les durées ITU : point 1, trait 3,
 * 1 entre deux signes, 3 entre deux lettres, 7 entre deux mots.
 * Les mots sont séparés par « / » ou un retour à la ligne, les lettres par des blancs.
 */
export function morseSchedule(morse: string): { beeps: Beep[]; totalUnits: number } {
  const beeps: Beep[] = [];
  let time = 0;
  const words = morse
    .split(/[/\n]/)
    .map((word) => word.trim())
    .filter((word) => word.length > 0);

  words.forEach((word, wordIndex) => {
    if (wordIndex > 0) time += 7;
    word.split(/\s+/).forEach((letter, letterIndex) => {
      if (letterIndex > 0) time += 3;
      let firstSymbol = true;
      for (const symbol of letter) {
        const duration = symbol === "." ? 1 : symbol === "-" ? 3 : 0;
        if (duration === 0) continue;
        if (!firstSymbol) time += 1;
        beeps.push({ start: time, duration });
        time += duration;
        firstSymbol = false;
      }
    });
  });

  return { beeps, totalUnits: time };
}
