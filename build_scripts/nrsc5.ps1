# Note: FAAD2 is turned off because it causes issues in GitHub Actions and is not needed

# Create build directory
New-Item -ItemType Directory -Force -Path build
Set-Location build

# Clone the repository
git clone https://github.com/theori-io/nrsc5.git
Set-Location nrsc5

# Reset to specific commit
git reset --hard a57dd5b5f93e08d9ccdeb5f6b670a16d7566f8f1

# Create build directory and navigate into it
New-Item -ItemType Directory -Force -Path build
Set-Location build

# Run cmake with specified options
cmake -DUSE_FAAD2=OFF ../
make

# Navigate back to the root directory and create bin directory
Set-Location ../..
New-Item -ItemType Directory -Force -Path bin

# Move the built binary to the bin directory with a specific name
$rustcVersion = rustc -vV | Select-String -Pattern 'host: (.+)' | ForEach-Object { $_.Matches[0].Groups[1].Value }
Move-Item -Force -Path ./nrsc5/build/src/nrsc5 -Destination ./bin/nrsc5-$rustcVersion

# Move to original directory
Set-Location ../