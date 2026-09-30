# Journal des modifications

Les évolutions notables de Glass Converter sont listées ici, la plus récente en premier.
Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et le projet respecte le [versionnage sémantique](https://semver.org/lang/fr/).

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

[0.2.0]: https://github.com/RevanSpec/Converter/compare/e409023...v0.2.0
[0.1.0]: https://github.com/RevanSpec/Converter/tree/e409023
