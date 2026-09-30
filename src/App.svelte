<script lang="ts">
  import { onMount } from "svelte";
  import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
  import type { CodecMeta } from "./bindings/CodecMeta";
  import type { ConvertRequest } from "./bindings/ConvertRequest";
  import type { Direction } from "./bindings/Direction";
  import type { OptionValue } from "./bindings/OptionValue";
  import type { Options } from "./bindings/Options";
  import type { Span } from "./bindings/Span";
  import { asCodecError, convert, listCodecs } from "./lib/api";
  import { LatestRequest } from "./lib/latest";
  import { MorsePlayer } from "./lib/morse-player";
  import { defaultOptions } from "./lib/options";
  import { spanToUtf16 } from "./lib/positions";
  import { FALLBACK_SAMPLE, SAMPLES } from "./lib/samples";
  import { charCount, formatMegabytes, utf8ByteLength } from "./lib/text";
  import FormatTabs from "./lib/components/FormatTabs.svelte";
  import Notice from "./lib/components/Notice.svelte";
  import OptionsBar from "./lib/components/OptionsBar.svelte";
  import SourcePanel from "./lib/components/SourcePanel.svelte";
  import TargetPanel from "./lib/components/TargetPanel.svelte";
  import Toast from "./lib/components/Toast.svelte";

  /** Au-delà, la conversion peut ralentir l'interface : un avertissement s'affiche. */
  const WARN_BYTES = 5_000_000;
  /** Au-delà, la saisie directe est refusée (le mode fichier arrivera plus tard). */
  const MAX_BYTES = 50_000_000;
  const EMPTY_STATS = { inChars: 0, inBytes: 0, outChars: 0, outBytes: 0 };

  // État de l'application (purement en mémoire, aucun historique sauvegardé)
  let codecs = $state<CodecMeta[]>([]);
  let codecId = $state("hex");
  let direction = $state<Direction>("encode");
  let optionsByCodec = $state<Record<string, Options>>({});
  let input = $state("");
  let output = $state("");
  let outputIsBinary = $state(false);
  let stats = $state({ ...EMPTY_STATS });
  let notice = $state<{ level: "error" | "warning"; message: string; span: Span | null } | null>(null);
  let status = $state("Prêt");
  let copied = $state(false);
  let toast = $state({ message: "", error: false, visible: false });
  let morsePlaying = $state(false);
  let sourceTextarea: HTMLTextAreaElement | undefined = $state();

  const codec = $derived(codecs.find((c) => c.id === codecId));
  const options = $derived(optionsByCodec[codecId] ?? {});
  const hasError = $derived(notice?.level === "error");
  const formatLabel = $derived(codec?.label ?? codecId);
  const sourceTitle = $derived(direction === "encode" ? "Texte en clair" : formatLabel);
  const targetTitle = $derived(
    direction === "encode" ? formatLabel : outputIsBinary ? "Octets" : "Texte en clair"
  );

  const conversions = new LatestRequest();
  const morse = new MorsePlayer(() => (morsePlaying = false));
  let toastTimer = 0;

  onMount(async () => {
    try {
      codecs = await listCodecs();
      optionsByCodec = Object.fromEntries(codecs.map((c) => [c.id, defaultOptions(c)]));
    } catch (error) {
      notice = {
        level: "error",
        message: `Impossible de charger les formats : ${asCodecError(error).message}`,
        span: null,
      };
    }
  });

  // Conversion en temps réel, 20 ms après le dernier changement de saisie, de format,
  // de sens ou d'option.
  $effect(() => {
    const request: ConvertRequest = {
      codec: codecId,
      direction,
      input,
      options: $state.snapshot(options),
    };
    if (codecs.length === 0) return;
    const timer = setTimeout(() => run(request), 20);
    return () => clearTimeout(timer);
  });

  async function run(request: ConvertRequest) {
    const isLatest = conversions.begin();

    if (!request.input) {
      output = "";
      outputIsBinary = false;
      notice = null;
      stats = { ...EMPTY_STATS };
      status = "Prêt";
      return;
    }

    const inputBytes = utf8ByteLength(request.input);
    if (inputBytes > MAX_BYTES) {
      fail(
        request.input,
        inputBytes,
        `Texte trop volumineux (${formatMegabytes(inputBytes)}, maximum ${formatMegabytes(MAX_BYTES)}) : le traitement de fichiers arrivera dans une prochaine version.`,
        null
      );
      return;
    }

    status = "Conversion...";
    try {
      const response = await convert(request);
      // Une saisie plus récente a déjà relancé une conversion : ce résultat est périmé.
      if (!isLatest()) return;

      outputIsBinary = response.output.kind === "bytes";
      output = response.output.kind === "text" ? response.output.text : response.output.hex;
      stats = {
        inChars: response.input_chars,
        inBytes: response.input_bytes,
        outChars: response.output_chars,
        outBytes: response.output_bytes,
      };
      notice =
        inputBytes > WARN_BYTES
          ? {
              level: "warning",
              message: `Texte volumineux (${formatMegabytes(inputBytes)}) : la conversion peut prendre du temps.`,
              span: null,
            }
          : null;
      status = "Converti";
    } catch (error) {
      if (!isLatest()) return;
      const { message, span } = asCodecError(error);
      fail(request.input, inputBytes, message, span);
    }
  }

  /** En erreur, l'ancien résultat disparaît : il ne peut plus être copié ni réinjecté. */
  function fail(text: string, bytes: number, message: string, span: Span | null) {
    output = "";
    outputIsBinary = false;
    stats = { inChars: charCount(text), inBytes: bytes, outChars: 0, outBytes: 0 };
    notice = { level: "error", message, span };
    status = "Erreur";
  }

  function selectCodec(id: string) {
    if (id !== codecId) morse.stop();
    codecId = id;
  }

  function setOption(id: string, value: OptionValue) {
    optionsByCodec[codecId] = { ...optionsByCodec[codecId], [id]: value };
  }

  function swapDirection() {
    direction = direction === "encode" ? "decode" : "encode";
    const previousInput = input;
    input = output;
    output = previousInput;
  }

  async function copyResult() {
    if (!output) return;
    try {
      await writeText(output);
      copied = true;
      showToast("Résultat copié dans le presse-papier !");
      setTimeout(() => (copied = false), 1600);
    } catch (error) {
      showToast(`Copie impossible : ${String(error)}`, true);
    }
  }

  async function pasteSource() {
    try {
      const text = await readText();
      if (!text) {
        showToast("Le presse-papier est vide", true);
        return;
      }
      input = text;
      showToast("Texte collé depuis le presse-papier");
    } catch {
      showToast("Le presse-papier ne contient pas de texte lisible", true);
    }
  }

  function clearAll() {
    conversions.invalidate();
    input = "";
    output = "";
    outputIsBinary = false;
    notice = null;
    stats = { ...EMPTY_STATS };
    status = "Prêt";
    showToast("Zone de conversion vidée");
    sourceTextarea?.focus();
  }

  async function loadSample() {
    const sample = SAMPLES[codecId] ?? FALLBACK_SAMPLE;
    if (direction === "encode") {
      input = sample;
      return;
    }
    // En décodage, l'exemple est l'encodage du texte clair avec les options courantes :
    // il est donc toujours valide.
    try {
      const response = await convert({
        codec: codecId,
        direction: "encode",
        input: sample,
        options: $state.snapshot(options),
      });
      input = response.output.kind === "text" ? response.output.text : response.output.hex;
    } catch (error) {
      showToast(`Exemple indisponible : ${asCodecError(error).message}`, true);
    }
  }

  function toggleMorse() {
    if (morsePlaying) {
      morse.stop();
      return;
    }
    const morseText = direction === "encode" ? output : input;
    if (morse.play(morseText)) {
      morsePlaying = true;
    } else {
      showToast("Aucun code Morse à écouter", true);
    }
  }

  function showToast(message: string, error = false) {
    toast = { message, error, visible: true };
    window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => (toast.visible = false), 2200);
  }

  /** Sélectionne la plage fautive dans la zone de saisie. */
  function locateError() {
    if (!notice?.span || !sourceTextarea) return;
    const { start, end } = spanToUtf16(input, notice.span);
    sourceTextarea.focus();
    sourceTextarea.setSelectionRange(start, end);
  }
</script>

<div class="app-container">
  <!-- Barre d'en-tête de l'application -->
  <header class="app-header glass-card">
    <div class="brand">
      <div class="brand-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="12 2 2 7 12 12 22 7 12 2"></polygon>
          <polyline points="2 17 12 22 22 17"></polyline>
          <polyline points="2 12 12 17 22 12"></polyline>
        </svg>
      </div>
      <div class="brand-text">
        <h1 class="brand-title">Glass Converter</h1>
        <span class="brand-subtitle">Encodeur &amp; Décodeur Universel</span>
      </div>
    </div>

    <!-- Badge de confidentialité stricte (Zéro historique) -->
    <div
      class="privacy-badge"
      title="Toutes les conversions sont traitées en mémoire vive. Aucun historique n'est stocké sur votre machine."
    >
      <span class="privacy-dot"></span>
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
        <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
      </svg>
      <span>Zéro Historique • Éphémère</span>
    </div>

    <!-- Actions rapides globales -->
    <div class="header-actions">
      <button class="btn-ghost" title="Insérer un texte d'exemple pour tester" onclick={loadSample}>
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="16" y1="13" x2="8" y2="13"></line>
          <line x1="16" y1="17" x2="8" y2="17"></line>
          <polyline points="10 9 9 9 8 9"></polyline>
        </svg>
        <span>Exemple</span>
      </button>
      <button class="btn-ghost btn-danger-hover" title="Effacer tout" onclick={clearAll}>
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
        <span>Vider</span>
      </button>
    </div>
  </header>

  <FormatTabs {codecs} selected={codecId} onselect={selectCodec} />

  <OptionsBar {codec} {options} onchange={setOption} {morsePlaying} ontogglemorse={toggleMorse} />

  {#if notice}
    <Notice
      level={notice.level}
      message={notice.message}
      onlocate={notice.span ? locateError : undefined}
    />
  {/if}

  <!-- Espace de travail : Panneau double de conversion -->
  <main class="converter-workspace">
    <SourcePanel
      title={sourceTitle}
      value={input}
      highlight={notice?.level === "error" ? notice.span : null}
      chars={stats.inChars}
      bytes={stats.inBytes}
      oninput={(value) => (input = value)}
      onpaste={pasteSource}
      onclear={() => {
        input = "";
        sourceTextarea?.focus();
      }}
      bind:textarea={sourceTextarea}
    />

    <!-- Séparateur central avec Bouton d'inversion des sens -->
    <div class="swap-divider">
      <div class="divider-line"></div>
      <button
        class="btn-swap glass-card"
        class:rotated={direction === "decode"}
        disabled={hasError || outputIsBinary}
        title={outputIsBinary
          ? "Un résultat binaire ne peut pas être réinjecté comme texte"
          : "Inverser le sens de conversion (Texte ⇄ Encodé)"}
        aria-label="Inverser le sens de conversion"
        onclick={swapDirection}
      >
        <svg class="swap-icon" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="17 1 21 5 17 9"></polyline>
          <path d="M3 11V9a4 4 0 0 1 4-4h14"></path>
          <polyline points="7 23 3 19 7 15"></polyline>
          <path d="M21 13v2a4 4 0 0 1-4 4H3"></path>
        </svg>
      </button>
      <div class="divider-line"></div>
    </div>

    <TargetPanel
      title={targetTitle}
      value={output}
      binary={outputIsBinary}
      chars={stats.outChars}
      bytes={stats.outBytes}
      {status}
      copyDisabled={hasError || !output}
      {copied}
      oncopy={copyResult}
    />
  </main>
</div>

<Toast message={toast.message} error={toast.error} visible={toast.visible} />
