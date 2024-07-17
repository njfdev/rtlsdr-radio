import subprocess
import sys
import os

if "win32" in sys.platform:
    cwd = os.getcwd()

    subprocess.run(["C:/msys64/usr/bin/bash.exe", "-l", "-c", "(cd " + cwd + " && ./build_scripts/nrsc5.sh)"], check=True)
else:
    subprocess.run(["sh", "./build_scripts/nrsc5.sh"], check=True)
