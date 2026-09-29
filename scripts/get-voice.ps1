# Downloads the Piper text-to-speech program and the voices into assets/piper/.
# These files are big, so they aren't stored in git. Run this once after cloning:
#
#   powershell -ExecutionPolicy Bypass -File scripts\get-voice.ps1
#
# Add other voices by name (listen to samples at https://rhasspy.github.io/piper-samples/):
#
#   powershell -ExecutionPolicy Bypass -File scripts\get-voice.ps1 -Voices en_US-ryan-medium
#
# The game's voice menu (the "Voice" button on "Who's playing?") lists every voice in
# assets/piper/voices.

param(
    [string[]]$Voices = @(
        "en_US-lessac-medium",
        "en_US-amy-medium",
        "en_US-kristin-medium",
        "en_US-hfc_female-medium",
        "en_US-ljspeech-high"
    )
)
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue" # makes Invoke-WebRequest much faster

$root = Join-Path $PSScriptRoot "..\assets\piper"
$voiceDir = Join-Path $root "voices"
New-Item -ItemType Directory -Force $voiceDir | Out-Null

# 1. The Piper program.
if (-not (Test-Path "$root\piper\piper.exe")) {
    Write-Host "Downloading Piper..."
    $zip = "$root\piper.zip"
    Invoke-WebRequest "https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_windows_amd64.zip" -OutFile $zip
    Expand-Archive $zip -DestinationPath $root -Force
    Remove-Item $zip
}

# An older setup kept a single voice as assets/piper/voice.onnx: move it into voices/.
if ((Test-Path "$root\voice.onnx") -and -not (Test-Path "$voiceDir\en_US-lessac-medium.onnx")) {
    Move-Item "$root\voice.onnx" "$voiceDir\en_US-lessac-medium.onnx"
    Move-Item "$root\voice.onnx.json" "$voiceDir\en_US-lessac-medium.onnx.json"
}

# 2. The voices (each is a model file plus its settings file).
foreach ($voice in $Voices) {
    $lang, $name, $quality = $voice -split "-"
    $base = "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/$($lang.Split('_')[0])/$lang/$name/$quality/$voice"
    foreach ($ext in @(".onnx", ".onnx.json")) {
        if (-not (Test-Path "$voiceDir\$voice$ext")) {
            Write-Host "Downloading voice $voice$ext..."
            Invoke-WebRequest "$base$ext" -OutFile "$voiceDir\$voice$ext"
        }
    }
}

Write-Host "Done! Voices installed in assets\piper\voices."
