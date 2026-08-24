# Protocole — TchatLAN

## 1. Objectif de ce document

Spécifier précisément le format des messages échangés, à deux niveaux : la découverte des pairs (avant connexion) et l'échange applicatif (après connexion QUIC établie). Ce document doit permettre d'implémenter le protocole sans ambiguïté et de le faire évoluer proprement.

## 2. Vue d'ensemble

Le protocole se déroule en deux temps :

1. **Découverte** : messages courts envoyés en clair sur une adresse multicast UDP, uniquement destinés à signaler la présence d'un pair et son adresse de connexion. Aucune donnée de conversation n'y transite.
2. **Session applicative** : une fois la connexion QUIC établie entre deux pairs, tous les messages (texte, contrôle, présence) sont chiffrés automatiquement par TLS 1.3 et suivent le format décrit plus bas.

## 3. Découverte par multicast

### 3.1 Paramètres réseau

| Paramètre | Valeur proposée |
|---|---|
| Adresse multicast | 239.255.42.99 (à confirmer, plage IPv4 multicast locale 239.0.0.0/8) |
| Port UDP | 45820 (à ajuster si conflit) |
| Fréquence d'annonce | toutes les 5 secondes |
| Timeout d'expiration d'un pair | 15 secondes sans annonce reçue |

### 3.2 Format du message d'annonce (Announce)

Sérialisé en JSON pour la simplicité et la lisibilité pendant le développement (un passage à `bincode` pourra être envisagé plus tard pour réduire la taille des paquets).

Champs du message :

- `type` : chaîne fixe `"announce"`.
- `peer_id` : identifiant unique du pair (UUID v4).
- `display_name` : nom d'affichage choisi par l'utilisateur.
- `quic_port` : port sur lequel ce pair écoute les connexions QUIC entrantes.
- `timestamp` : horodatage Unix de l'envoi du message, utilisé pour détecter les messages trop anciens.

L'adresse IP du pair n'est pas incluse dans le message : elle est déduite directement de l'adresse source du paquet UDP reçu.

### 3.3 Comportement à la réception

1. Ignorer les annonces provenant de son propre `peer_id` (auto-écoute évitée en comparant l'identifiant).
2. Si le `peer_id` est inconnu, ajouter le pair à la liste locale avec son adresse IP, son port QUIC et l'horodatage de réception.
3. Si le `peer_id` est déjà connu, mettre à jour l'horodatage de dernière annonce.
4. Une tâche périodique parcourt la liste des pairs et retire ceux dont le dernier horodatage dépasse le timeout défini.

## 4. Session applicative (après connexion QUIC)

### 4.1 Organisation des flux (streams)

Une connexion QUIC entre deux pairs ouvre au minimum deux flux logiques :

- **Flux de contrôle** : messages de présence, accusés de réception, fermeture de session.
- **Flux de conversation** : messages de chat proprement dits.

Cette séparation évite qu'un gros volume de messages de contrôle ne retarde l'affichage des messages de chat, et inversement.

### 4.2 Format général d'un message applicatif

Tout message applicatif partage une enveloppe commune :

- `type` : type du message (voir section 4.3).
- `message_id` : identifiant unique du message (UUID v4), utilisé pour les accusés de réception.
- `sender_id` : `peer_id` de l'expéditeur.
- `timestamp` : horodatage Unix d'envoi.
- `payload` : contenu spécifique au type de message.

### 4.3 Types de messages

| Type | Rôle | Contenu du `payload` |
|---|---|---|
| `HELLO` | Premier message envoyé après connexion QUIC, confirme l'identité applicative | `display_name` |
| `MESSAGE` | Message de chat texte | `content` (texte du message) |
| `ACK` | Accusé de réception d'un message | `message_id` du message accusé |
| `PING` / `PONG` | Vérification que la connexion est toujours active | vide |
| `BYE` | Signale une déconnexion volontaire et propre | vide |
| `FILE_OFFER` (réservé, extension future) | Proposition de transfert de fichier, réutilisant le moteur de Toolé | métadonnées du fichier (nom, taille, empreinte SHA-256) |

### 4.4 Cycle de vie d'un message texte

1. L'utilisateur saisit un message dans l'interface.
2. L'application construit un message de type `MESSAGE`, l'enregistre localement en base avec un statut "envoyé".
3. Le message est sérialisé et envoyé sur le flux de conversation QUIC vers le pair destinataire.
4. À réception, le pair distant désérialise le message, l'affiche, l'enregistre localement, puis renvoie un `ACK` référençant le `message_id`.
5. À réception de l'`ACK`, l'expéditeur met à jour le statut du message en base ("délivré").

### 4.5 Gestion de la présence

- Un message `PING` est envoyé automatiquement si aucun message n'a été échangé avec un pair pendant une durée définie (par exemple 10 secondes), pour vérifier que la connexion QUIC est toujours valide.
- L'absence de `PONG` après un délai raisonnable entraîne la fermeture de la connexion et le passage du pair en statut "hors ligne" dans l'interface (le pair pourra réapparaître via une nouvelle annonce multicast et une reconnexion).

### 4.6 Fermeture de session

Un pair qui se déconnecte volontairement envoie un message `BYE` avant de fermer la connexion QUIC, ce qui permet à l'autre pair de mettre à jour immédiatement le statut sans attendre l'expiration d'un timeout.

## 5. Évolutions futures du protocole

- **Chiffrement applicatif additionnel** : au-delà du chiffrement de transport TLS 1.3, ajouter un chiffrement de bout en bout au niveau applicatif (par exemple via des clés échangées lors du `HELLO`) pour se prémunir d'un attaquant capable d'intercepter le trafic avant le handshake TLS.
- **Groupes de discussion** : introduire un `group_id` dans l'enveloppe des messages et un mécanisme de diffusion à plusieurs pairs simultanément.
- **Partage de fichiers** : réactiver le type `FILE_OFFER` et réutiliser directement le moteur de transfert chunké avec vérification SHA-256 déjà construit pour Toolé.
- **Accusés de lecture distincts des accusés de réception** : différencier "message reçu par le pair" et "message lu par l'utilisateur".
