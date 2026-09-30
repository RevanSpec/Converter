import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import { invoke } from "@tauri-apps/api/core";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import { LatestRequest } from "./latest";
import { morseSchedule } from "./morse";
import { charCount, formatMegabytes, utf8ByteLength } from "./text";

// Interface des options de conversion
interface ConvertOptions {
  hex_separator?: string;
  hex_uppercase?: boolean;
  binary_spaced?: boolean;
  base64_url_safe?: boolean;
  caesar_shift?: number;
  punycode_prefix?: boolean;
  morse_unknown?: "error" | "transliterate" | "ignore";
}

interface ConvertResult {
  output: string;
  input_chars: number;
  input_bytes: number;
  output_chars: number;
  output_bytes: number;
}

/** Au-delà, la conversion peut ralentir l'interface : un avertissement s'affiche. */
const WARN_BYTES = 5_000_000;
/** Au-delà, la saisie directe est refusée (le mode fichier arrivera plus tard). */
const MAX_BYTES = 50_000_000;
/** Durée d'un point Morse, en secondes. */
const MORSE_UNIT = 0.07;
/** Fréquence des bips Morse, en hertz. */
const MORSE_FREQUENCY = 650;

// Textes clairs d'exemple ; en décodage, l'exemple est leur encodage.
const SAMPLES: Record<string, string> = {
  hex: "Bienvenue sur Glass Converter ! Un encodeur ultra-rapide en Rust 🦀",
  binary: "Tauri + Rust = Performance & Élégance",
  base64: "Sécurité & Confidentialité : aucun historique conservé.",
  base32: "RFC 4648 Base32 Encoding",
  morse: "SOS HELLO WORLD 2026",
  url: "https://example.com/fr/docs/?search=glassmorphism&speed=fast#modern",
  caesar: "Ce message est chiffré avec l'algorithme historique de César !",
  html: "<span class=\"glass-pill\">Verre dépoli & reflets luminescents</span>",
  asciidec: "Code ASCII Décimal",
  asciioct: "Octal 8-bits",
  reverse: "Épuré, moderne et fenêtré",
  punycode: "café-crème.fr",
};

interface MorsePlayback {
  oscillators: OscillatorNode[];
  output: GainNode;
  endTimer: number;
}

// État de l'application (purement en mémoire, aucun historique sauvegardé)
class ConverterApp {
  private currentFormat = "hex";
  private toEncoded = true; // true: Clair -> Encodé ; false: Encodé -> Clair
  private options: ConvertOptions = {
    hex_separator: " ",
    hex_uppercase: false,
    binary_spaced: true,
    base64_url_safe: false,
    caesar_shift: 13,
    punycode_prefix: true,
    morse_unknown: "transliterate",
  };

  private debounceTimer: number | null = null;
  private conversions = new LatestRequest();
  private audioCtx: AudioContext | null = null;
  private morsePlayback: MorsePlayback | null = null;

  // Éléments DOM
  private sourceInput = document.getElementById("source-input") as HTMLTextAreaElement;
  private targetOutput = document.getElementById("target-output") as HTMLTextAreaElement;
  private sourceTitle = document.getElementById("source-title") as HTMLElement;
  private targetTitle = document.getElementById("target-title") as HTMLElement;
  private sourceCharCount = document.getElementById("source-char-count") as HTMLElement;
  private sourceByteCount = document.getElementById("source-byte-count") as HTMLElement;
  private targetCharCount = document.getElementById("target-char-count") as HTMLElement;
  private targetByteCount = document.getElementById("target-byte-count") as HTMLElement;
  private noticeBanner = document.getElementById("error-banner") as HTMLElement;
  private noticeText = document.getElementById("error-text") as HTMLElement;
  private swapBtn = document.getElementById("btn-swap-direction") as HTMLButtonElement;
  private copyBtn = document.getElementById("btn-copy-target") as HTMLButtonElement;
  private copyBtnText = document.getElementById("copy-btn-text") as HTMLElement;
  private toast = document.getElementById("toast") as HTMLElement;
  private toastText = document.getElementById("toast-text") as HTMLElement;
  private statusIndicator = document.getElementById("conversion-status") as HTMLElement;
  private morseSoundLabel = document.getElementById("morse-sound-label") as HTMLElement;

  private formatLabels: Record<string, string> = {
    hex: "Hexadécimal",
    binary: "Binaire",
    base64: "Base64",
    base32: "Base32",
    morse: "Code Morse",
    url: "URL Encode",
    caesar: "ROT13 / César",
    html: "HTML Entities",
    asciidec: "ASCII Décimal",
    asciioct: "ASCII Octal",
    reverse: "Inversion de texte",
    punycode: "Punycode (IDN)",
  };

  constructor() {
    this.initEventListeners();
    this.updateTitles();
    this.updateOptionsVisibility();
  }

  private initEventListeners() {
    // Saisie en temps réel
    this.sourceInput.addEventListener("input", () => {
      this.scheduleConversion();
    });

    // Inversion de sens
    this.swapBtn.addEventListener("click", () => {
      this.swapDirection();
    });

    // Sélection d'onglets de format
    const formatTabs = document.querySelectorAll<HTMLButtonElement>(".format-tab");
    formatTabs.forEach((tab) => {
      tab.addEventListener("click", () => {
        const fmt = tab.getAttribute("data-format");
        if (fmt && fmt !== this.currentFormat) {
          formatTabs.forEach((t) => t.classList.remove("active"));
          tab.classList.add("active");
          this.currentFormat = fmt;
          this.updateTitles();
          this.updateOptionsVisibility();
          this.scheduleConversion();
        }
      });
    });

    // Options Hexadécimal (séparateurs)
    this.bindSegments("[data-hex-sep]", "data-hex-sep", (value) => {
      this.options.hex_separator = value;
    });

    // Option Hexadécimal (Majuscules)
    const hexUpperCheckbox = document.getElementById("hex-uppercase") as HTMLInputElement;
    if (hexUpperCheckbox) {
      hexUpperCheckbox.addEventListener("change", (e) => {
        this.options.hex_uppercase = (e.target as HTMLInputElement).checked;
        this.scheduleConversion();
      });
    }

    // Options Binaire
    this.bindSegments("[data-bin-spaced]", "data-bin-spaced", (value) => {
      this.options.binary_spaced = value === "true";
    });

    // Options Base64
    this.bindSegments("[data-b64-url]", "data-b64-url", (value) => {
      this.options.base64_url_safe = value === "true";
    });

    // Options César / ROT13
    const caesarSlider = document.getElementById("caesar-shift-slider") as HTMLInputElement;
    const caesarValBadge = document.getElementById("caesar-shift-val") as HTMLElement;
    const caesarResetBtn = document.getElementById("btn-caesar-reset") as HTMLButtonElement;

    if (caesarSlider && caesarValBadge) {
      caesarSlider.addEventListener("input", () => {
        const val = parseInt(caesarSlider.value, 10);
        this.options.caesar_shift = val;
        caesarValBadge.textContent = val === 13 ? `+${val} (ROT13)` : `+${val}`;
        this.scheduleConversion();
      });
    }

    if (caesarResetBtn && caesarSlider && caesarValBadge) {
      caesarResetBtn.addEventListener("click", () => {
        caesarSlider.value = "13";
        this.options.caesar_shift = 13;
        caesarValBadge.textContent = "+13 (ROT13)";
        this.scheduleConversion();
      });
    }

    // Options Punycode
    this.bindSegments("[data-puny-prefix]", "data-puny-prefix", (value) => {
      this.options.punycode_prefix = value === "true";
    });

    // Options Morse (caractères sans code)
    this.bindSegments("[data-morse-unknown]", "data-morse-unknown", (value) => {
      this.options.morse_unknown = value as ConvertOptions["morse_unknown"];
    });

    // Bouton Morse Audio
    const morseSoundBtn = document.getElementById("btn-play-morse");
    if (morseSoundBtn) {
      morseSoundBtn.addEventListener("click", () => {
        this.toggleMorseAudio();
      });
    }

    // Copier
    this.copyBtn.addEventListener("click", () => {
      this.copyToClipboard();
    });

    // Coller
    const pasteBtn = document.getElementById("btn-paste-source");
    if (pasteBtn) {
      pasteBtn.addEventListener("click", () => {
        this.pasteFromClipboard();
      });
    }

    // Vider la source
    const clearSourceBtn = document.getElementById("btn-clear-source");
    if (clearSourceBtn) {
      clearSourceBtn.addEventListener("click", () => {
        this.sourceInput.value = "";
        this.scheduleConversion();
        this.sourceInput.focus();
      });
    }

    // Tout effacer
    const clearAllBtn = document.getElementById("btn-clear-all");
    if (clearAllBtn) {
      clearAllBtn.addEventListener("click", () => {
        this.conversions.invalidate();
        this.sourceInput.value = "";
        this.targetOutput.value = "";
        this.hideNotice();
        this.setErrorState(false);
        this.updateStats(0, 0, 0, 0);
        this.updateStatus("Prêt");
        this.showToast("Zone de conversion vidée");
        this.sourceInput.focus();
      });
    }

    // Exemple
    const exampleBtn = document.getElementById("btn-example");
    if (exampleBtn) {
      exampleBtn.addEventListener("click", () => {
        this.loadSample();
      });
    }
  }

  /** Contrôle segmenté : un seul segment actif, sa valeur est transmise à `apply`. */
  private bindSegments(selector: string, attribute: string, apply: (value: string) => void) {
    const segments = document.querySelectorAll<HTMLButtonElement>(selector);
    segments.forEach((seg) => {
      seg.addEventListener("click", () => {
        segments.forEach((s) => s.classList.remove("active"));
        seg.classList.add("active");
        apply(seg.getAttribute(attribute) ?? "");
        this.scheduleConversion();
      });
    });
  }

  private updateTitles() {
    const fmtName = this.formatLabels[this.currentFormat] || this.currentFormat;
    if (this.toEncoded) {
      this.sourceTitle.textContent = "Texte en clair";
      this.targetTitle.textContent = fmtName;
    } else {
      this.sourceTitle.textContent = fmtName;
      this.targetTitle.textContent = "Texte en clair";
    }
  }

  private updateOptionsVisibility() {
    const allGroups = document.querySelectorAll(".options-group");
    allGroups.forEach((g) => g.classList.add("hidden"));

    const specificGroup = document.getElementById(`opts-${this.currentFormat}`);
    if (specificGroup) {
      specificGroup.classList.remove("hidden");
    } else {
      const genericGroup = document.getElementById("opts-generic");
      if (genericGroup) genericGroup.classList.remove("hidden");
    }
  }

  private swapDirection() {
    this.toEncoded = !this.toEncoded;
    this.swapBtn.classList.toggle("rotated");

    // Échange du contenu
    const oldSource = this.sourceInput.value;
    const oldTarget = this.targetOutput.value;

    this.sourceInput.value = oldTarget;
    this.targetOutput.value = oldSource;

    this.updateTitles();
    this.scheduleConversion();
  }

  private scheduleConversion() {
    if (this.debounceTimer) {
      window.clearTimeout(this.debounceTimer);
    }
    this.debounceTimer = window.setTimeout(() => {
      this.performConversion();
    }, 20);
  }

  private async performConversion() {
    const input = this.sourceInput.value;
    const isLatest = this.conversions.begin();

    if (!input) {
      this.targetOutput.value = "";
      this.hideNotice();
      this.setErrorState(false);
      this.updateStats(0, 0, 0, 0);
      this.updateStatus("Prêt");
      return;
    }

    const inputBytes = utf8ByteLength(input);
    if (inputBytes > MAX_BYTES) {
      this.showConversionError(
        `Texte trop volumineux (${formatMegabytes(inputBytes)}, maximum ${formatMegabytes(MAX_BYTES)}) : le traitement de fichiers arrivera dans une prochaine version.`,
        input,
        inputBytes
      );
      return;
    }

    try {
      this.updateStatus("Conversion...");
      const result = await invoke<ConvertResult>("convert_text", {
        input,
        format: this.currentFormat,
        toEncoded: this.toEncoded,
        options: this.options,
      });
      // Une saisie plus récente a déjà relancé une conversion : ce résultat est périmé.
      if (!isLatest()) return;

      this.targetOutput.value = result.output;
      this.setErrorState(false);
      this.updateStats(
        result.input_chars,
        result.input_bytes,
        result.output_chars,
        result.output_bytes
      );
      if (inputBytes > WARN_BYTES) {
        this.showNotice(
          `Texte volumineux (${formatMegabytes(inputBytes)}) : la conversion peut prendre du temps.`,
          "warning"
        );
      } else {
        this.hideNotice();
      }
      this.updateStatus("Converti");
    } catch (err: unknown) {
      if (!isLatest()) return;
      this.showConversionError(typeof err === "string" ? err : String(err), input, inputBytes);
    }
  }

  /** En erreur, l'ancien résultat disparaît : il ne peut plus être copié ni réinjecté. */
  private showConversionError(message: string, input: string, inputBytes: number) {
    this.targetOutput.value = "";
    this.updateStats(charCount(input), inputBytes, 0, 0);
    this.showNotice(message, "error");
    this.setErrorState(true);
    this.updateStatus("Erreur");
  }

  private setErrorState(hasError: boolean) {
    this.copyBtn.disabled = hasError;
    this.swapBtn.disabled = hasError;
  }

  private updateStats(inChars: number, inBytes: number, outChars: number, outBytes: number) {
    this.sourceCharCount.textContent = inChars.toString();
    this.sourceByteCount.textContent = inBytes.toString();
    this.targetCharCount.textContent = outChars.toString();
    this.targetByteCount.textContent = outBytes.toString();
  }

  private updateStatus(text: string) {
    if (this.statusIndicator) {
      this.statusIndicator.innerHTML = `<span class="status-pulse"></span> ${text}`;
    }
  }

  private showNotice(msg: string, level: "error" | "warning") {
    this.noticeText.textContent = msg;
    this.noticeBanner.classList.toggle("warning", level === "warning");
    this.noticeBanner.classList.remove("hidden");
  }

  private hideNotice() {
    this.noticeBanner.classList.add("hidden");
  }

  private async copyToClipboard() {
    const text = this.targetOutput.value;
    if (!text) return;

    try {
      await writeText(text);
      this.copyBtn.classList.add("btn-copied");
      this.copyBtnText.textContent = "Copié !";
      this.showToast("Résultat copié dans le presse-papier !");

      setTimeout(() => {
        this.copyBtn.classList.remove("btn-copied");
        this.copyBtnText.textContent = "Copier";
      }, 1600);
    } catch (err) {
      this.showToast(`Copie impossible : ${String(err)}`, true);
    }
  }

  private async pasteFromClipboard() {
    try {
      const text = await readText();
      if (!text) {
        this.showToast("Le presse-papier est vide", true);
        return;
      }
      this.sourceInput.value = text;
      this.scheduleConversion();
      this.showToast("Texte collé depuis le presse-papier");
    } catch {
      this.showToast("Le presse-papier ne contient pas de texte lisible", true);
    }
  }

  private showToast(message: string, isError = false) {
    this.toastText.textContent = message;
    this.toast.classList.toggle("toast-error", isError);
    this.toast.classList.add("show");
    setTimeout(() => {
      this.toast.classList.remove("show");
    }, 2200);
  }

  private async loadSample() {
    const sample = SAMPLES[this.currentFormat] || "Exemple de texte moderne";

    if (this.toEncoded) {
      this.sourceInput.value = sample;
    } else {
      // En décodage, l'exemple est l'encodage du texte clair avec les options courantes :
      // il est donc toujours valide, quel que soit le décalage César choisi.
      try {
        const result = await invoke<ConvertResult>("convert_text", {
          input: sample,
          format: this.currentFormat,
          toEncoded: true,
          options: this.options,
        });
        this.sourceInput.value = result.output;
      } catch (err) {
        this.showToast(`Exemple indisponible : ${String(err)}`, true);
        return;
      }
    }

    this.scheduleConversion();
  }

  // Synthétiseur audio Web Audio API pour le code Morse
  private toggleMorseAudio() {
    if (this.morsePlayback) {
      this.stopMorseAudio();
      return;
    }

    const morseText = this.toEncoded ? this.targetOutput.value : this.sourceInput.value;
    const { beeps, totalUnits } = morseSchedule(morseText);
    if (beeps.length === 0) {
      this.showToast("Aucun code Morse à écouter", true);
      return;
    }

    this.audioCtx ??= new AudioContext();
    const ctx = this.audioCtx;
    const output = ctx.createGain();
    output.connect(ctx.destination);

    const origin = ctx.currentTime + 0.05;
    const oscillators = beeps.map((beep) =>
      this.playBeep(ctx, output, origin + beep.start * MORSE_UNIT, beep.duration * MORSE_UNIT)
    );
    const endTimer = window.setTimeout(
      () => this.stopMorseAudio(),
      (0.05 + totalUnits * MORSE_UNIT) * 1000 + 100
    );

    this.morsePlayback = { oscillators, output, endTimer };
    this.morseSoundLabel.textContent = "Arrêter l'audio";
  }

  private playBeep(
    ctx: AudioContext,
    output: AudioNode,
    startTime: number,
    duration: number
  ): OscillatorNode {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = "sine";
    osc.frequency.setValueAtTime(MORSE_FREQUENCY, startTime);

    // Enveloppe d'attaque et d'extinction douce (éviter les clics audio)
    gain.gain.setValueAtTime(0, startTime);
    gain.gain.linearRampToValueAtTime(0.2, startTime + 0.005);
    gain.gain.setValueAtTime(0.2, startTime + duration - 0.005);
    gain.gain.linearRampToValueAtTime(0, startTime + duration);

    osc.connect(gain);
    gain.connect(output);

    osc.start(startTime);
    osc.stop(startTime + duration);
    return osc;
  }

  /** Coupe le son tout de suite, y compris les bips déjà programmés. */
  private stopMorseAudio() {
    const playback = this.morsePlayback;
    if (!playback) return;

    this.morsePlayback = null;
    window.clearTimeout(playback.endTimer);
    playback.output.disconnect();
    for (const osc of playback.oscillators) {
      try {
        osc.stop();
      } catch {
        // Bip déjà terminé
      }
    }
    this.morseSoundLabel.textContent = "Écouter le Morse (Bips audio)";
  }
}

// Initialisation dès que le DOM est chargé
window.addEventListener("DOMContentLoaded", () => {
  new ConverterApp();
});
