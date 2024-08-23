import subprocess
import sys
import os
from pathlib import Path

# handle building nrsc5
if (not os.getenv("VITE_EXCLUDE_SIDECAR") == "true"):
    if "win32" in sys.platform:
        cwd = os.getcwd().replace("\\", "/")

        subprocess.run(["C:/msys64/usr/bin/bash.exe", "-l", "-c", "(cd " + cwd + " && ./build_scripts/nrsc5.sh)"], check=True)
    else:
        subprocess.run(["sh", "./build_scripts/nrsc5.sh"], check=True)
else:
    file = Path("./build/bin/nrsc5-" + os.getenv("TAURI_ENV_TARGET_TRIPLE") + (".exe" if "win32" in sys.platform else ""))
    file.parent.mkdir(parents=True, exist_ok=True)
    file.write_bytes(b"")


build_dir = Path("./build")
file.parent.mkdir(exist_ok=True)

orig_dir = os.getcwd()
os.chdir(build_dir)

# handle building required libs
subprocess.run(["cmake", "../build_scripts"], check=True)

# prevent build errors on macOS Apple Silicon
os.environ.pop('IPHONEOS_DEPLOYMENT_TARGET', None)

subprocess.run(["cmake", "--build", "."], check=True)

os.chdir(orig_dir)