<div align="center">
  <img src="../assets/app-icon.png" alt="Icône UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>Application légère pour la zone de notification Windows qui met automatiquement en sourdine les jeux et applications choisis lorsqu’ils ne sont plus au premier plan.</strong></p>

  <p>
    <a href="../README.md">English</a> · <a href="README_ko.md">한국어</a> · <a href="README_ja.md">日本語</a> · <a href="README_zh-CN.md">简体中文</a> · <a href="README_es.md">Español</a> · Français · <a href="README_pt.md">Português</a> · <a href="README_hi.md">हिन्दी</a> · <a href="README_ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=Build&logo=githubactions&logoColor=white" alt="Build status"></a>
    &nbsp;
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    &nbsp;
    <a href="../LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>Fonctionnement 100 % local &nbsp;·&nbsp; Aucun accès réseau &nbsp;·&nbsp; Aucune télémétrie &nbsp;·&nbsp; Aucune installation requise</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">Télécharger</a>
    · <a href="#utilisation">Utilisation</a>
    · <a href="#sécurité-et-confidentialité">Confidentialité</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute est une petite application légère pour la zone de notification Windows qui met automatiquement en sourdine les jeux et applications choisis lorsqu’ils passent en arrière-plan, puis rétablit leur son lorsqu’ils reviennent au premier plan.

- Compilée comme application native Rust, elle s’exécute sans environnement d’exécution séparé.
- L’exécutable fait environ 500 Ko.
- Elle ne se limite pas aux jeux : vous pouvez aussi enregistrer des applications classiques comme des navigateurs, des messageries, des lanceurs et des lecteurs multimédias.
- La mise en sourdine et le rétablissement du son ne s’appliquent qu’aux sessions qu’UnfocusMute a modifiées lui-même ; les sessions que vous aviez déjà mises en sourdine ne sont pas touchées.

<p align="center">
  <img src="../assets/screenshot_fr.png" width="600" alt="Fenêtre principale d’UnfocusMute">
</p>

---

## Utile quand

- Vous gardez un jeu ou une application en cours d’exécution et passez souvent à une autre fenêtre avec Alt+Tab.
- Vous voulez laisser silencieuse une application qui ne propose pas d’option de sourdine en arrière-plan.
- Vous voulez couper seulement le son en arrière-plan d’une application précise pendant que vous faites autre chose.

## Fonctionnalités

- Met automatiquement en sourdine les applications enregistrées lorsqu’elles passent en arrière-plan, puis rétablit leur son à leur retour au premier plan.
- Permet de choisir une application ayant une session audio, de la retrouver dans `Tous les processus` ou de saisir un nom comme `game.exe`.
- Enregistre une application entière par `.exe`, ou seulement l’instance en cours d’exécution par PID.
- Notes par application, état en temps réel et pause / reprise individuelle.
- Continue la surveillance depuis la zone de notification après la fermeture de la fenêtre, avec des actions pour ouvrir / masquer / mettre tout en pause / quitter.
- Paramètres pour `Démarrer réduit dans la zone de notification`, `Démarrage automatique à la connexion Windows` et `À la sortie, restaurer le son mis en sourdine par UnfocusMute`.
- Choix de la langue au premier démarrage, puis changement immédiat entre 9 langues dans l’application.

---

## Télécharger et lancer

Sous Windows 10/11, téléchargez le paquet ZIP puis extrayez-le pour lancer l’application.

| Paquet le plus récent |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [Fichier SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [Notes de version](https://github.com/ilsd7/UnfocusMute/releases/latest) |

Une fois le ZIP extrait, déplacez le dossier `UnfocusMute-windows-x64` à l’emplacement où vous souhaitez conserver l’application, puis lancez `UnfocusMute-v<version>.exe` depuis ce dossier.

UnfocusMute est une application autonome qui ne nécessite aucune installation. Vous n’avez pas non plus besoin d’installer Rust, Visual Studio Build Tools, MinGW ni d’autres outils de développement.

> **Remarque :** Comme les certificats de signature de code ont un coût, l’application est actuellement distribuée sans signature de code Windows. Windows SmartScreen ou un avertissement d’éditeur inconnu peut apparaître au premier lancement. Pour vérifier vous-même l’intégrité du fichier, consultez [Vérifier les fichiers de publication](#vérifier-les-fichiers-de-publication).

## À savoir avant utilisation

UnfocusMute s’appuie sur les noms de processus, les informations de fenêtre au premier plan et les sessions CoreAudio fournies par Windows.

Si un pilote, un réglage d’autorisation ou un logiciel de sécurité limite l’accès aux sessions, le contrôle de sourdine peut être partiellement limité.

La liste par défaut affiche uniquement les applications qui disposent déjà d’une session audio. Si une application n’a pas encore créé de session audio, passez à `Tous les processus` pour la retrouver parmi les processus `.exe` en cours. Utilisez `Sessions audio uniquement` pour revenir à la liste filtrée.

**Comportement de l’enregistrement par PID :** Windows ne fournit pas toujours le même PID pour une session audio et pour la fenêtre au premier plan. Pour compenser cela, UnfocusMute considère que l’application est revenue au premier plan lorsque le nom `.exe` du PID enregistré correspond au nom `.exe` de la fenêtre actuellement active.

Si plusieurs instances du même `.exe` sont ouvertes en même temps, il n’est donc pas toujours possible d’isoler parfaitement un PID précis. Dans ce cas, le son peut être rétabli lorsqu’une autre instance est au premier plan.

**Compatibilité anti-triche :** UnfocusMute n’injecte pas de code dans les jeux, ne lit pas la mémoire du jeu, n’intercepte pas les entrées et ne modifie pas les fichiers du jeu. Il utilise seulement les informations de processus/fenêtre au premier plan de Windows et les commandes de sourdine des sessions CoreAudio. Il devrait donc fonctionner sans problème avec la plupart des systèmes anti-triche, mais la compatibilité avec tous les systèmes anti-triche ne peut pas être garantie.

## Utilisation

1. Lancez UnfocusMute.
2. Choisissez la langue dans l’écran affiché au premier démarrage. La langue par défaut est l’anglais.
3. Lancez le jeu ou l’application à enregistrer.
4. Utilisez le champ `Rechercher un processus` pour trouver une application ou choisissez-la dans la liste, puis cliquez sur `Enregistrer`. Si l’application n’a pas encore créé de session audio, passez à `Tous les processus` pour parcourir la liste de tous les processus en cours. Utilisez `Sessions audio uniquement` pour revenir à la liste filtrée. Si elle n’est pas dans la liste, saisissez le nom `.exe` manuellement.
5. Si vous devez enregistrer seulement un PID précis, cliquez sur `Vue PID` et choisissez l’entrée concernée. Une entrée par PID n’est valable que pour l’instance actuellement ouverte ; si l’application redémarre avec un autre PID, sélectionnez-la à nouveau.
6. Faites un clic droit sur une application enregistrée pour modifier sa note ou utiliser `Pause`.
7. Ouvrez `Paramètres` en bas à gauche pour modifier le comportement.
8. Lorsque vous fermez la fenêtre, l’application reste dans la zone de notification et continue de surveiller les applications enregistrées. Cliquez sur `Quitter` pour l’arrêter complètement.

## Utiliser les notes des applications enregistrées

Si le nom du processus ne suffit pas à reconnaître l’application, faites un clic droit sur l’application enregistrée et choisissez `Modifier la note`. La note s’affiche au-dessus du nom du processus dans la liste, sans influencer la détection de l’application.

C’est utile lorsqu’un même lanceur de jeu ouvre plusieurs processus, ou lorsqu’un nom d’exécutable n’indique pas clairement son rôle.

- `htgame.exe - NTE`
- `game.exe (PID 21976) - client du serveur de test`

Les notes sont enregistrées localement avec les autres réglages dans `%APPDATA%\UnfocusMute\config.json`.

## Trouver le nom de l’exécutable

Si vous ne savez pas quel nom enregistrer, vérifiez dans le Gestionnaire des tâches le nom de l’exécutable qui se termine par `.exe`.

1. Lancez d’abord l’application à enregistrer.
2. Utilisez `Alt`+`Tab` ou `Windows`+`Tab` pour quitter l’application et revenir au bureau Windows.
3. Appuyez sur `Ctrl`+`Shift`+`Esc` pour ouvrir le Gestionnaire des tâches.
4. Triez la liste des processus par `CPU` afin de retrouver l’application que vous venez de lancer.
5. Faites un clic droit sur cet élément et ouvrez `Propriétés`.
6. Relevez le nom de l’exécutable se terminant par `.exe`, par exemple `game.exe`, puis ajoutez-le à UnfocusMute.

## Dépannage

Si une application n’apparaît pas dans la liste, ou si l’enregistrement par PID ne se comporte pas comme prévu, consultez d’abord [À savoir avant utilisation](#à-savoir-avant-utilisation) et [Trouver le nom de l’exécutable](#trouver-le-nom-de-lexécutable).

Si l’état en haut de la fenêtre passe à `À vérifier`, cliquez sur `Détails` pour consulter le message d’erreur détaillé.

Si le problème persiste, signalez-le dans GitHub Issues.

Si vous pensez qu’il s’agit d’une faille de sécurité, ne publiez pas les détails dans une Issue publique. Utilisez la procédure de signalement privé et consultez [SECURITY.md](../SECURITY.md) pour plus d’informations.

## Fichier de configuration

Pour consulter ou sauvegarder directement le fichier de configuration, cliquez sur `Ouvrir le dossier de configuration` dans Paramètres.

L’Explorateur de fichiers ouvre le dossier `%APPDATA%\UnfocusMute`, où les paramètres sont enregistrés.

Vous pouvez modifier directement le fichier de configuration, mais si son format est invalide et qu’il ne peut pas être lu, il est sauvegardé sous `config.invalid-<timestamp>.json`. Si le problème est détecté au démarrage de l’application, les paramètres sont restaurés aux valeurs par défaut ; s’il est détecté pendant l’exécution, un nouveau fichier de configuration est créé à partir des paramètres actuels de l’application.

---

## Sécurité et confidentialité

UnfocusMute fonctionne entièrement en local. L’application fonctionne normalement même sans connexion Internet et ne demande pas de droits administrateur. Elle n’effectue pas non plus de requêtes réseau automatiques, n’utilise pas de télémétrie, n’envoie pas de rapports de plantage, ne fait pas de journalisation distante et ne collecte pas de données.

La seule exception est le bouton `Dépôt GitHub` dans les paramètres : le dépôt GitHub du projet ne s’ouvre dans votre navigateur par défaut que lorsque vous cliquez dessus.

La détection des sessions audio et le contrôle de la sourdine utilisent uniquement les API Windows CoreAudio. UnfocusMute n’injecte pas de code dans les processus ciblés, ne lit pas leur mémoire et n’intercepte pas les entrées.

### Informations stockées

UnfocusMute stocke uniquement les réglages nécessaires à son fonctionnement dans `%APPDATA%\UnfocusMute\config.json`.

- Noms de processus enregistrés
- PID enregistrés directement
- Dernier état de sourdine des applications enregistrées
- Notes que vous écrivez
- Langue et paramètres choisis
- Position de la fenêtre

Ces informations ne sont envoyées nulle part.

Si vous activez le démarrage automatique à l’ouverture de session Windows, le chemin de l’exécutable actuel est aussi enregistré dans la valeur `UnfocusMute` sous `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

### Informations non stockées

UnfocusMute ne stocke pas l’historique d’utilisation, les journaux d’activité, les journaux d’erreurs, les données audio, les titres de fenêtres, les frappes au clavier ni aucune information qui n’est pas indiquée ci-dessus dans « Informations stockées ».

### Suppression

Pour supprimer tous les fichiers liés à l’application, supprimez le dossier `UnfocusMute-windows-x64`, puis le dossier `%APPDATA%\UnfocusMute`.

Si vous avez déjà activé le démarrage automatique, supprimez aussi la valeur `UnfocusMute` sous `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`.

---

## Vérifier les fichiers de publication

Il ne faut pas supposer que les fichiers mis en ligne sur GitHub Releases correspondent toujours au code source publié dans le dépôt.

Si les droits de publication sont détournés ou si un compte est compromis, des fichiers compilés depuis un autre code ou des fichiers modifiés pourraient être ajoutés à une version publiée.

Par transparence, UnfocusMute fournit une méthode permettant de vérifier que les fichiers mis en ligne sur GitHub Releases sont bien des artefacts officiels générés par GitHub Actions à partir du code source du tag correspondant dans ce dépôt.

L’archive ZIP publiée et le fichier de somme de contrôle SHA-256 sont générés automatiquement par GitHub Actions, et chaque fichier est fourni avec une attestation de provenance de compilation.

Les commandes ci-dessous permettent de vérifier que le ZIP téléchargé a été généré par la compilation officielle de ce dépôt.

```powershell
gh attestation verify .\UnfocusMute-windows-x64.zip -R ilsd7/UnfocusMute
gh attestation verify .\UnfocusMute-windows-x64.zip.sha256 -R ilsd7/UnfocusMute
```

---

## Compiler depuis le code source

La cible recommandée pour les versions publiées est `x86_64-pc-windows-msvc`.

Prérequis :

- Rust stable
- Visual Studio Build Tools 2022 ou Visual Studio 2022
- SDK Windows 10/11

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

Les compilations destinées aux versions publiées sont configurées pour réduire la taille du binaire. Le profil de release de `Cargo.toml` retire les symboles, active LTO, utilise une seule codegen unit, définit `panic = "abort"` et optimise pour la taille.

Exécutable :

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

ZIP de distribution :

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Actualiser les avis de licences tierces :

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

Le résultat est créé dans `dist\UnfocusMute-windows-x64.zip`, avec le fichier de vérification SHA-256 `dist\UnfocusMute-windows-x64.zip.sha256` au même emplacement. Le ZIP contient l’exécutable avec numéro de version (`UnfocusMute-v<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md` et les README `.txt` traduits dans `docs`.

---

## Licence

Apache License 2.0. Consultez [LICENSE](../LICENSE) pour les détails.

Consultez [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) pour les avis de licences des crates Rust tierces.
