# Downloads the Piper text-to-speech program and a voice into assets/piper/.
# These files are big, so they aren't stored in git. Run this once after cloning:
#
#   powershell -ExecutionPolicy Bypass -File scripts\get-voice.ps1
#
# Try a different voice (listen to samples at https://rhasspy.github.io/piper-samples/):
#
#   powershell -ExecutionPolicy Bypass -File scripts\get-voice.ps1 -Voice en_US-amy-medium

param(
    [string]$Voice = "en_US-lessac-medium"
)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue" # makes Invoke-WebRequest much faster

$root = Join-Path $PSScriptRoot "..\assets\piper"
New-Item -ItemType Directory -Force $root | Out-Null

# 1. The Piper program.
if (-not (Test-Path "$root\piper\piper.exe")) {
    Write-Host "Downloading Piper..."
    $zip = "$root\piper.zip"
    Invoke-WebRequest "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_windows_amd64.zip" -OutFile $zip
    Expand-Archive $zip -DestinationPath $root -Force
    Remove-Item $zip
}

# 2. The voice (a model file plus its settings file).
$lang, $name, $quality = $Voice -split "-"
$base = "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/$($lang.Split('_')[0])/$lang/$name/$quality/$Voice"
foreach ($ext in @(".onnx", ".onnx.json")) {
    if (-not (Test-Path "$root\voice$ext")) {
        Write-Host "Downloading voice $Voice$ext..."
        Invoke-WebRequest "$base$ext" -OutFile "$root\voice$ext"
    }
}

# Clear clips made with a previous voice so everything uses the new one.
Remove-Item -Recurse -Force (Join-Path $PSScriptRoot "..\assets\voice-cache") -ErrorAction SilentlyContinue

Write-Host "Done! Voice installed in assets\piper."
