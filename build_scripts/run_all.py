import subprocess
import sys
import os

if (not os.getenv("NEXT_PUBLIC_EXCLUDE_SIDECAR") == "true"):
    if "win32" in sys.platform:
        cwd = os.getcwd().replace("\\", "/")

        subprocess.run(["C:/msys64/usr/bin/bash.exe", "-l", "-c", "(cd " + cwd + " && ./build_scripts/nrsc5.sh)"], check=True)
    else:
        subprocess.run(["sh", "./build_scripts/nrsc5.sh"], check=True)
else:
    # replace nrsc5 executable with an empty file if sidecar is disabled
    rust_target_command = "rustc -vV | sed -n 's|host: ||p'"
    process = subprocess.Popen(rust_target_command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    # get the out and err
    stdout, stderr = process.communicate()

    # decode the string
    rust_target_string = stdout.decode('utf-8').strip()

    open("./build/bin/nrsc5-" + rust_target_string, 'w').close()