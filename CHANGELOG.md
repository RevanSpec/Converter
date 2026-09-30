# Journal des modifications

Les évolutions notables de Glass Converter sont listées ici, la plus récente en premier.
Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et le projet respecte le [versionnage sémantique](https://semver.org/lang/fr/).

## [0.4.0] - 2026-09-30

### Ajouté

- Mode Chaîne : plusieurs formats enchaînés, chacun avec son sens et ses options, activable, déplaçable à la souris ou au clavier (`Alt+↑/↓`) ; aperçu, taille, texte ou octets et durée sous chaque couche ; la première erreur nomme sa couche.
- « Inverser » retourne une chaîne (ordre et sens) et reprend sa sortie comme entrée.
- Recettes : forme courte à copier-coller (`base64:dec|hex:dec`), JSON versionné, recettes prêtes à l'emploi (Base64 double, Data URI, gzip + Base64, SAML) et « Mes recettes », écrites seulement quand vous enregistrez ou supprimez une recette, sans texte saisi ni option secrète.
- Formats gzip, zlib, Deflate et Brotli (mode Chaîne), et Data URI.
- Résultat binaire affiché en hexdump, copiable en hexadécimal ou en Base64.

### Modifié

- Plafond de 100 Mo par conversion et par couche : une bombe de décompression s'arrête avec une erreur.

## [0.3.0] - 2026-09-30

### Ajouté

- Moteur séparé `converter-core` : chaque format travaille sur des octets ; un décodage qui ne donne pas de texte affiche les octets en hexadécimal.
- Base64 sans « = » à l'encodage (forme des JWT) ; URL en mode « URI complète » et option « + = espace ».
- Chaque erreur indique sa plage : la zone fautive est surlignée dans le texte saisi, et « Voir dans le texte » la sélectionne.
- Onglets groupés par catégorie, navigation au clavier, rôles et libellés d'accessibilité.

### Modifié

- Interface réécrite en Svelte 5 et construite à partir des formats décrits par le moteur.
- « ASCII Décimal » et « ASCII Octal » deviennent « Octets (décimal) » et « Octets (octal) ».
- URL : le « + » n'est plus lu comme une espace, sauf avec l'option « + = espace ».
- Workspace Cargo à la racine : la version se déclare dans le `Cargo.toml` racine et l'exécutable sort dans `target/release/`.

## [0.2.0] - 2026-09-30

### Corrigé

- Punycode : le décodage ne transforme plus les mots et domaines ASCII ; l'encodage suit IDNA (minuscules, normalisation).
- Base64 : un texte sans « = » final (JWT…) se décode, et l'alphabet URL-safe est détecté.
- Le décodage ne supprime plus les blancs de début et de fin (César, Inversion, HTML, URL, Punycode).
- Morse : `É` pris en charge, retours à la ligne conservés, arrêt immédiat de l'audio, 7 unités entre deux mots.
- Inversion par graphèmes : emojis composés et accents restent intacts.
- HTML : toutes les entités nommées HTML5 sont décodées.
- Base32 tronqué refusé ; séparateurs hexadécimaux `-`, `\x`, `%` et `;` acceptés ; positions d'erreur exactes.
- En cas d'erreur, l'ancien résultat n'est plus affiché ni copiable.
- Les exemples de décodage sont toujours valides ; l'option hexadécimale « Continu » fonctionne.

### Ajouté

- Réglage Morse pour les caractères sans code : erreur, translittérer (`à` → `A`) ou ignorer.
- Avertissement au-delà de 5 Mo saisis, refus au-delà de 50 Mo.
- Tests : vecteurs RFC 4648, tests de propriété, tests de non-régression, tests Vitest ; CI GitHub Actions.
- Double licence MIT OR Apache-2.0.

### Modifié

- La conversion s'exécute hors du thread de l'interface, et seule la réponse la plus récente s'affiche.
- Le presse-papier passe par le plugin officiel de Tauri, avec un message en cas d'échec.

### Sécurité

- Aucune requête réseau : polices embarquées, CSP stricte, webview en navigation privée.
- API Tauri non exposée globalement ; plugin `opener` inutilisé retiré.

## [0.1.0]

Première version : 12 formats, conversion bidirectionnelle en temps réel, interface glassmorphism.

[0.4.0]: https://github.com/RevanSpec/Converter/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RevanSpec/Converter/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/RevanSpec/Converter/compare/e409023...v0.2.0
[0.1.0]: https://github.com/RevanSpec/Converter/tree/e409023
