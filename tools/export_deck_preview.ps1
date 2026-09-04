$ErrorActionPreference = "Continue"

$repo = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$deck = Join-Path $repo "docs\13forge-final-nontechnical-hackathon-deck.pptx"
$out = Join-Path $repo "docs\deck-assets\pptx-preview-nontechnical"

New-Item -ItemType Directory -Force -Path $out | Out-Null

$pp = New-Object -ComObject PowerPoint.Application
$pp.Visible = -1
$pres = $pp.Presentations.Open($deck, $true, $false, $false)
$pres.Export($out, "PNG", 1440, 810)

try { $pres.Close() } catch {}
try { $pp.Quit() } catch {}

Get-ChildItem $out | Select-Object FullName,Length
