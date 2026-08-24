# Roadmap — TchatLAN

## 1. Objectif de ce document

Découper le développement du projet en phases claires, chacune testable et validable indépendamment, en cohérence avec une approche TDD (Red-Green-Refactor). Chaque phase précise son objectif, ses livrables et ses critères de validation.

## 2. Phase 0 — Mise en place du projet

**Objectif** : disposer d'un squelette de projet compilable et organisé.

- Créer un workspace Cargo avec les crates `core`, `tui`, `storage`.
- Mettre en place les dépendances de base : `tokio`, `serde`, `quinn`, `sqlx`, `ratatui`.
- Configurer un pipeline CI minimal (compilation + tests) via GitHub Actions, sur le modèle de ce qui existe déjà pour Toolé.

**Critère de validation** : `cargo build` et `cargo test` passent sur un projet vide mais structuré.

## 3. Phase 1 — Découverte multicast

**Objectif** : deux instances de l'application se détectent automatiquement sur le même réseau.

- Implémenter l'émission périodique du message `announce` (voir `PROTOCOLE.md`, section 3).
- Implémenter l'écoute multicast et la mise à jour de la liste des pairs.
- Implémenter l'expiration automatique des pairs inactifs.
- Écrire des tests unitaires sur la sérialisation/désérialisation des messages d'annonce.
- Écrire un test d'intégration lançant deux instances locales et vérifiant qu'elles se détectent mutuellement.

**Critère de validation** : lancer l'application sur deux terminaux (ou deux machines) et voir chacune apparaître dans la liste de pairs de l'autre.

## 4. Phase 2 — Transport QUIC point-à-point

**Objectif** : établir une connexion chiffrée directe entre deux pairs détectés.

- Générer un certificat auto-signé au premier démarrage.
- Implémenter le point d'écoute QUIC entrant.
- Implémenter l'initiation d'une connexion QUIC sortante vers un pair choisi.
- Tester l'envoi et la réception d'octets bruts sur un flux QUIC entre deux instances locales.

**Critère de validation** : une instance peut se connecter à une autre et échanger des données arbitraires sur un flux QUIC, avec la connexion visible comme "chiffrée" (TLS 1.3 actif).

## 5. Phase 3 — Protocole applicatif complet

**Objectif** : implémenter l'ensemble des types de messages définis dans `PROTOCOLE.md`.

- Implémenter l'enveloppe commune des messages et chaque type (`HELLO`, `MESSAGE`, `ACK`, `PING`/`PONG`, `BYE`).
- Implémenter le cycle de vie complet d'un message texte, avec accusé de réception.
- Implémenter la détection de connexion perdue via `PING`/`PONG` et la reconnexion automatique.
- Tests unitaires sur chaque type de message, tests d'intégration sur l'échange complet entre deux instances.

**Critère de validation** : deux instances échangent des messages texte avec accusé de réception visible, et une déconnexion volontaire (`BYE`) est correctement détectée par l'autre partie.

## 6. Phase 4 — Interface terminal (ratatui)

**Objectif** : rendre le chat utilisable via une interface terminal.

- Définir le trait d'abstraction entre `core` et l'interface (à l'image de `Arc<dyn UI>` dans Toolé).
- Construire l'écran principal : liste des pairs, zone de conversation, zone de saisie.
- Relier les événements de l'interface (sélection d'un pair, envoi d'un message) aux actions de `core`.
- Afficher en temps réel les messages entrants et les changements de statut des pairs (en ligne/hors ligne).

**Critère de validation** : un utilisateur peut lancer l'application, voir les pairs disponibles, sélectionner un pair et échanger des messages visibles à l'écran, sans manipuler directement le code.

## 7. Phase 5 — Persistance locale

**Objectif** : conserver l'historique des conversations entre deux lancements de l'application.

- Définir le schéma de base de données (table des messages, table des pairs connus).
- Mettre en place les migrations via `sqlx-cli`, sur le modèle du projet de gestion utilisateurs déjà réalisé.
- Enregistrer chaque message envoyé ou reçu, avec son statut.
- Charger l'historique au démarrage et l'afficher dans l'interface.

**Critère de validation** : après redémarrage de l'application, l'historique des conversations précédentes est toujours visible.

## 8. Phase 6 — Sécurité renforcée

**Objectif** : réduire les risques liés à l'absence d'autorité de certification centrale.

- Implémenter la vérification d'empreinte de certificat à la première connexion avec un pair (confiance à la première utilisation).
- Stocker les empreintes connues localement et alerter l'utilisateur en cas de changement inattendu.
- Documenter les limites de sécurité restantes dans `ARCHITECTURE.md`.

**Critère de validation** : une tentative de connexion avec un certificat différent de celui enregistré précédemment déclenche un avertissement visible.

## 9. Phase 7 — Fonctionnalités avancées (optionnel)

**Objectif** : étendre les capacités du chat au-delà de l'échange texte simple.

- Groupes de discussion (diffusion à plusieurs pairs).
- Partage de fichiers en réutilisant le moteur de transfert chunké de Toolé (`FILE_OFFER`).
- Notifications système lors de la réception d'un message.

**Critère de validation** : à définir selon la fonctionnalité choisie en priorité.

## 10. Phase 8 — Packaging et distribution

**Objectif** : rendre l'application facilement installable, sur le modèle de ce qui a été fait pour Toolé.

- Publier un paquet Homebrew.
- Publier un `PKGBUILD` pour l'AUR (Arch Linux).
- Publier un paquet APT (Debian/Ubuntu).
- Mettre à jour la documentation d'installation dans `README.md` en conséquence.

**Critère de validation** : l'application peut être installée sur une machine tierce via l'un de ces canaux, sans passer par la compilation manuelle.

## 11. Récapitulatif des dépendances entre phases

- Les phases 1 et 2 sont indépendantes et peuvent être développées en parallèle.
- La phase 3 nécessite que les phases 1 et 2 soient terminées.
- La phase 4 nécessite que la phase 3 soit terminée (le protocole doit exister avant d'avoir une interface qui l'exploite).
- La phase 5 peut être développée en parallèle de la phase 4.
- Les phases 6, 7 et 8 sont postérieures à un socle fonctionnel complet (phases 1 à 5 terminées).
