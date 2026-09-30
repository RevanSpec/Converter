# 🔮 Glass Converter — Encodeur & Décodeur Universel

[![CI](https://github.com/RevanSpec/Converter/actions/workflows/ci.yml/badge.svg)](https://github.com/RevanSpec/Converter/actions/workflows/ci.yml)

Application de bureau moderne, ultra-rapide et épurée conçue en **Rust** et **Tauri v2**, avec une interface à effet **Glassmorphism** (*verre dépoli, reflets lumineux et typographie soignée*).

L'application permet la conversion bidirectionnelle instantanée entre du texte en clair et de multiples formats d'encodage et de chiffrement, avec une garantie absolue de **Zéro Historique** (traitement 100% en mémoire volatile).

---

## ✨ Fonctionnalités Clés

- **⚡ Moteur Rust Haute Performance** : Traitement instantané des octets UTF-8, sans latence.
- **⇄ Conversion Bidirectionnelle** :
  - Sens 1 : *Texte en clair* $\rightarrow$ *Format encodé*
  - Sens 2 : *Format encodé* $\rightarrow$ *Texte en clair*
  - Bouton interactif d'inversion des sens avec animation fluide de rotation.
- **🔒 Zéro Historique & Confidentialité Totale** :
  - Traitement purement en mémoire vive (RAM).
  - Aucun stockage local (`localStorage`, cookies, fichiers logs ou bases de données proscrits) ; webview en navigation privée.
  - Aucune requête réseau : polices embarquées, CSP stricte, contrôle automatique du build en CI.
  - Bouton « Vider » pour purger instantanément les deux volets.
- **🎨 Design Glassmorphism Luxueux** :
  - Arrière-plan sombre avec halos d'ambiance dynamiques.
  - Cartes en verre dépoli (`backdrop-filter: blur(28px)`).
  - Bordures luminescentes et accents néon (indigo, cyan, violet).
  - Polices modernes (*Inter* pour l'UI, *JetBrains Mono* pour l'éditeur).
- **📋 Ergonomie Avancée** :
  - Détection automatique et affichage des erreurs de syntaxe (ex: caractère hexadécimal invalide, longueur impaire).
  - Boutons « Copier » et « Coller » avec notification toast discrète.
  - Compteurs en temps réel de caractères et d'octets UTF-8.
  - Synthétiseur audio Web Audio API pour écouter la transcription du **Code Morse** en bips sonores réels.

---

## 📦 Formats Supportés

| Format | Description & Options |
| :--- | :--- |
| **Hexadécimal** | Séparateurs configurables (espace, continu, préfixe `0x`, deux-points), majuscules/minuscules ; au décodage, accepte aussi `\x`, `%`, `-`, `,` et `;` |
| **Binaire** | Groupes d'octets de 8 bits ou séquence continue |
| **Base64** | Standard RFC 4648 ou URL-Safe (`-_`), avec ou sans `=` (forme des JWT) ; au décodage, `=` final facultatif et alphabet détecté automatiquement |
| **Base32** | Alphabet RFC 4648 avec gestion du padding ; texte tronqué signalé |
| **Code Morse** | Standard international ITU-R (dont `É`), lignes conservées, caractères sans code refusés, translittérés (`à` → `A`) ou ignorés, lecteur audio intégré |
| **URL Encode** | Percent-encoding RFC 3986 : composant (tout encoder) ou URI complète (garde `:/?#&=`), option « `+` = espace » des formulaires |
| **HTML Entities** | Échappement, et déséchappement de toutes les entités nommées HTML5 et des entités numériques |
| **ROT13 / César** | Décalage alphabétique configurable de 1 à 25 (curseur dynamique) |
| **Octets (décimal)** | Valeur de chaque octet UTF-8, de 0 à 255 |
| **Octets (octal)** | Valeur de chaque octet UTF-8 en base 8, de 000 à 377 |
| **Inversion** | Inversion par graphèmes : emojis composés et accents restent intacts |
| **Punycode (IDN)** | Noms de domaine internationalisés (UTS #46, préfixe `xn--`) ou Punycode brut RFC 3492 |

Un décodage qui ne produit pas du texte UTF-8 (une image en Base64, par exemple) affiche ses octets en hexadécimal. Chaque erreur indique sa position, et la zone fautive est surlignée dans le texte saisi.

---

## 🧱 Architecture

- `crates/converter-core` : le moteur, en Rust et sans Tauri. Chaque format implémente le trait `Codec` sur des octets et décrit ses options ; le registre les rassemble.
- `src-tauri` : l'application, qui expose les commandes `list_codecs` et `convert`.
- `src` : l'interface Svelte 5, construite à partir de `list_codecs`. Ses types (`src/bindings`) sont générés depuis Rust par `ts-rs` à chaque `cargo test`.

Ajouter un format : un fichier dans `crates/converter-core/src/codecs/`, avec ses tests, puis une ligne dans `codecs::all`. L'interface l'affiche sans autre modification.

---

## 🚀 Lancer l'Application en Développement

### 1. Prérequis
- [Rust](https://www.rust-lang.org/) (rustc & cargo)
- [Node.js](https://nodejs.org/) (npm)

### 2. Démarrage
À la racine du projet :

```bash
npm run tauri dev
```

Cette commande démarre le serveur de développement Vite et lance la fenêtre de l'application de bureau native.

---

## 🧪 Tests et Vérifications

Le moteur de conversion est couvert par quatre familles de tests :

- des tests unitaires dans le fichier de chaque format (`crates/converter-core/src/codecs/`) ;
- les vecteurs officiels de la RFC 4648 pour Base16, Base32 et Base64 (`crates/converter-core/tests/rfc4648.rs`) ;
- des tests de propriété `proptest` : décoder(encoder(x)) redonne x pour chaque format, y compris sur des octets quelconques, avec 256 cas aléatoires par propriété à chaque exécution (`crates/converter-core/tests/roundtrip_props.rs`) ;
- un test de non-régression par bug corrigé (`crates/converter-core/tests/regressions.rs`).

L'interface a ses tests Vitest (`src/lib/*.test.ts`) : calendrier des bips Morse, réponses périmées, tailles de texte, options par défaut, positions des erreurs.

La CI GitHub Actions lance les mêmes vérifications à chaque push sur `main` et sur chaque pull request. En local :

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

```bash
npm ci
npm test
npm run build
npm run check:offline
```

---

## 🔨 Compiler l'Exécutable Windows (.exe)

Pour générer l'installateur et l'exécutable binaire optimisé :

```bash
npm run tauri build
```

L'exécutable autonome sera généré dans :
`target/release/glass-converter.exe`

Le numéro de version se modifie uniquement dans le `Cargo.toml` racine (`[workspace.package]`) : les crates et Tauri le reprennent automatiquement.

---

## 🗺️ Feuille de Route

Les prochaines étapes (correctifs, moteur en octets, conversion multi-couche, nouveaux formats) sont détaillées dans [ROADMAP.md](ROADMAP.md).

---

## 📄 Licence

Glass Converter est distribué, au choix, sous l'une de ces deux licences :

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- Licence MIT ([LICENSE-MIT](LICENSE-MIT))

Sauf mention contraire explicite, toute contribution soumise pour inclusion dans ce projet, au sens de la licence Apache-2.0, est distribuée sous cette double licence, sans condition supplémentaire.
