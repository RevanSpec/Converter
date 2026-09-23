import { invoke } from "@tauri-apps/api/core";

// Interface des options de conversion
interface ConvertOptions {
  hex_separator?: string;
  hex_uppercase?: boolean;
  binary_spaced?: boolean;
  base64_url_safe?: boolean;
  caesar_shift?: number;
  punycode_prefix?: boolean;
}

interface ConvertResult {
  output: string;
  input_chars: number;
  input_bytes: number;
  output_chars: number;
  output_bytes: number;
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
  };

  private debounceTimer: number | null = null;
  private isMorsePlaying = false;
  private audioCtx: AudioContext | null = null;

  // Éléments DOM
  private sourceInput = document.getElementById("source-input") as HTMLTextAreaElement;
  private targetOutput = document.getElementById("target-output") as HTMLTextAreaElement;
  private sourceTitle = document.getElementById("source-title") as HTMLElement;
  private targetTitle = document.getElementById("target-title") as HTMLElement;
  private sourceCharCount = document.getElementById("source-char-count") as HTMLElement;
  private sourceByteCount = document.getElementById("source-byte-count") as HTMLElement;
  private targetCharCount = document.getElementById("target-char-count") as HTMLElement;
  private targetByteCount = document.getElementById("target-byte-count") as HTMLElement;
  private errorBanner = document.getElementById("error-banner") as HTMLElement;
  private errorText = document.getElementById("error-text") as HTMLElement;
  private swapBtn = document.getElementById("btn-swap-direction") as HTMLButtonElement;
  private copyBtn = document.getElementById("btn-copy-target") as HTMLButtonElement;
  private copyBtnText = document.getElementById("copy-btn-text") as HTMLElement;
  private toast = document.getElementById("toast") as HTMLElement;
  private toastText = document.getElementById("toast-text") as HTMLElement;
  private statusIndicator = document.getElementById("conversion-status") as HTMLElement;

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
    const hexSegments = document.querySelectorAll<HTMLButtonElement>("[data-hex-sep]");
    hexSegments.forEach((seg) => {
      seg.addEventListener("click", () => {
        hexSegments.forEach((s) => s.classList.remove("active"));
        seg.classList.add("active");
        this.options.hex_separator = seg.getAttribute("data-hex-sep") || " ";
        this.scheduleConversion();
      });
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
    const binSegments = document.querySelectorAll<HTMLButtonElement>("[data-bin-spaced]");
    binSegments.forEach((seg) => {
      seg.addEventListener("click", () => {
        binSegments.forEach((s) => s.classList.remove("active"));
        seg.classList.add("active");
        this.options.binary_spaced = seg.getAttribute("data-bin-spaced") === "true";
        this.scheduleConversion();
      });
    });

    // Options Base64
    const b64Segments = document.querySelectorAll<HTMLButtonElement>("[data-b64-url]");
    b64Segments.forEach((seg) => {
      seg.addEventListener("click", () => {
        b64Segments.forEach((s) => s.classList.remove("active"));
        seg.classList.add("active");
        this.options.base64_url_safe = seg.getAttribute("data-b64-url") === "true";
        this.scheduleConversion();
      });
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
    const punySegments = document.querySelectorAll<HTMLButtonElement>("[data-puny-prefix]");
    punySegments.forEach((seg) => {
      seg.addEventListener("click", () => {
        punySegments.forEach((s) => s.classList.remove("active"));
        seg.classList.add("active");
        this.options.punycode_prefix = seg.getAttribute("data-puny-prefix") === "true";
        this.scheduleConversion();
      });
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
      pasteBtn.addEventListener("click", async () => {
        try {
          const text = await navigator.clipboard.readText();
          if (text) {
            this.sourceInput.value = text;
            this.scheduleConversion();
            this.showToast("Texte collé depuis le presse-papier");
          }
        } catch (e) {
          console.error("Impossible de lire le presse-papier:", e);
        }
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
        this.sourceInput.value = "";
        this.targetOutput.value = "";
        this.hideError();
        this.updateStats(0, 0, 0, 0);
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

    if (!input) {
      this.targetOutput.value = "";
      this.hideError();
      this.updateStats(0, 0, 0, 0);
      this.updateStatus("Prêt");
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

      this.targetOutput.value = result.output;
      this.hideError();
      this.updateStats(
        result.input_chars,
        result.input_bytes,
        result.output_chars,
        result.output_bytes
      );
      this.updateStatus("Converti");
    } catch (err: unknown) {
      const errMsg = typeof err === "string" ? err : String(err);
      this.showError(errMsg);
      this.updateStatus("Erreur");
    }
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

  private showError(msg: string) {
    this.errorText.textContent = msg;
    this.errorBanner.classList.remove("hidden");
  }

  private hideError() {
    this.errorBanner.classList.add("hidden");
  }

  private async copyToClipboard() {
    const text = this.targetOutput.value;
    if (!text) return;

    try {
      await navigator.clipboard.writeText(text);
      this.copyBtn.classList.add("btn-copied");
      this.copyBtnText.textContent = "Copié !";
      this.showToast("Résultat copié dans le presse-papier !");

      setTimeout(() => {
        this.copyBtn.classList.remove("btn-copied");
        this.copyBtnText.textContent = "Copier";
      }, 1600);
    } catch (err) {
      console.error("Échec de la copie:", err);
    }
  }

  private showToast(message: string) {
    this.toastText.textContent = message;
    this.toast.classList.add("show");
    setTimeout(() => {
      this.toast.classList.remove("show");
    }, 2200);
  }

  private loadSample() {
    const samples: Record<string, string> = {
      hex: "Bienvenue sur Glass Converter ! Un encodeur ultra-rapide en Rust 🦀",
      binary: "Tauri + Rust = Performance & Élégance",
      base64: "Sécurité & Confidentialité : aucun historique conservé.",
      base32: "RFC 4648 Base32 Encoding",
      morse: "SOS HELLO WORLD 2026",
      url: "https://tauri.app/fr/docs/?search=glassmorphism&speed=fast#modern",
      caesar: "Ce message est chiffré avec l'algorithme historique de César !",
      html: "<span class=\"glass-pill\">Verre dépoli & reflets luminescents</span>",
      asciidec: "Code ASCII Décimal",
      asciioct: "Octal 8-bits",
      reverse: "Épuré, moderne et fenêtré",
      punycode: "café-crème.fr",
    };

    if (this.toEncoded) {
      this.sourceInput.value = samples[this.currentFormat] || "Exemple de texte moderne";
    } else {
      // Si on est en mode décodage, charger un exemple déjà encodé !
      const encodedSamples: Record<string, string> = {
        hex: "42 6f 6e 6a 6f 75 72 20 6c 65 20 6d 6f 6e 64 65",
        binary: "01000010 01101111 01101110 01101010 01101111 01110101 01110010",
        base64: "VGF1cmkgKyBSdXN0ID0gUGVyZm9ybWFuY2U=",
        base32: "JBSWY3DPEBLW64TMMQ======",
        morse: "... --- ... / .... . .-.. .-.. ---",
        url: "https%3A%2F%2Fexample.com%2F%3Fq%3Dtest%20modern",
        caesar: "Pr zrffntr rfg puvssre nirp y'nytbevguzr!",
        html: "&lt;div&gt;Texte &amp; Symboles&lt;/div&gt;",
        asciidec: "72 101 108 108 111",
        asciioct: "110 145 154 154 157",
        reverse: "ertênet te enredom ,érupÉ",
        punycode: "xn--caf-crme-d4a.fr",
      };
      this.sourceInput.value = encodedSamples[this.currentFormat] || "48 65 6c 6c 6f";
    }

    this.scheduleConversion();
  }

  // Synthétiseur audio Web Audio API pour le code Morse
  private async toggleMorseAudio() {
    if (this.isMorsePlaying) {
      this.stopMorseAudio();
      return;
    }

    const morseText = this.toEncoded ? this.targetOutput.value : this.sourceInput.value;
    if (!morseText || !morseText.trim()) {
      this.showToast("Aucun code Morse à écouter");
      return;
    }

    this.isMorsePlaying = true;
    const soundLabel = document.getElementById("morse-sound-label");
    if (soundLabel) soundLabel.textContent = "Arrêter l'audio";

    if (!this.audioCtx) {
      this.audioCtx = new (window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext)();
    }

    const dotDuration = 0.07; // 70ms par point
    const freq = 650; // Fréquence 650Hz agréable

    try {
      let currentTime = this.audioCtx.currentTime + 0.05;

      for (const char of morseText) {
        if (!this.isMorsePlaying) break;

        if (char === ".") {
          this.playBeep(currentTime, dotDuration, freq);
          currentTime += dotDuration + dotDuration; // Durée du point + silence inter-élément
        } else if (char === "-") {
          this.playBeep(currentTime, dotDuration * 3, freq);
          currentTime += dotDuration * 3 + dotDuration; // Durée du trait + silence
        } else if (char === " ") {
          currentTime += dotDuration * 2; // Espace entre lettres
        } else if (char === "/") {
          currentTime += dotDuration * 5; // Espace entre mots
        }
      }

      const totalWait = (currentTime - this.audioCtx.currentTime) * 1000;
      setTimeout(() => {
        this.stopMorseAudio();
      }, Math.max(totalWait, 100));
    } catch (e) {
      console.error("Audio error:", e);
      this.stopMorseAudio();
    }
  }

  private playBeep(startTime: number, duration: number, freq: number) {
    if (!this.audioCtx) return;
    const osc = this.audioCtx.createOscillator();
    const gain = this.audioCtx.createGain();

    osc.type = "sine";
    osc.frequency.setValueAtTime(freq, startTime);

    // Enveloppe d'attaque et d'extinction douce (éviter les clics audio)
    gain.gain.setValueAtTime(0, startTime);
    gain.gain.linearRampToValueAtTime(0.2, startTime + 0.005);
    gain.gain.setValueAtTime(0.2, startTime + duration - 0.005);
    gain.gain.linearRampToValueAtTime(0, startTime + duration);

    osc.connect(gain);
    gain.connect(this.audioCtx.destination);

    osc.start(startTime);
    osc.stop(startTime + duration);
  }

  private stopMorseAudio() {
    this.isMorsePlaying = false;
    const soundLabel = document.getElementById("morse-sound-label");
    if (soundLabel) soundLabel.textContent = "Écouter le Morse (Bips audio)";
  }
}

// Initialisation dès que le DOM est chargé
window.addEventListener("DOMContentLoaded", () => {
  new ConverterApp();
});
