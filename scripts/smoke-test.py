#!/usr/bin/env python3
"""Lance Prisma installé sur un faux client Steam, débloque un succès et capture l'écran.

    python3 scripts/smoke-test.py <exécutable de Prisma> <exécutable steam_fixture> <dossier de sortie>

Les données et les secrets restent isolés dans un dossier temporaire : rien ne touche au compte de la machine.
"""

import json
import os
import platform
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# Le premier lancement d'un exécutable fraîchement installé peut être long, le temps que l'antivirus l'examine.
STARTUP_TIMEOUT = 120
UNLOCKED_TROPHY = "Fond de l'océan"
FIXTURE_GAMES = 3


def main() -> int:
    # La console Windows n'écrit pas l'UTF-8 par défaut.
    sys.stdout.reconfigure(encoding="utf-8")
    app, fixture, output = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    output.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="prisma-smoke-"))
    data, steam = work / "data", work / "steam"
    subprocess.run([fixture, steam], check=True)

    log = (output / "prisma.log").open("w", encoding="utf-8")
    environment = {**os.environ, "PRISMA_DATA_DIR": str(data), "PRISMA_STEAM_ROOT": str(steam)}
    process = subprocess.Popen([app], env=environment, stdout=log, stderr=subprocess.STDOUT)
    failures = []
    try:
        if wait_for_library(process, data / "library.json"):
            exercise(fixture, steam, output)
        elif process.poll() is None:
            failures.append(f"{stored_games(data / 'library.json')} jeu(x) enregistré(s) au lieu de {FIXTURE_GAMES}")
        if process.poll() is not None:
            failures.append(f"l'app s'est arrêtée (code {process.returncode})")
    finally:
        process.kill()
        process.wait()
        log.close()

    text = (output / "prisma.log").read_text(encoding="utf-8", errors="replace")
    if UNLOCKED_TROPHY not in text:
        failures.append(f"le succès « {UNLOCKED_TROPHY} » n'a pas été annoncé")
    failures += [f"erreur dans le journal : {line}" for line in text.splitlines() if "[ERROR]" in line]
    return report(failures, output)


def wait_for_library(process: subprocess.Popen, library: Path) -> bool:
    """L'app est prête quand elle a enregistré les jeux du faux client ; l'interface a encore besoin d'un instant."""
    deadline = time.monotonic() + STARTUP_TIMEOUT
    while time.monotonic() < deadline and process.poll() is None:
        if stored_games(library) == FIXTURE_GAMES:
            time.sleep(5)
            return True
        time.sleep(1)
    return False


def exercise(fixture: Path, steam: Path, output: Path) -> None:
    capture(output / "1-bibliotheque.png")
    subprocess.run([fixture, steam, "unlock"], check=True)
    # Le succès s'affiche d'abord, puis le platine qu'il complète, une fois la première carte refermée.
    time.sleep(3)
    capture(output / "2-notification-succes.png")
    time.sleep(6)
    capture(output / "3-notification-platine.png")
    time.sleep(3)


def stored_games(library: Path) -> int:
    try:
        return len(json.loads(library.read_text(encoding="utf-8")).get("games", []))
    except (OSError, ValueError):
        return 0


def capture(path: Path) -> None:
    system = platform.system()
    if system == "Windows":
        script = (
            "Add-Type -AssemblyName System.Windows.Forms,System.Drawing;"
            "$b=[System.Windows.Forms.SystemInformation]::VirtualScreen;"
            "$i=New-Object System.Drawing.Bitmap $b.Width,$b.Height;"
            "[System.Drawing.Graphics]::FromImage($i).CopyFromScreen($b.Left,$b.Top,0,0,$i.Size);"
            f"$i.Save('{path}')"
        )
        command = ["powershell", "-NoProfile", "-Command", script]
    elif system == "Darwin":
        command = ["screencapture", "-x", str(path)]
    else:
        command = ["import", "-window", "root", str(path)]
    if subprocess.run(command).returncode != 0:
        print(f"Capture impossible : {path.name}")


def report(failures: list[str], output: Path) -> int:
    print((output / "prisma.log").read_text(encoding="utf-8", errors="replace"))
    for failure in failures:
        print(f"ÉCHEC : {failure}")
    if not failures:
        print("✅ Prisma démarre, enregistre la bibliothèque et annonce le succès débloqué.")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
