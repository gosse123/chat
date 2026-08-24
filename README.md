# TchatLAN

> Nom provisoire — à renommer librement (remplacer "TchatLAN" par le nom final dans les 4 fichiers).

## 1. Description

TchatLAN est une application de chat fonctionnant exclusivement sur réseau local (LAN), sans serveur central. Contrairement à WhatsApp ou Telegram qui reposent sur une infrastructure cloud, TchatLAN permet à des machines situées sur le même réseau de se découvrir automatiquement et d'échanger des messages directement entre elles, de pair à pair.

Le projet reprend l'architecture éprouvée de **Toolé** (transfert de fichiers P2P chiffré) et l'adapte au cas d'usage de la messagerie instantanée : transport QUIC/TLS 1.3, séparation stricte entre logique métier et interface, gestion concurrente via des tâches Tokio.

## 2. Pourquoi ce projet

- Comprendre en profondeur les mécanismes de découverte réseau (multicast) et de transport bas niveau (QUIC).
- Réutiliser et faire évoluer les briques déjà maîtrisées sur Toolé plutôt que de repartir de zéro.
- Construire une application 100% terminal (TUI), sans dépendance à un serveur ou à Internet.
- Approfondir la pratique du TDD (cycle Red-Green-Refactor) sur un projet réseau complet.

## 3. Fonctionnalités visées

- Découverte automatique des pairs présents sur le réseau local (multicast UDP).
- Connexion directe pair-à-pair via QUIC, chiffrée en TLS 1.3.
- Envoi et réception de messages texte en temps réel.
- Liste des pairs connectés/déconnectés, mise à jour en direct.
- Historique de conversation persisté localement (SQLite).
- Interface terminal interactive (ratatui).
- Évolutions futures possibles : groupes de discussion, partage de fichiers (réutilisation du moteur Toolé), accusés de lecture.

## 4. Différences avec un chat classique type WhatsApp

| Aspect | WhatsApp | TchatLAN |
|---|---|---|
| Infrastructure | Serveurs centraux (cloud) | Aucune, pur P2P local |
| Découverte des contacts | Carnet de contacts / numéro de téléphone | Multicast automatique sur le LAN |
| Portée réseau | Internet, mondial | Réseau local uniquement |
| Dépendance à un compte | Oui | Non |
| Fonctionnement hors ligne (sans Internet) | Non | Oui, tant que les machines sont sur le même LAN |

## 5. Stack technique

- **Langage** : Rust
- **Runtime asynchrone** : Tokio
- **Transport chiffré** : QUIC via `quinn`, TLS 1.3
- **Découverte réseau** : sockets UDP multicast (crate `socket2` / `tokio::net::UdpSocket`)
- **Interface utilisateur** : `ratatui` (TUI)
- **Persistance locale** : SQLite via `sqlx`
- **Sérialisation** : `serde` (JSON ou `bincode` selon les messages)

## 6. Architecture en un coup d'œil

L'application est découpée en modules indépendants : un module de découverte (annonce et détection des pairs), un module de transport (connexions QUIC), un module de protocole applicatif (format et types de messages), un module de persistance (historique local), et un module d'interface (ratatui), relié au reste via une abstraction (trait) comme cela a été fait pour Toolé.

Le détail complet est décrit dans `ARCHITECTURE.md`.

## 7. Documents du projet

- `README.md` — ce document, vue d'ensemble et prise en main.
- `ARCHITECTURE.md` — conception détaillée du système, composants, choix techniques.
- `PROTOCOLE.md` — spécification du protocole applicatif (messages, découverte, formats).
- `ROADMAP.md` — plan de développement par phases, avec critères de validation.

## 8. Prérequis

- Rust (édition 2021 ou supérieure), installé via `rustup`.
- Un réseau local supportant le trafic multicast (la plupart des réseaux domestiques/bureau conviennent ; certains réseaux Wi-Fi d'entreprise bloquent le multicast).
- Deux machines minimum sur le même LAN pour tester la découverte et l'échange.

## 9. Installation (à ajuster une fois le code démarré)

1. Cloner le dépôt.
2. Compiler le projet avec `cargo build --release`.
3. Lancer l'application sur chaque machine du réseau avec `cargo run --release`.
4. Les instances se découvrent automatiquement ; la liste des pairs apparaît dans l'interface.

## 10. Structure de projet envisagée

- `crates/core` — logique métier pure (découverte, protocole, transport), indépendante de l'UI.
- `crates/tui` — interface terminal avec `ratatui`, consomme `core` via un trait d'abstraction.
- `crates/storage` — accès à la base SQLite via `sqlx`, historique des messages.
- `tests/` — tests d'intégration bout-en-bout (deux instances simulées communiquant entre elles).

## 11. Licence

À définir (MIT ou Apache-2.0 recommandées pour un projet Rust open source, comme pour Toolé).
