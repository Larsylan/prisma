# Paquets

Fichiers de publication de Prisma dans les gestionnaires de paquets, à mettre à jour à chaque version (numéro, adresses, sommes SHA-256 de `SHA256SUMS.txt`).

| Gestionnaire | Fichiers | Publication |
|---|---|---|
| AUR (Arch Linux) | `aur/PKGBUILD` | Dépôt `ssh://aur@aur.archlinux.org/prisma-trophies-bin.git`, avec `.SRCINFO` (à régénérer par `makepkg --printsrcinfo` après chaque modification du PKGBUILD) |
| winget (Windows) | `winget/*.yaml` | Pull request dans `microsoft/winget-pkgs`, dossier `manifests/l/Larsylan/Prisma/<version>/` |

Le paquet Arch s'appelle `prisma-trophies-bin` : le nom `prisma` est déjà pris par l'outil de base de données du même nom.
