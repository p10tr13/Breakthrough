# Tex Compilation

To compile the documents and clean up temporary files automatically, use the `CompileTex.ps1` script:

```ps1
cd .\docs\

# Compile the outline
.\CompileTex.ps1 -InputFile .\overleaf\outline.tex -OutputDir .

# Compile the main report
.\CompileTex.ps1 -InputFile .\overleaf\report.tex -OutputDir .

# Compile the presentation
.\CompileTex.ps1 -InputFile .\overleaf\presentation.tex -OutputDir .
```