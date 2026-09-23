# 🔮 Glass Converter — Encodeur & Décodeur Universel

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
  - Aucun stockage local (`localStorage`, cookies, fichiers logs ou bases de données proscrits).
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
| **Hexadécimal** | Séparateurs configurables (espace, continu, préfixe `0x`, deux-points), majuscules/minuscules |
| **Binaire** | Groupes d'octets de 8 bits ou séquence continue |
| **Base64** | Standard RFC 4648 ou URL-Safe (`-_`) |
| **Base32** | Alphabet RFC 4648 avec gestion du padding |
| **Code Morse** | Standard international ITU-R avec lecteur audio intégré |
| **URL Encode** | Percent-encoding conforme URI |
| **HTML Entities** | Échappement et déséchappement des entités nommées et numériques |
| **ROT13 / César** | Décalage alphabétique configurable de 1 à 25 (curseur dynamique) |
| **ASCII Décimal** | Séquences de valeurs d'octets numériques base 10 |
| **ASCII Octal** | Séquences d'octets en base 8 |
| **Inversion** | Inversion pure de chaîne |
| **Punycode (IDN)** | Standard RFC 3492 pour noms de domaine internationalisés (option préfixe `xn--` ou brut) |

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

## 🧪 Tests Unitaires Rust

Le moteur de conversion dispose d'une suite complète de tests unitaires couvrant l'encodage, le décodage et la validation des erreurs :

```bash
cd src-tauri
cargo test
```

---

## 🔨 Compiler l'Exécutable Windows (.exe)

Pour générer l'installateur et l'exécutable binaire optimisé :

```bash
npm run tauri build
```

L'exécutable autonome sera généré dans :
`src-tauri/target/release/glass-converter.exe`
