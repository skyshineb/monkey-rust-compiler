$ErrorActionPreference = "Stop"

function Get-EnvOrDefault {
    param(
        [string]$Name,
        [string]$Default
    )

    $value = [Environment]::GetEnvironmentVariable($Name)
    if ([string]::IsNullOrWhiteSpace($value)) {
        return $Default
    }
    return $value
}

function Format-Decimal {
    param([double]$Value)

    return [string]::Format([System.Globalization.CultureInfo]::InvariantCulture, "{0:N2}", $Value)
}

function Parse-CommandLine {
    param([string]$CommandLine)

    $tokens = New-Object System.Collections.Generic.List[string]
    $current = New-Object System.Text.StringBuilder
    $inSingle = $false
    $inDouble = $false

    foreach ($ch in $CommandLine.ToCharArray()) {
        if ($ch -eq "'" -and -not $inDouble) {
            $inSingle = -not $inSingle
            continue
        }
        if ($ch -eq '"' -and -not $inSingle) {
            $inDouble = -not $inDouble
            continue
        }
        if ([char]::IsWhiteSpace($ch) -and -not $inSingle -and -not $inDouble) {
            if ($current.Length -gt 0) {
                $tokens.Add($current.ToString())
                $null = $current.Clear()
            }
            continue
        }
        $null = $current.Append($ch)
    }

    if ($inSingle -or $inDouble) {
        throw "MONKEY_JAVA_REF_CMD has unterminated quotes."
    }

    if ($current.Length -gt 0) {
        $tokens.Add($current.ToString())
    }

    if ($tokens.Count -eq 0) {
        throw "MONKEY_JAVA_REF_CMD is empty."
    }

    return $tokens.ToArray()
}

function Invoke-BenchRound {
    param(
        [ValidateSet("rust", "java")]
        [string]$Runtime,
        [string]$Program,
        [string[]]$BaseArgs,
        [string]$BenchPath
    )

    $args = @($BaseArgs) + @("run", $BenchPath)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $output = & $Program @args 2>&1
    $exitCode = $LASTEXITCODE
    $sw.Stop()

    if ($exitCode -ne 0) {
        Write-Host ""
        Write-Host "Benchmark command failed for runtime=$Runtime file=$BenchPath exit=$exitCode"
        if ($output) {
            $output | ForEach-Object { Write-Host $_ }
        }
        throw "Benchmark failed."
    }

    return $sw.Elapsed.TotalMilliseconds
}

function Measure-RuntimeForBench {
    param(
        [ValidateSet("rust", "java")]
        [string]$Runtime,
        [string]$Program,
        [string[]]$BaseArgs,
        [string]$BenchPath,
        [int]$Rounds
    )

    $times = New-Object System.Collections.Generic.List[double]
    for ($i = 1; $i -le $Rounds; $i++) {
        $ms = Invoke-BenchRound -Runtime $Runtime -Program $Program -BaseArgs $BaseArgs -BenchPath $BenchPath
        $times.Add($ms)
        Write-Host ("{0,-5} {1,-14} round {2}: {3,8} ms" -f $Runtime, [System.IO.Path]::GetFileName($BenchPath), $i, (Format-Decimal $ms))
    }

    return [PSCustomObject]@{
        Runtime = $Runtime
        Bench = [System.IO.Path]::GetFileNameWithoutExtension($BenchPath)
        AvgMs = ($times | Measure-Object -Average).Average
        MinMs = ($times | Measure-Object -Minimum).Minimum
        MaxMs = ($times | Measure-Object -Maximum).Maximum
        Rounds = $Rounds
    }
}

$rootDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$benchDir = Join-Path $rootDir "bench"

$roundsRaw = Get-EnvOrDefault -Name "BENCH_ROUNDS" -Default "3"
$parsedRounds = 0
if (-not [int]::TryParse($roundsRaw, [ref]$parsedRounds)) {
    throw "BENCH_ROUNDS must be an integer, got '$roundsRaw'."
}
$rounds = $parsedRounds
if ($rounds -lt 1) {
    throw "BENCH_ROUNDS must be >= 1."
}

$benchNames = @("b1", "b2", "b3", "b4", "b5")
$benchFilter = [Environment]::GetEnvironmentVariable("BENCH_FILTER")
if (-not [string]::IsNullOrWhiteSpace($benchFilter)) {
    $benchNames = $benchNames | Where-Object { $_ -like "*$benchFilter*" }
}

if ($benchNames.Count -eq 0) {
    throw "BENCH_FILTER '$benchFilter' matched no benchmarks."
}

$benchPaths = @()
foreach ($name in $benchNames) {
    $path = Join-Path $benchDir "$name.monkey"
    if (-not (Test-Path $path)) {
        throw "Missing benchmark file: $path"
    }
    $benchPaths += $path
}

$rustBin = Get-EnvOrDefault -Name "MONKEY_RUST_BIN" -Default (Join-Path $rootDir "target\release\monkey.exe")
if (-not (Test-Path $rustBin)) {
    throw "Rust binary not found: $rustBin. Build with 'cargo build --release' or set MONKEY_RUST_BIN."
}

$javaCmdRaw = [Environment]::GetEnvironmentVariable("MONKEY_JAVA_REF_CMD")
$hasJava = -not [string]::IsNullOrWhiteSpace($javaCmdRaw)
$javaProgram = ""
$javaArgs = @()
if ($hasJava) {
    $parts = @(Parse-CommandLine -CommandLine $javaCmdRaw)
    $javaProgram = $parts[0]
    if ($parts.Length -gt 1) {
        $javaArgs = $parts[1..($parts.Length - 1)]
    }
}

Write-Host "Benchmark root: $rootDir"
Write-Host "Rust binary:    $rustBin"
Write-Host "Rounds:         $rounds"
if (-not [string]::IsNullOrWhiteSpace($benchFilter)) {
    Write-Host "Filter:         $benchFilter"
}
if ($hasJava) {
    Write-Host "Java command:   $javaCmdRaw"
} else {
    Write-Host "Java command:   <not set, Java run skipped>"
}
Write-Host ""

$results = New-Object System.Collections.Generic.List[object]

foreach ($benchPath in $benchPaths) {
    Write-Host "=== $([System.IO.Path]::GetFileName($benchPath)) ==="
    $rustResult = Measure-RuntimeForBench -Runtime "rust" -Program $rustBin -BaseArgs @() -BenchPath $benchPath -Rounds $rounds
    $results.Add($rustResult)

    if ($hasJava) {
        $javaResult = Measure-RuntimeForBench -Runtime "java" -Program $javaProgram -BaseArgs $javaArgs -BenchPath $benchPath -Rounds $rounds
        $results.Add($javaResult)
    }
    Write-Host ""
}

Write-Host "Summary (ms)"
Write-Host ("{0,-8} {1,-8} {2,10} {3,10} {4,10} {5,8} {6,14}" -f "bench", "runtime", "avg", "min", "max", "rounds", "rust_vs_java")

foreach ($bench in ($results | Select-Object -ExpandProperty Bench -Unique)) {
    $rustRow = $results | Where-Object { $_.Bench -eq $bench -and $_.Runtime -eq "rust" } | Select-Object -First 1
    $javaRow = $results | Where-Object { $_.Bench -eq $bench -and $_.Runtime -eq "java" } | Select-Object -First 1

    $ratio = "-"
    if ($rustRow -and $javaRow -and $rustRow.AvgMs -gt 0) {
        $ratio = "$(Format-Decimal ($javaRow.AvgMs / $rustRow.AvgMs))x"
    }

    if ($rustRow) {
        Write-Host ("{0,-8} {1,-8} {2,10} {3,10} {4,10} {5,8} {6,14}" -f $bench, "rust", (Format-Decimal $rustRow.AvgMs), (Format-Decimal $rustRow.MinMs), (Format-Decimal $rustRow.MaxMs), $rustRow.Rounds, $ratio)
    }
    if ($javaRow) {
        Write-Host ("{0,-8} {1,-8} {2,10} {3,10} {4,10} {5,8} {6,14}" -f $bench, "java", (Format-Decimal $javaRow.AvgMs), (Format-Decimal $javaRow.MinMs), (Format-Decimal $javaRow.MaxMs), $javaRow.Rounds, "-")
    }
}
