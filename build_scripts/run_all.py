import subprocess
import sys
import os
from pathlib import Path
import shutil
import glob

# prevent build errors on macOS Apple Silicon
os.environ.pop('IPHONEOS_DEPLOYMENT_TARGET', None)

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
build_dir.parent.mkdir(exist_ok=True)

orig_dir = os.getcwd()
os.chdir(build_dir)

# handle building required libs
subprocess.run(["cmake", "../build_scripts"], check=True)

subprocess.run(["cmake", "--build", "."], check=True)

# move all the files we need into dist
dist_dir = Path("./dist")
dist_dir.parent.mkdir(exist_ok=True)

resources_dir = dist_dir.joinpath("resources")
resources_dir.parent.mkdir(exist_ok=True)

out_dir = Path("./out")

# copy the files into dist

# copy dlls
dll_files = glob.glob(str(out_dir.joinpath("bin/*.dll")))
for dll in dll_files:
    shutil.copy(dll, dist_dir)

# copy include dir
shutil.copytree(out_dir.joinpath("include"), resources_dir.joinpath("include"), dirs_exist_ok=True)

# copy lib dir
shutil.copytree(out_dir.joinpath("lib"), resources_dir.joinpath("lib"), dirs_exist_ok=True)

# update .pc files with new path
pc_files = glob.glob(str(resources_dir.joinpath("lib/pkgconfig/*.pc")))
for pc in pc_files:
    with open(pc, 'r') as f:
        lines = f.readlines()
    
    lines[0] = "prefix=" + str(resources_dir.absolute()) + "\n"

    with open(pc, 'w') as f:
        f.writelines(lines)

os.chdir(orig_dir)