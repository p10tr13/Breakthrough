param (
    [string]$InputFile = "report.tex",
    [string]$OutputDir = ".\output_pdf"
)

$AbsoluteOutputDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputDir)
$File = Get-Item -Path $InputFile -ErrorAction SilentlyContinue

if (-not $File) {
    Write-Host "Error: Cannot find file '$InputFile'." -ForegroundColor Red
    exit
}

$TexDir = $File.DirectoryName
$BaseName = $File.BaseName
$PdfFile = "$BaseName.pdf"

Push-Location
Set-Location -Path $TexDir

Write-Host "Working directory temporarily changed to: $TexDir" -ForegroundColor DarkGray
Write-Host "Starting compilation of $($File.Name)..." -ForegroundColor DarkGray

pdflatex -interaction=nonstopmode $File.Name *> $null
pdflatex -interaction=nonstopmode $File.Name *> $null

$ErrorsAndWarnings = $log2 | Select-String -Pattern "(!|Warning:|Error|Undefined|No file)"
if ($ErrorsAndWarnings) {
    Write-Host "`nLaTeX reported some issues:" -ForegroundColor Yellow
    $ErrorsAndWarnings | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    Write-Host "Please check your .tex file or the $BaseName.log file for details.`n" -ForegroundColor Yellow
}

if (Test-Path $PdfFile) {
    if (-not (Test-Path $AbsoluteOutputDir)) {
        New-Item -ItemType Directory -Path $AbsoluteOutputDir -Force | Out-Null
    }
    
    Move-Item -Path $PdfFile -Destination "$AbsoluteOutputDir\$PdfFile" -Force
    Write-Host "Created: $AbsoluteOutputDir\$PdfFile" -ForegroundColor Green
}
else {
    Write-Host "`nError: Compilation failed. PDF file was not created." -ForegroundColor Red
}

Write-Host "Cleaning up temporary files..." -ForegroundColor DarkGray

$Extensions = @(".aux", ".out", ".toc", ".fls", ".fdb_latexmk", ".synctex.gz", ".bbl", ".blg", ".nav", ".snm", ".vrb")

foreach ($Ext in $Extensions) {
    $TempFile = "$BaseName$Ext"
    if (Test-Path $TempFile) {
        Remove-Item -Path $TempFile -Force
    }
}

if (Test-Path "texput.log") {
    Remove-Item "texput.log" -Force
}

Pop-Location
Write-Host "Script execution finished." -ForegroundColor Cyan