# 💬 Système de Chat Multicast LAN

> Système de messagerie instantanée décentralisé permettant à plusieurs machines d'échanger des messages sur un réseau local (LAN) à l'aide du **multicast IP**.

---

## 1. Présentation du projet

### 1.1 Contexte

Les outils de communication modernes tels que WhatsApp, Discord ou Messenger dépendent généralement d'une connexion Internet et d'une infrastructure serveur.

Ce projet propose une approche différente : créer un système de communication **entièrement local**, capable de fonctionner entre plusieurs machines connectées au même réseau.

Le projet a également une vocation pédagogique. Il permet d'expérimenter concrètement avec :

* les sockets réseau ;
* UDP ;
* le multicast IP ;
* la découverte automatique des participants ;
* la communication entre plusieurs machines ;
* le chiffrement des données ;
* la persistance des messages ;
* la conception architecturale d'un système distribué.

### 1.2 Objectif

L'objectif est de concevoir une application de chat fonctionnant sur un réseau local et permettant aux utilisateurs de :

* découvrir automatiquement les autres participants ;
* rejoindre un groupe de discussion sans créer de compte ;
* envoyer et recevoir des messages en temps réel ;
* conserver un historique des conversations ;
* communiquer sans serveur central ;
* protéger le contenu des messages par chiffrement.

### 1.3 Principe général

Chaque instance de l'application représente un **nœud du réseau**.

Les différents nœuds communiquent via un groupe **IP multicast**.

```text
                    Réseau local
                         │
              Groupe multicast IP
                         │
       ┌─────────────────┼─────────────────┐
       │                 │                 │
    ┌──────┐          ┌──────┐          ┌──────┐
    │ PC 1 │          │ PC 2 │          │ PC 3 │
    └──────┘          └──────┘          └──────┘
       │                 │                 │
       └────────── Chat multicast ─────────┘
```

Il n'existe donc pas de serveur central obligatoire.

---

# 2. Fonctionnalités

## 2.1 Découverte automatique

Lorsqu'un utilisateur démarre l'application, une annonce de présence est envoyée sur le groupe multicast.

Les autres utilisateurs peuvent ainsi détecter automatiquement le nouveau participant.

### Exemple

```text
[14:23:41] Alice est en ligne
[14:23:43] Bob est en ligne
[14:23:47] Charlie est en ligne
```

---

## 2.2 Messagerie de groupe

Un message envoyé par un utilisateur est transmis au groupe multicast.

Tous les participants connectés au groupe peuvent recevoir le message.

```text
Alice → "Bonjour tout le monde !"

        ↓ multicast

Bob     → reçoit le message
Charlie → reçoit le message
David   → reçoit le message
```

---

## 2.3 Historique des conversations

Les messages reçus sont conservés afin de pouvoir être consultés ultérieurement.

Le système conserve notamment :

* le pseudo de l'expéditeur ;
* le contenu du message ;
* la date et l'heure d'envoi.

Le système prévoit la conservation des **100 derniers messages**.

L'historique doit rester disponible après le redémarrage de l'application.

---

## 2.4 Chiffrement

Les messages sont chiffrés avant leur transmission sur le réseau.

À la réception, ils sont déchiffrés afin d'être affichés à l'utilisateur.

```text
Message clair
     │
     ▼
Chiffrement
     │
     ▼
Réseau multicast
     │
     ▼
Déchiffrement
     │
     ▼
Message clair
```

Le choix architectural retenu est un **chiffrement symétrique AES**.

La clé peut être partagée manuellement entre les utilisateurs.

---

## 2.5 Fonctionnement sans compte

Aucun système de création de compte n'est nécessaire.

L'application est conçue pour un usage local et temporaire.

Un utilisateur peut donc démarrer l'application et rejoindre directement le chat.

---

## 2.6 Fonctionnement hors Internet

Le système est limité au réseau local.

Il n'a pas pour objectif de fournir une communication Internet ou inter-réseaux.

Le multicast est configuré avec un **TTL de 1** afin de limiter la portée des paquets au réseau local configuré.

---

# 3. Périmètre du projet

### Inclus

* [x] Communication sur réseau local
* [x] Multicast IP
* [x] Découverte automatique
* [x] Messagerie de groupe
* [x] Historique des messages
* [x] Chiffrement
* [x] Interface terminal
* [x] Fonctionnement sans serveur central
* [x] Fonctionnement sans compte utilisateur

### Hors périmètre

* [ ] Communication Internet
* [ ] Création de comptes
* [ ] Authentification
* [ ] Messagerie 1-à-1
* [ ] Gestion avancée des salons
* [ ] Interface graphique

---

# 4. Architecture

L'architecture du système repose sur quatre principaux composants :

```text
┌──────────────────────────────────────────┐
│          SYSTÈME DE CHAT LAN             │
│                                          │
│  ┌─────────────┐                         │
│  │ Interface   │                         │
│  │ Terminal    │                         │
│  └──────┬──────┘                         │
│         │                                │
│         ▼                                │
│  ┌─────────────┐      ┌──────────────┐  │
│  │   Réseau    │◄────►│  Sécurité    │  │
│  │ UDP Multicast│      │    AES       │  │
│  └──────┬──────┘      └──────────────┘  │
│         │                                │
│         ▼                                │
│  ┌─────────────┐                         │
│  │ Historique  │                         │
│  │             │                         │
│  └─────────────┘                         │
└──────────────────────────────────────────┘
```

## 4.1 Interface Terminal

Responsabilités :

* récupérer les messages saisis par l'utilisateur ;
* rejeter les messages vides ;
* afficher les messages reçus ;
* permettre la consultation de l'historique.

L'interface est volontairement séparée du réseau afin de pouvoir éventuellement être remplacée par une interface graphique.

---

## 4.2 Module Réseau

Le module réseau constitue le cœur de la communication.

Il est responsable de :

* créer et configurer les sockets UDP ;
* rejoindre le groupe multicast ;
* envoyer les messages ;
* écouter les messages entrants ;
* détecter les nouveaux utilisateurs ;
* gérer les notifications de départ ;
* configurer le TTL multicast.

Responsabilités architecturales :

```text
G-01 → Découverte des utilisateurs
G-02 → Réception des messages
G-06 → Absence d'authentification
G-07 → Limitation du multicast au réseau local
```

---

## 4.3 Module Sécurité

Le module sécurité est responsable du chiffrement et du déchiffrement.

```text
           ┌─────────────┐
Message ──►│ Chiffrement │──► Réseau
           └─────────────┘

           ┌──────────────┐
Réseau ───►│ Déchiffrement│──► Message
           └──────────────┘
```

Cette séparation permet de changer ultérieurement le mécanisme cryptographique sans modifier l'ensemble du système.

---

## 4.4 Module Historique

Le module historique est responsable de la persistance.

Il permet :

* d'ajouter un message ;
* de conserver les messages ;
* de récupérer l'historique ;
* de restaurer les données après redémarrage.

L'architecture actuelle prévoit une persistance simple dans un fichier.

Une base SQLite pourrait être utilisée ultérieurement si les besoins évoluent.

---

# 5. Modèle de données

Chaque message possède la structure logique suivante :

```rust
struct Message {
    pseudo: String,
    contenu: String,
    time: Date<Utc>,
}
```

### Signification

| Champ     | Description                       |
| --------- | --------------------------------- |
| `pseudo`  | Identité affichée de l'expéditeur |
| `contenu` | Contenu du message                |
| `time`    | Date et heure du message          |

---

# 6. Protocole de communication

Les messages échangés utilisent une structure JSON.

### Message utilisateur

```json
{
    "type": "message",
    "pseudo": "Alice",
    "content": "Bonjour tout le monde !",
    "timestamp": "2026-08-24T14:23:45Z"
}
```

## Types de messages

| Type        | Description                          |
| ----------- | ------------------------------------ |
| `message`   | Message utilisateur standard         |
| `discovery` | Annonce de présence d'un utilisateur |
| `leave`     | Notification de départ               |

Cette distinction permet au protocole de différencier les différents événements circulant sur le réseau.

---

# 7. Exigences fonctionnelles

| ID    | Exigence                                                                                        |
| ----- | ----------------------------------------------------------------------------------------------- |
| EX-01 | Le système doit permettre aux utilisateurs de se découvrir automatiquement sur le réseau local. |
| EX-02 | Le système doit permettre l'envoi de messages entre les machines du même réseau.                |
| EX-03 | Le système doit conserver l'historique des messages.                                            |
| EX-04 | Le système doit sécuriser la transmission des messages par chiffrement.                         |
| EX-05 | Le système doit rejeter les messages vides.                                                     |
| EX-06 | Le système ne doit pas nécessiter la création d'un compte utilisateur.                          |
| EX-07 | Le système ne doit pas permettre la communication hors du réseau local.                         |

---

# 8. Garanties du système

Une exigence décrit ce que le système doit faire.

Une garantie décrit une propriété **vérifiable** que le système doit respecter.

| Garantie | Description                                                                                                      |
| -------- | ---------------------------------------------------------------------------------------------------------------- |
| G-01     | Un nouvel utilisateur est annoncé aux utilisateurs connectés dans les 5 secondes suivant sa connexion.           |
| G-02     | Un message envoyé est reçu par les utilisateurs connectés au groupe multicast.                                   |
| G-03     | Les 100 derniers messages restent disponibles après redémarrage.                                                 |
| G-04     | Le contenu des messages transmis est illisible lors d'une interception réseau lorsque le chiffrement est activé. |
| G-05     | Aucun message vide n'est transmis sur le réseau.                                                                 |
| G-06     | Aucun identifiant ou mot de passe n'est nécessaire pour rejoindre le chat.                                       |
| G-07     | Le multicast utilise un TTL de 1 afin de limiter la portée réseau.                                               |

---

# 9. Traçabilité

La chaîne de conception du système suit le principe :

```text
Exigence
    │
    ▼
Garantie
    │
    ▼
Responsabilité
    │
    ▼
Composant
    │
    ▼
Implémentation
    │
    ▼
Test
```

Exemple :

```text
EX-04
Sécuriser les messages
        │
        ▼
G-04
Messages illisibles en cas d'interception
        │
        ▼
Responsabilité
Chiffrer avant transmission
        │
        ▼
Module Sécurité
        │
        ▼
AES
        │
        ▼
Test avec Wireshark
```

Cette traçabilité permet de relier directement une exigence métier à son implémentation et à son test.

---

# 10. Décisions architecturales

## 10.1 Pourquoi UDP Multicast ?

Le projet utilise UDP Multicast plutôt qu'un simple broadcast.

### Avantages

* communication vers plusieurs participants ;
* pas besoin de serveur central ;
* seuls les membres du groupe multicast reçoivent les données ;
* adapté à un environnement LAN ;
* possibilité de créer plusieurs groupes multicast.

### Alternative rejetée

**Broadcast**

Le multicast offre une meilleure organisation de la communication en permettant de cibler un groupe précis.

---

## 10.2 Pourquoi ne pas utiliser un serveur TCP centralisé ?

Une architecture client/serveur aurait nécessité :

```text
Client ──► Serveur ◄── Client
             ▲
             │
           Client
```

L'architecture retenue est plutôt :

```text
Client ◄────► Multicast ◄────► Client
                   ▲
                   │
                 Client
```

Cela supprime la dépendance à un serveur central et correspond au caractère local et décentralisé du projet.

---

## 10.3 Pourquoi un fichier plutôt que SQLite ?

Le projet conserve un nombre limité de messages.

La conservation des 100 derniers messages ne nécessite donc pas une infrastructure de base de données complexe.

Un fichier est :

* simple ;
* léger ;
* facile à manipuler ;
* adapté à un projet pédagogique.

SQLite pourrait être introduit dans une évolution future.

---

## 10.4 Pourquoi utiliser des threads ?

L'écoute réseau peut fonctionner dans un thread séparé.

```text
                 Application
                     │
          ┌──────────┴──────────┐
          │                     │
          ▼                     ▼
    Thread principal       Thread réseau
          │                     │
      Interface              Écoute UDP
          │                     │
          └──────────┬──────────┘
                     │
                  Messages
```

Cette approche simplifie la compréhension du fonctionnement concurrent du programme.

Une architecture asynchrone pourrait être envisagée ultérieurement.

---

# 11. Validation et tests

## Test 01 — Découverte

### Configuration

* PC1 démarre l'application.
* PC2 démarre ensuite l'application.

### Résultat attendu

PC1 et PC2 détectent leur présence respective dans un délai maximal de 5 secondes.

---

## Test 02 — Diffusion d'un message

### Configuration

Trois machines sont connectées au même réseau.

```text
PC1 ─┐
PC2 ─┼── Multicast
PC3 ─┘
```

PC1 envoie :

```text
Bonjour !
```

### Résultat attendu

PC2 et PC3 affichent le message.

---

## Test 03 — Persistance

### Étapes

1. Envoyer plusieurs messages.
2. Fermer l'application.
3. Redémarrer l'application.

### Résultat attendu

Les messages précédemment enregistrés sont toujours accessibles.

---

## Test 04 — Chiffrement

Utiliser Wireshark pour capturer les paquets réseau.

### Résultat attendu

Lorsque le chiffrement est activé, le contenu du message ne doit pas apparaître en clair dans les données capturées.

---

## Test 05 — Message vide

Entrer :

```text
   
```

### Résultat attendu

Le système refuse l'envoi et affiche un message d'erreur.

```text
Message vide, réessayez.
```

---

## Test 06 — Absence d'authentification

Lancer l'application sans fournir de compte ou de mot de passe.

### Résultat attendu

L'utilisateur peut rejoindre directement le chat.

---

## Test 07 — TTL

Vérifier que le socket multicast utilise :

```text
TTL = 1
```

### Résultat attendu

Les paquets multicast restent limités au périmètre réseau prévu.

---

# 12. Organisation possible du projet

L'architecture peut être représentée sous la forme suivante :


### `main.rs`

Point d'entrée de l'application et gestion de l'interface terminal.

### `multicast.rs`

Gestion :

* des sockets UDP ;
* du multicast ;
* de la découverte ;
* de l'envoi ;
* de la réception.

### `history.rs`

Gestion :

* de l'historique ;
* de la sauvegarde ;
* du chargement.

### `crypto.rs`

Gestion :

* du chiffrement ;
* du déchiffrement ;
* des clés.

---

# 13. Évolutions possibles

L'architecture a été pensée pour permettre différentes évolutions.

## 13.1 Interface graphique

Une interface GUI pourrait remplacer l'interface terminal.

```text
           ┌───────────────┐
           │ Interface GUI │
           └───────┬───────┘
                   │
                   ▼
             Module Réseau
                   │
          ┌────────┴────────┐
          ▼                 ▼
      Sécurité          Historique
```

Les modules réseau, sécurité et historique pourraient rester indépendants de l'interface.

---

## 13.2 Messagerie privée

Une fonctionnalité de communication 1-à-1 pourrait être ajoutée.

Le protocole devrait alors permettre d'identifier un destinataire spécifique.

---

## 13.3 Plusieurs salons

Plusieurs groupes multicast pourraient être utilisés :

```text
Salon général  → 239.0.0.1
Salon développeurs → 239.0.0.2
Salon étudiants → 239.0.0.3
```

L'interface pourrait permettre à l'utilisateur de sélectionner le salon souhaité.

---

## 13.4 Base de données

Le fichier d'historique pourrait être remplacé par SQLite.

```text
Historique
    │
    ├── Version actuelle
    │      └── Fichier
    │
    └── Évolution
           └── SQLite
```

---

## 13.5 Authentification

Une authentification pourrait être ajoutée dans une future version.

Cette évolution modifierait toutefois une contrainte fondamentale de la version actuelle : le fonctionnement sans compte.

---

## 13.6 Passage à une architecture WebSocket

Le module réseau pourrait éventuellement être remplacé par une architecture utilisant WebSocket.

L'interface et le module historique pourraient rester relativement indépendants de ce changement.

---

# 14. Glossaire

| Terme         | Définition                                                                                        |
| ------------- | ------------------------------------------------------------------------------------------------- |
| **LAN**       | Local Area Network, réseau local reliant plusieurs machines.                                      |
| **UDP**       | User Datagram Protocol, protocole de transport sans connexion.                                    |
| **Multicast** | Transmission d'un paquet vers un groupe de destinataires.                                         |
| **Unicast**   | Communication entre un émetteur et un destinataire.                                               |
| **Broadcast** | Transmission vers tous les hôtes d'un réseau local.                                               |
| **TTL**       | Time To Live, valeur limitant la portée d'un paquet IP.                                           |
| **IGMP**      | Internet Group Management Protocol, utilisé pour gérer l'appartenance aux groupes multicast IPv4. |
| **Socket**    | Interface logicielle permettant à une application de communiquer sur un réseau.                   |
| **Invariant** | Propriété qui doit toujours être vraie.                                                           |
| **Cohésion**  | Degré de cohérence des responsabilités regroupées dans un composant.                              |
| **AES**       | Advanced Encryption Standard, algorithme de chiffrement symétrique.                               |

---

# 15. Limites actuelles

La version actuelle présente volontairement plusieurs limitations :

* fonctionnement limité au réseau local ;
* absence de serveur central ;
* absence de comptes utilisateurs ;
* absence d'authentification ;
* absence de messagerie privée ;
* absence d'interface graphique ;
* mécanisme de persistance simple ;
* clé de chiffrement partagée manuellement.

Ces limitations ne sont pas nécessairement des défauts : certaines correspondent directement au périmètre fonctionnel initial du projet.

---

# 16. Philosophie architecturale

Le projet repose sur un principe simple :

> **Chaque responsabilité doit avoir un propriétaire clair et chaque décision architecturale doit être justifiée par une exigence.**

La conception suit donc la chaîne :

```text
┌──────────────┐
│  Exigences   │
└──────┬───────┘
       ▼
┌──────────────┐
│  Garanties   │
└──────┬───────┘
       ▼
┌──────────────┐
│Responsabilités│
└──────┬───────┘
       ▼
┌──────────────┐
│  Composants  │
└──────┬───────┘
       ▼
┌──────────────┐
│ Implémentation│
└──────┬───────┘
       ▼
┌──────────────┐
│    Tests     │
└──────────────┘
```

Cette approche permet d'éviter de simplement écrire du code fonctionnalité par fonctionnalité sans vision globale de l'architecture.

---

# 17. Installation

### Prérequis

* **Rust** (edition 2024) — installer via [rustup](https://rustup.rs/) :

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

* Un système Linux (multicast UDP, `socket2` avec `set_reuse_port`)
* Les machines doivent être connectées au **même réseau local**

### Cloner le projet

```bash
git clone https://github.com/gosse123/chat.git
cd chat
```

### Compiler

```bash
cargo build --release
```

Le binaire est généré dans `target/release/chat`.

---

# 18. Utilisation

### Lancement simple

```bash
cargo run -- <pseudo>
```

Exemple :

```bash
cargo run -- Alice
```

L'application démarre, rejoint le groupe multicast et affiche :

```text
Bienvenue Alice! Tapez /help pour les commandes.
```

### Lancement avec chiffrement

Pour chiffrer les messages avec AES-256-GCM, fournir une clé hexadécimale de 64 caractères (32 octets) via `--key` :

```bash
cargo run -- Alice --key 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
```

> Toutes les instances doivent utiliser la **même clé** pour pouvoir se déchiffrer mutuellement.

### Commandes disponibles

| Commande | Description |
|----------|-------------|
| `/help` | Afficher la liste des commandes |
| `/history` | Afficher les 20 derniers messages |
| `/quit` | Quitter le chat (envoie un message de départ) |

### Exemple de session

```text
$ cargo run -- Alice
Bienvenue Alice! Tapez /help pour les commandes.
> Bonjour tout le monde !
> Comment allez-vous ?
> /history
--- Historique (2 messages) ---
[14:23:45] Alice : Bonjour tout le monde !
[14:23:48] Alice : Comment allez-vous ?
--- Fin de l'historique ---
> /quit
Au revoir!
```

### Tester entre plusieurs machines

1. Compiler le projet sur chaque machine du réseau local
2. Lancer l'application avec un pseudo différent sur chaque machine
3. Les messages envoyés par une machine sont reçues par toutes les autres

```text
Machine A : cargo run -- Alice
Machine B : cargo run -- Bob
Machine C : cargo run -- Charlie
```

Les messages s'affichent automatiquement sur chaque machine participante.

---

# 19. Tests

### Lancer tous les tests

```bash
cargo test
```

### Résultat attendu

```text
running 65 tests
...
test result: ok. 65 passed; 0 failed; 0 ignored
```

### Types de tests

| Catégorie | Nombre | Description |
|-----------|--------|-------------|
| Unitaires `models` | 10 | Sérialisation/désérialisation JSON |
| Unitaires `security` | 11 | Chiffrement/déchiffrement AES-256-GCM |
| Unitaires `history` | 9 | Logique en mémoire, limite 100 messages |
| Unitaires `ui` | 6 | Affichage terminal |
| Intégration `history` | 9 | Persistance fichier (tempfile) |
| Intégration `network` | 7 | Envoi/réception UDP réel |
| Intégration `security` | 9 | Bout en bout chiffrement + protocole |

---

# 20. Conclusion

Ce projet constitue une implémentation concrète d'un système de communication distribué fonctionnant sur un réseau local.

Il met en pratique plusieurs concepts importants de l'ingénierie logicielle et des réseaux :

* communication UDP ;
* multicast IP ;
* découverte distribuée ;
* concurrence ;
* chiffrement ;
* persistance ;
* séparation des responsabilités ;
* traçabilité des exigences ;
* tests d'intégration ;
* évolutivité architecturale.

L'architecture retenue permet surtout de conserver une séparation claire entre **interface**, **réseau**, **sécurité** et **persistance**.

Le projet peut ainsi évoluer progressivement sans nécessiter une réécriture complète du système.

### Chaîne architecturale finale

**Exigence → Garantie → Responsabilité → Composant → Implémentation → Test**

C'est cette chaîne qui constitue le fondement architectural du système de chat multicast LAN.
