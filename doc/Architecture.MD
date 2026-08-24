# Architecture technique détaillée — Système de Chat Multicast LAN

## 1. Vision globale

Le système est une application de messagerie instantanée décentralisée fonctionnant sur un réseau local (LAN).

Il n'existe pas de serveur central. Chaque ordinateur exécutant l'application constitue un nœud du système et participe directement aux communications.

```text
                         RÉSEAU LOCAL

                  Groupe multicast 239.x.x.x
                              │
               ┌──────────────┼──────────────┐
               │              │              │
               ▼              ▼              ▼
           ┌────────┐     ┌────────┐     ┌────────┐
           │ Client │     │ Client │     │ Client │
           │   A    │     │   B    │     │   C    │
           └────────┘     └────────┘     └────────┘
```

Chaque nœud possède localement :

* une interface utilisateur ;
* une logique applicative ;
* un socket UDP ;
* une gestion du multicast ;
* un module de sécurité ;
* un historique local.

L'architecture générale suit la chaîne :

```text
Exigence
    ↓
Garantie
    ↓
Responsabilité
    ↓
Composant
    ↓
Implémentation
    ↓
Test
```

Cette chaîne permet de conserver une traçabilité entre les besoins fonctionnels, l'architecture et le code.

---

# 2. Architecture générale

Le système peut être représenté comme suit :

```text
┌─────────────────────────────────────────────────────┐
│                   APPLICATION                       │
│                                                     │
│  ┌──────────────┐                                   │
│  │ Interface    │                                   │
│  │ Terminal     │                                   │
│  └──────┬───────┘                                   │
│         │                                           │
│         ▼                                           │
│  ┌──────────────┐                                   │
│  │ Application  │                                   │
│  │ / Chat Logic │                                   │
│  └───┬─────┬────┘                                   │
│      │     │                                        │
│      │     ├─────────────────┐                      │
│      │     │                 │                      │
│      ▼     ▼                 ▼                      │
│  ┌───────┐ ┌──────────────┐ ┌──────────────┐       │
│  │Network│ │   Security   │ │   History    │       │
│  │       │ │     AES      │ │              │       │
│  └───┬───┘ └──────────────┘ └──────────────┘       │
│      │                                              │
└──────┼──────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────┐
│                     LAN                             │
│                                                     │
│                 UDP MULTICAST                       │
└─────────────────────────────────────────────────────┘
```

---

# 3. Principes architecturaux

L'architecture repose sur plusieurs principes.

## 3.1 Séparation des responsabilités

Chaque composant possède une responsabilité clairement définie.

```text
Interface  → interaction avec l'utilisateur
Network    → communication réseau
Security   → chiffrement/déchiffrement
History    → persistance
Application → orchestration
```

Un composant ne doit pas prendre en charge des responsabilités qui appartiennent à un autre composant.

---

## 3.2 Faible couplage

Les composants doivent communiquer par des interfaces clairement définies.

Par exemple, l'interface terminal ne doit pas avoir à connaître les détails de fonctionnement d'un socket UDP.

Elle doit simplement demander l'envoi d'un message à la logique applicative.

```text
Interface
    │
    │ send_message()
    ▼
Application
    │
    ▼
Network
```

Ainsi, une future interface graphique pourra remplacer l'interface terminal sans modifier directement le fonctionnement du réseau.

---

## 3.3 Forte cohésion

Les fonctionnalités qui changent pour les mêmes raisons sont regroupées.

Par exemple :

```text
Network
├── Socket
├── Multicast
├── Discovery
└── Protocol
```

Ces éléments sont tous liés à la communication réseau.

À l'inverse :

```text
History
```

reste indépendant du réseau autant que possible.

---

# 4. Architecture en couches

Une organisation logique en couches peut être utilisée :

```text
┌───────────────────────────────┐
│       PRESENTATION            │
│                               │
│       Interface Terminal      │
└───────────────┬───────────────┘
                │
                ▼
┌───────────────────────────────┐
│       APPLICATION             │
│                               │
│       Chat / Orchestration    │
└───────────────┬───────────────┘
                │
                ▼
┌───────────────────────────────┐
│       INFRASTRUCTURE          │
│                               │
│ Network │ Security │ History  │
└───────────────┬───────────────┘
                │
                ▼
┌───────────────────────────────┐
│           OS / LAN            │
│                               │
│ UDP / IP / Wi-Fi / Ethernet   │
└───────────────────────────────┘
```

Cette séparation permet de distinguer :

* ce que voit l'utilisateur ;
* la logique de l'application ;
* les détails techniques ;
* l'infrastructure réseau.

---

# 5. Composant Interface

## Responsabilité

Le composant Interface est responsable de l'interaction avec l'utilisateur.

Il doit :

* lire les entrées utilisateur ;
* valider les messages ;
* afficher les messages ;
* afficher les événements de présence ;
* afficher les erreurs ;
* permettre la consultation de l'historique.

## Flux d'entrée

```text
Utilisateur
    │
    ▼
stdin
    │
    ▼
Interface
    │
    ▼
Validation
```

## Validation

Un message vide ne doit jamais être envoyé.

```text
Message
   │
   ▼
Est-il vide ?
   │
   ├── Oui ──► Erreur
   │
   └── Non ──► Continuer
```

Cette responsabilité correspond à la garantie G-05.

## Sortie

Lorsqu'un message est reçu :

```text
Network
   │
   ▼
Application
   │
   ▼
Interface
   │
   ▼
Terminal
```

---

# 6. Composant Application

Le composant Application joue le rôle d'orchestrateur.

Il coordonne les autres composants sans prendre leur responsabilité.

Il peut être vu comme :

```text
                 Application
                     │
        ┌────────────┼────────────┐
        │            │            │
        ▼            ▼            ▼
     Network      Security      History
```

Il décide notamment :

* quand envoyer un message ;
* quand enregistrer un message ;
* quand afficher un événement ;
* comment traiter un événement reçu.

L'Application ne doit cependant pas connaître les détails internes des sockets ou du système de fichiers.

---

# 7. Composant Network

Le module réseau constitue le cœur de la communication.

Il est responsable de :

* créer le socket UDP ;
* configurer le socket ;
* rejoindre le groupe multicast ;
* envoyer les paquets ;
* recevoir les paquets ;
* gérer la découverte ;
* gérer les notifications de départ ;
* configurer le TTL.

Une décomposition interne peut être utilisée :

```text
network/
├── socket.rs
├── multicast.rs
├── discovery.rs
└── protocol.rs
```

---

# 8. Socket UDP

Le transport utilisé est UDP.

UDP est adapté au modèle multicast mais ne fournit pas :

* de connexion ;
* de garantie de livraison ;
* de retransmission automatique ;
* d'ordre garanti des paquets.

L'application doit donc considérer les messages réseau comme des données pouvant potentiellement être perdues ou dupliquées.

Architecture :

```text
Application
    │
    ▼
Network
    │
    ▼
UDP Socket
    │
    ▼
IP Multicast
```

---

# 9. Groupe multicast

Tous les clients rejoignent le même groupe multicast.

Exemple conceptuel :

```text
239.0.0.1:5000
```

Architecture :

```text
                    239.0.0.1:5000
                          │
             ┌────────────┼────────────┐
             │            │            │
             ▼            ▼            ▼
           PC A         PC B         PC C
```

Lorsqu'un client envoie un message au groupe :

```text
Alice
  │
  ▼
UDP
  │
  ▼
239.0.0.1:5000
  │
  ├────► Bob
  ├────► Charlie
  └────► David
```

Le système n'a donc pas besoin de maintenir une connexion TCP individuelle avec chaque utilisateur.

---

# 10. TTL multicast

Le système utilise :

```text
TTL = 1
```

Le TTL est configuré au niveau du socket multicast.

Son objectif est de limiter la portée des paquets multicast.

```text
Application
     │
     ▼
Multicast Packet
     │
     ▼
TTL = 1
     │
     ▼
Réseau local
```

Cette configuration correspond à la garantie G-07.

---

# 11. Découverte des utilisateurs

Lorsqu'un utilisateur démarre l'application :

```text
Client A
   │
   ▼
Création socket
   │
   ▼
Rejoint multicast
   │
   ▼
Envoie discovery
   │
   ▼
Groupe multicast
   │
   ├────► Client B
   ├────► Client C
   └────► Client D
```

Les autres clients reçoivent l'annonce.

Le système peut alors mettre à jour son état local :

```text
Utilisateurs connus
├── Alice
├── Bob
└── Charlie
```

Le format exact du message `discovery` doit être défini dans la spécification du protocole.

---

# 12. Protocole applicatif

UDP transporte uniquement des octets.

Il ne connaît pas les concepts :

* `message` ;
* `discovery` ;
* `leave` ;
* `pseudo` ;
* `timestamp`.

Ces concepts appartiennent au protocole applicatif défini par le projet.

Architecture :

```text
┌──────────────────────┐
│ Application Protocol│
│       JSON           │
├──────────────────────┤
│         UDP          │
├──────────────────────┤
│          IP          │
├──────────────────────┤
│     Wi-Fi / Ethernet │
└──────────────────────┘
```

---

# 13. Types de messages

Le protocole définit trois types principaux :

```text
message
discovery
leave
```

Le récepteur effectue conceptuellement :

```text
Paquet reçu
     │
     ▼
Désérialisation
     │
     ▼
Quel est le type ?
     │
 ┌───┼─────────┐
 ▼   ▼         ▼
msg discovery leave
 │     │         │
 ▼     ▼         ▼
Chat  Présence  Départ
```

---

# 14. Message utilisateur

Format défini dans la spécification :

```json
{
    "type": "message",
    "pseudo": "Alice",
    "content": "Bonjour tout le monde !",
    "timestamp": "2026-08-24T14:23:45Z"
}
```

Le message possède donc :

* un type ;
* un pseudo ;
* un contenu ;
* un timestamp.

---

# 15. Modèle interne

Le modèle logique d'un message est :

```rust
struct Message {
    pseudo: String,
    contenu: String,
    time: Date<Utc>,
}
```

Il faut distinguer le modèle interne du format réseau.

```text
Message interne
      │
      ▼
Conversion
      │
      ▼
Message réseau
      │
      ▼
JSON
```

Le JSON représente le protocole de transport.

Le `struct Message` représente le modèle interne de l'application.

---

# 16. Identité d'un utilisateur

Le système utilise actuellement un pseudo.

Cependant, architecturalement, il faut distinguer :

```text
Pseudo
```

et :

```text
Identifiant unique du nœud
```

Exemple conceptuel :

```text
NodeId = 8f31...
Pseudo = Alice
```

Le pseudo sert à l'affichage.

Le `NodeId` permettrait d'identifier de manière unique une instance.

Cette distinction devient importante pour :

* éviter les collisions de pseudo ;
* identifier un utilisateur dans `leave` ;
* détecter les doublons ;
* gérer une future messagerie privée.

Ce point doit être ajouté à la spécification si cette fonctionnalité est retenue.

---

# 17. Composant Security

Le module Security est responsable de la protection des messages.

Flux d'envoi :

```text
Message clair
     │
     ▼
Security
     │
     ▼
Chiffrement
     │
     ▼
Message chiffré
     │
     ▼
Network
```

Flux de réception :

```text
Network
     │
     ▼
Message chiffré
     │
     ▼
Security
     │
     ▼
Déchiffrement
     │
     ▼
Message clair
```

---

# 18. Chiffrement AES

Le choix architectural actuel est un chiffrement symétrique basé sur AES.

Le principe est :

```text
              même clé
            ┌───────────┐
            │           │
Message ──► AES       AES ◄── Message chiffré
            │           │
            ▼           ▼
        Chiffré       Clair
```

L'émetteur et le récepteur doivent posséder une clé compatible.

La spécification actuelle indique que la clé peut être partagée manuellement.

---

# 19. Gestion des clés

Le chiffrement et la gestion des clés sont deux problèmes distincts.

```text
Chiffrement
    │
    └── Comment protéger les données ?

Gestion des clés
    │
    └── Comment obtenir et partager la clé ?
```

Le projet actuel définit principalement le mécanisme de chiffrement.

La gestion des clés devra être spécifiée plus précisément si le projet évolue.

---

# 20. Composant History

Le module History est responsable de la persistance.

Flux :

```text
Message reçu
     │
     ▼
History
     │
     ├── Mémoire
     │
     └── Disque
```

Au démarrage :

```text
Disque
   │
   ▼
History.load()
   │
   ▼
Mémoire
```

Pendant l'exécution :

```text
Message
   │
   ▼
History.add()
   │
   ▼
Mémoire
   │
   ▼
Disque
```

---

# 21. Limite de 100 messages

Le système doit conserver les 100 derniers messages.

Conceptuellement :

```text
M1 M2 M3 ... M98 M99 M100
```

Lorsqu'un nouveau message arrive :

```text
M1 M2 M3 ... M99 M100 M101
```

Le système retire alors :

```text
M1
```

pour conserver :

```text
M2 M3 ... M100 M101
```

Cela correspond à une collection bornée.

Une implémentation possible serait un buffer circulaire, mais le document actuel ne fixe pas encore cette décision.

---

# 22. Persistance

L'historique doit survivre au redémarrage.

Cycle :

```text
             APPLICATION
                  │
                  ▼
              Message
                  │
                  ▼
              History
                  │
                  ▼
                Fichier
                  │
                  │
             redémarrage
                  │
                  ▼
                Fichier
                  │
                  ▼
              History
                  │
                  ▼
              Application
```

Le choix actuel privilégie un fichier simple plutôt qu'une base SQLite, car le volume de données est faible.

---

# 23. Architecture des threads

Le système doit continuer à recevoir des messages pendant que l'utilisateur saisit du texte.

Une séparation entre le thread principal et le thread réseau est donc proposée.

```text
                    APPLICATION
                         │
              ┌──────────┴──────────┐
              │                     │
              ▼                     ▼
       Thread principal        Thread réseau
              │                     │
              ▼                     ▼
          Interface              UDP recv
              │                     │
              ▼                     ▼
          stdin                 messages
```

Le thread réseau doit pouvoir écouter le socket indépendamment de la saisie utilisateur.

---

# 24. Communication entre threads

Une architecture Rust utilisant des channels peut être utilisée.

Exemple conceptuel :

```text
             UI THREAD
                 │
                 │ Message à envoyer
                 ▼
          ┌──────────────┐
          │    Channel   │
          └──────┬───────┘
                 │
                 ▼
          NETWORK THREAD
```

Et dans l'autre sens :

```text
NETWORK THREAD
       │
       │ Message reçu
       ▼
┌──────────────┐
│    Channel   │
└──────┬───────┘
       │
       ▼
   UI THREAD
```

En Rust, `std::sync::mpsc` peut servir à cette communication dans une implémentation simple.

---

# 25. Flux complet de démarrage

Lorsqu'une instance démarre :

```text
1. Lancement de l'application
          │
          ▼
2. Chargement de l'historique
          │
          ▼
3. Création du socket UDP
          │
          ▼
4. Configuration multicast
          │
          ▼
5. Configuration TTL = 1
          │
          ▼
6. Rejoindre le groupe multicast
          │
          ▼
7. Démarrer le thread réseau
          │
          ▼
8. Envoyer discovery
          │
          ▼
9. Attendre les interactions utilisateur
```

---

# 26. Flux complet d'envoi

Lorsqu'Alice écrit un message :

```text
Alice
  │
  ▼
Interface
  │
  ▼
Validation
  │
  ▼
Message métier
  │
  ▼
Sérialisation
  │
  ▼
Chiffrement
  │
  ▼
UDP
  │
  ▼
Multicast
  │
  ▼
LAN
```

Les autres nœuds reçoivent ensuite le paquet.

---

# 27. Flux complet de réception

Lorsqu'un message arrive :

```text
LAN
 │
 ▼
UDP socket
 │
 ▼
Bytes
 │
 ▼
Déchiffrement
 │
 ▼
Désérialisation JSON
 │
 ▼
Identification du type
 │
 ├──── message ────► History ───► UI
 │
 ├──── discovery ──► Presence ──► UI
 │
 └──── leave ──────► Presence ──► UI
```

---

# 28. Flux complet d'arrêt

Lorsqu'un utilisateur quitte normalement :

```text
Application
     │
     ▼
Créer événement leave
     │
     ▼
Envoyer multicast
     │
     ▼
Autres clients
     │
     ▼
Mise à jour présence
```

Puis :

```text
Arrêt
  │
  ▼
Arrêt du thread réseau
  │
  ▼
Fermeture du socket
  │
  ▼
Persistance éventuelle
  │
  ▼
Fin du processus
```

Le comportement en cas de fermeture brutale doit être traité séparément.

---

# 29. Gestion des erreurs

L'architecture doit également prévoir les erreurs.

Exemples :

```text
Socket impossible à créer
       │
       ▼
Erreur réseau
```

```text
Impossible de rejoindre multicast
       │
       ▼
Erreur configuration réseau
```

```text
JSON invalide
       │
       ▼
Paquet ignoré + journalisation
```

```text
Message vide
       │
       ▼
Erreur utilisateur
```

```text
Historique corrompu
       │
       ▼
Erreur persistence
```

Une règle importante :

> Une erreur provenant du réseau ne doit pas nécessairement faire planter toute l'application.

---

# 30. Organisation du code Rust

```text
src/
├── main.rs
│
├── app/
│   ├── mod.rs
│   └── chat.rs
│
├── network/
│   ├── mod.rs
│   ├── socket.rs
│   ├── multicast.rs
│   ├── discovery.rs
│   └── protocol.rs
│
├── security/
│   ├── mod.rs
│   └── crypto.rs
│
├── history/
│   ├── mod.rs
│   └── storage.rs
│
├── ui/
│   ├── mod.rs
│   └── terminal.rs
│
└── models/
    ├── mod.rs
    └── message.rs
```

La deuxième structure correspond mieux à l'architecture détaillée.

---

# 31. Responsabilités des modules Rust

## `main.rs`

Responsable du démarrage.

```text
main
 │
 ├── configuration
 ├── création application
 └── lancement
```

Il doit rester relativement léger.

---

## `app/chat.rs`

Responsable de l'orchestration du chat.

```text
Chat
 ├── send_message()
 ├── handle_message()
 ├── handle_discovery()
 └── handle_leave()
```

---

## `network/socket.rs`

Responsable du socket.

```text
Socket
 ├── bind()
 ├── configure()
 ├── send()
 └── receive()
```

---

## `network/multicast.rs`

Responsable du groupe multicast.

```text
Multicast
 ├── join()
 ├── leave()
 ├── send()
 └── receive()
```

---

## `network/discovery.rs`

Responsable de la découverte.

```text
Discovery
 ├── announce()
 ├── handle_discovery()
 └── handle_leave()
```

---

## `network/protocol.rs`

Responsable du protocole.

```text
Protocol
 ├── serialize()
 ├── deserialize()
 └── validate()
```

---

## `security/crypto.rs`

Responsable du chiffrement.

```text
Crypto
 ├── encrypt()
 └── decrypt()
```

---

## `history/storage.rs`

Responsable de la persistance.

```text
History
 ├── load()
 ├── add()
 ├── get_recent()
 └── save()
```

---

## `ui/terminal.rs`

Responsable du terminal.

```text
Terminal
 ├── read_input()
 ├── display_message()
 ├── display_error()
 └── display_presence()
```

---

# 32. Dépendances entre modules

La direction recommandée est :

```text
                    UI
                    │
                    ▼
                   APP
               /    │    \
              ▼     ▼     ▼
          NETWORK SECURITY HISTORY
              │
              ▼
             UDP
```

Il faut éviter autant que possible :

```text
Network ──► UI
History ──► UI
Security ──► UI
```

Les modules techniques ne doivent pas dépendre directement de la présentation.

---

# 33. Modèle de communication entre composants

Lors d'un envoi :

```text
UI
 │
 │ Message
 ▼
APP
 │
 │ Message
 ▼
SECURITY
 │
 │ Ciphertext
 ▼
NETWORK
 │
 ▼
UDP
```

Lors d'une réception :

```text
UDP
 │
 ▼
NETWORK
 │
 │ Ciphertext
 ▼
SECURITY
 │
 │ Message
 ▼
APP
 │
 ├────► HISTORY
 │
 └────► UI
```

---

# 34. Tests architecturaux

Chaque garantie doit être associée à un test.

| Garantie | Test                                       |
| -------- | ------------------------------------------ |
| G-01     | Test de découverte avec plusieurs machines |
| G-02     | Test de diffusion multicast                |
| G-03     | Test de persistance après redémarrage      |
| G-04     | Capture réseau avec Wireshark              |
| G-05     | Tentative d'envoi d'un message vide        |
| G-06     | Démarrage sans compte                      |
| G-07     | Vérification du TTL                        |

---

# 35. Test de découverte

```text
PC1 démarre
     │
     ▼
discovery
     │
     ▼
PC2
     │
     ▼
"PC1 est en ligne"
```

Résultat attendu :

```text
Découverte ≤ 5 secondes
```

---

# 36. Test de diffusion

```text
PC1
 │
 │ "Bonjour"
 ▼
Multicast
 │
 ├────► PC2
 └────► PC3
```

Résultat :

```text
PC2 → reçoit
PC3 → reçoit
```

---

# 37. Test de persistance

```text
Envoyer messages
       │
       ▼
Sauvegarder
       │
       ▼
Fermer
       │
       ▼
Redémarrer
       │
       ▼
Charger
       │
       ▼
Messages présents
```

---

# 38. Test de sécurité

Utiliser Wireshark :

```text
Application
    │
    ▼
Message
    │
    ▼
Chiffrement
    │
    ▼
Réseau
    │
    ▼
Wireshark
```

Le contenu du message ne doit pas être lisible en clair lorsque le chiffrement est activé.

---

# 39. Évolutivité

L'architecture permet plusieurs évolutions.

## GUI

```text
Terminal
   │
   └──► remplacé par GUI
```

Les autres modules peuvent rester identiques.

---

## SQLite

```text
History
   │
   ├── FileStorage
   │
   └── SQLiteStorage
```

La couche applicative peut continuer à travailler avec la même abstraction.

---

## Plusieurs salons

```text
239.0.0.1 → Général
239.0.0.2 → Développement
239.0.0.3 → Étudiants
```

---

## Messagerie privée

Le protocole pourrait ajouter un destinataire :

```text
sender
recipient
content
timestamp
```

La logique réseau devrait alors différencier :

```text
message de groupe
```

et :

```text
message privé
```

---

# 40. Limites architecturales actuelles

Le protocole UDP multicast implique plusieurs limites.

### Perte de paquets

UDP ne garantit pas la livraison.

```text
Alice
  │
  ▼
UDP
  │
  X
  │
Bob
```

Le système ne possède actuellement aucune garantie de retransmission.

### Ordre des messages

Deux messages peuvent potentiellement être reçus dans un ordre différent de leur ordre d'envoi.

### Duplication

Un paquet pourrait potentiellement être traité plusieurs fois.

### Déconnexion brutale

Si une machine disparaît sans envoyer `leave`, les autres clients peuvent conserver temporairement son état de présence.

Ces problématiques devront être traitées si le projet évolue vers un système plus robuste.

---

# 41. Points à spécifier avant l'implémentation finale

L'architecture générale est définie, mais plusieurs détails techniques doivent encore être verrouillés.

## Réseau

* adresse multicast ;
* port ;
* interface réseau ;
* taille maximale d'un paquet ;
* comportement en cas d'échec ;
* gestion des interfaces multiples.

## Protocole

* structure exacte de `message` ;
* structure exacte de `discovery` ;
* structure exacte de `leave` ;
* version du protocole ;
* identifiant unique d'un nœud ;
* validation des messages.

## Sécurité

* variante exacte d'AES ;
* mode de fonctionnement ;
* nonce/IV ;
* authentification du message ;
* stockage de la clé ;
* partage de la clé.

## Historique

* format du fichier ;
* emplacement ;
* gestion des erreurs ;
* stratégie de sauvegarde ;
* structure utilisée pour conserver les 100 derniers messages.

## Concurrence

* nombre de threads ;
* channels ;
* propriétaire du socket ;
* mécanisme d'arrêt ;
* synchronisation de l'historique.

## Résilience

* paquets perdus ;
* paquets dupliqués ;
* paquets hors protocole ;
* départ brutal ;
* changement de réseau ;
* plusieurs instances sur une même machine.

---

# 42. Architecture finale

L'architecture complète peut être résumée ainsi :

```text
                         UTILISATEUR
                              │
                              ▼
                    ┌─────────────────┐
                    │       UI        │
                    │    Terminal     │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  APPLICATION    │
                    │  Chat / Logic   │
                    └───────┬─────────┘
                            /|\
                           / | \
                          /  |  \
                         ▼   ▼   ▼
                ┌────────┐ ┌───────┐ ┌─────────┐
                │Network │ │Crypto │ │ History │
                └───┬────┘ └───────┘ └─────────┘
                    │
                    ▼
              ┌──────────────┐
              │ UDP Multicast│
              │   TTL = 1    │
              └──────┬───────┘
                     │
                     ▼
                  LAN
                     │
            ┌────────┼────────┐
            ▼        ▼        ▼
           PC1      PC2      PC3
```

---

# 43. Flux global final

## Envoi

```text
Utilisateur
    ↓
UI
    ↓
Application
    ↓
Validation
    ↓
Modèle Message
    ↓
Sérialisation
    ↓
Chiffrement
    ↓
Network
    ↓
UDP
    ↓
Multicast
    ↓
LAN
```

## Réception

```text
LAN
    ↓
UDP
    ↓
Network
    ↓
Déchiffrement
    ↓
Désérialisation
    ↓
Validation
    ↓
Application
    ├──────────────► History
    │
    └──────────────► UI
                         ↓
                     Terminal
```

## Découverte

```text
Nouveau client
      ↓
Join multicast
      ↓
Discovery
      ↓
Multicast
      ↓
Clients existants
      ↓
Mise à jour présence
```

## Départ

```text
Client
   ↓
Leave
   ↓
Multicast
   ↓
Clients existants
   ↓
Suppression / mise à jour présence
```

---

# 44. Conclusion architecturale

Le système est un système distribué local basé sur UDP multicast.

Son architecture sépare clairement :

```text
Interface
    ↓
Application
    ↓
Network
    ↓
Security
    ↓
History
```

Les responsabilités sont reliées aux garanties :

```text
EX-01 → G-01 → Network
EX-02 → G-02 → Network
EX-03 → G-03 → History
EX-04 → G-04 → Security
EX-05 → G-05 → Interface
EX-06 → G-06 → Network / Configuration
EX-07 → G-07 → Network / Configuration
```

La conception complète suit donc :

```text
Exigences
    ↓
Garanties
    ↓
Responsabilités
    ↓
Composants
    ↓
Modules Rust
    ↓
Threads
    ↓
Channels
    ↓
Protocoles
    ↓
Sockets
    ↓
UDP Multicast
    ↓
Réseau LAN
```

L'étape suivante de conception doit être la **spécification formelle du protocole réseau**, car c'est elle qui permettra ensuite de définir précisément les structures Rust, les messages `message/discovery/leave`, les états d'un nœud, les diagrammes de séquence et les contrats entre les modules.
