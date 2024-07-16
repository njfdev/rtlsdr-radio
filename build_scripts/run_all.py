import subprocess
import sys

if "win32" in sys.platform:
    subprocess.run(["pwsh", ".\\build_scripts\\nrsc5.ps1"], check=True)
else:
    subprocess.run(["sh", "./build_scripts/nrsc5.sh"], check=True)
