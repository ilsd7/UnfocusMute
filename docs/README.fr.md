# UnfocusMute

[한국어](../README.md) | [English](README.en.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | Français | [Português](README.pt.md) | [हिन्दी](README.hi.md) | [العربية](README.ar.md)

UnfocusMute est une petite application légère pour la zone de notification Windows. Elle coupe automatiquement le son des jeux ou applications sélectionnés lorsqu’ils passent en arrière-plan.

Écrite en Rust et distribuée comme application portable, elle ne contrôle que les sessions audio que vous enregistrez. Quand une application revient au premier plan, UnfocusMute réactive uniquement les sessions qu’elle avait coupées elle-même, sans toucher aux coupures manuelles.

## Pratique Pour

- Passer souvent d’un jeu à un navigateur, une messagerie ou une fenêtre de travail
- Garder une application en arrière-plan silencieuse sans ouvrir le mélangeur de volume Windows
- Gérer les applications de type navigateur qui utilisent plusieurs processus avec le même `.exe`
- Utiliser un outil léger écrit en Rust, exécutable depuis un ZIP sans installateur

## Fonctionnalités

- Coupe automatiquement le son uniquement des applications enregistrées qui ne sont pas au premier plan
- Réactive l’audio seulement pour les sessions coupées par UnfocusMute
- Ajoute des cibles depuis la liste des applications en cours ou en saisissant un exécutable comme `game.exe`
- Regroupe par défaut les processus `.exe` identiques, utile pour les navigateurs avec plusieurs PID
- Permet l’enregistrement par PID avec `Afficher les PID`
- Fonctionne dans la zone de notification, avec pause, raccourci vers le fichier de configuration et protection contre les doubles lancements
- Sélection de la langue au premier lancement et changement immédiat entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी et العربية
- Configuration locale dans `%APPDATA%\UnfocusMute\config.json`
- Aucune requête réseau, aucun compte, aucune télémétrie ni fichier journal séparé

## Télécharger et Lancer

Téléchargez le ZIP Windows, extrayez-le, puis lancez `UnfocusMute.exe`. L’application est portable : aucun installateur n’est nécessaire, et vous n’avez pas besoin d’un runtime séparé, de Rust, de Visual Studio Build Tools ni de MinGW.

## Utilisation

1. Lancez UnfocusMute.
2. Sélectionnez une langue au premier lancement. English est sélectionné par défaut.
3. Ouvrez le jeu ou l’application à gérer.
4. Actualisez la liste des applications en cours, sélectionnez un élément, puis cliquez sur `Ajouter la sélection`.
5. Les entrées avec le même `.exe` sont regroupées par défaut.
6. Utilisez `Afficher les PID` uniquement si vous devez enregistrer une instance précise.
7. Fermer la fenêtre laisse UnfocusMute actif dans la zone de notification. Utilisez `Quitter` pour l’arrêter complètement.

## Réglages Par Défaut

Au premier lancement, vous pouvez choisir si UnfocusMute démarre automatiquement à l’ouverture de session Windows. Les nouvelles configurations désactivent le démarrage automatique par défaut, activent le démarrage réduit dans la zone de notification et réactivent le son des applications à la fermeture.

Cliquez sur `Langue` dans l’application pour passer immédiatement entre English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी et العربية. Le choix est enregistré automatiquement.

## Build Développeur

La cible de publication recommandée est `x86_64-pc-windows-msvc`.

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

Créer un ZIP de distribution :

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Le paquet est créé dans `dist\UnfocusMute-<version>-windows-x64.zip` et contient l’exécutable ainsi que `LICENSE`.

## Confidentialité

UnfocusMute stocke uniquement les noms de processus enregistrés, les PID facultatifs, la langue de l’interface, la position de la fenêtre et les préférences de démarrage dans un fichier de configuration local. La détection des sessions audio et le contrôle du son sont traités localement avec les API Windows CoreAudio.

## Licence

Apache License 2.0. Consultez [LICENSE](../LICENSE).
