import { morseSchedule } from "./morse";

/** Durée d'un point Morse, en secondes. */
const UNIT = 0.07;
/** Fréquence des bips, en hertz. */
const FREQUENCY = 650;

interface Playback {
  oscillators: OscillatorNode[];
  output: GainNode;
  endTimer: number;
}

/** Joue un texte Morse en bips (Web Audio) ; `stop` coupe aussi les bips déjà programmés. */
export class MorsePlayer {
  private context: AudioContext | null = null;
  private playback: Playback | null = null;
  private readonly onEnd: () => void;

  /** `onEnd` est appelé quand la lecture se termine ou est arrêtée. */
  constructor(onEnd: () => void) {
    this.onEnd = onEnd;
  }

  /** Renvoie `false` s'il n'y a aucun signal à jouer. */
  play(morse: string): boolean {
    this.stop();
    const { beeps, totalUnits } = morseSchedule(morse);
    if (beeps.length === 0) return false;

    this.context ??= new AudioContext();
    const context = this.context;
    const output = context.createGain();
    output.connect(context.destination);

    const origin = context.currentTime + 0.05;
    const oscillators = beeps.map((beep) =>
      beepAt(context, output, origin + beep.start * UNIT, beep.duration * UNIT)
    );
    const endTimer = window.setTimeout(
      () => this.stop(),
      (0.05 + totalUnits * UNIT) * 1000 + 100
    );
    this.playback = { oscillators, output, endTimer };
    return true;
  }

  stop(): void {
    const playback = this.playback;
    if (!playback) return;

    this.playback = null;
    window.clearTimeout(playback.endTimer);
    playback.output.disconnect();
    for (const oscillator of playback.oscillators) {
      try {
        oscillator.stop();
      } catch {
        // Bip déjà terminé
      }
    }
    this.onEnd();
  }
}

function beepAt(
  context: AudioContext,
  output: AudioNode,
  startTime: number,
  duration: number
): OscillatorNode {
  const oscillator = context.createOscillator();
  const gain = context.createGain();

  oscillator.type = "sine";
  oscillator.frequency.setValueAtTime(FREQUENCY, startTime);

  // Enveloppe d'attaque et d'extinction douce (éviter les clics audio)
  gain.gain.setValueAtTime(0, startTime);
  gain.gain.linearRampToValueAtTime(0.2, startTime + 0.005);
  gain.gain.setValueAtTime(0.2, startTime + duration - 0.005);
  gain.gain.linearRampToValueAtTime(0, startTime + duration);

  oscillator.connect(gain);
  gain.connect(output);
  oscillator.start(startTime);
  oscillator.stop(startTime + duration);
  return oscillator;
}
