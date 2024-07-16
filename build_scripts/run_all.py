import subprocess
import sys

if "win32" in sys.platform:
    subprocess.run(["C:/msys64/usr/bin/bash.exe", "-c", "./build_scripts/nrsc5.sh"], check=True)
else:
    subprocess.run(["sh", "./build_scripts/nrsc5.sh"], check=True)
