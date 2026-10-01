# Mutation check for the benchmark harness.
#
# Every mutation below is a way the harness could be quietly unfair to one of the
# three libraries, or quietly flattering to this one, without anybody noticing
# from the output. Each is applied, the tests are run, and the change is reverted.
#
# The list is not exhaustive and it is not meant to be. It is the set of things
# that were actually wrong at least once while this crate was written, plus the
# obvious neighbours of those.

param([switch]$Keep)

$ErrorActionPreference = 'Continue'
$crate = $null
Push-Location $PSScriptRoot
try {
    $crate = (Resolve-Path '.').Path
} finally {
    Pop-Location
}

$env:VCPKG_ROOT = if ($env:MAGICAL_BENCH_VCPKG_ROOT) { $env:MAGICAL_BENCH_VCPKG_ROOT } else { $env:VCPKG_ROOT }
$env:VCPKGRS_TRIPLET = if ($env:VCPKGRS_TRIPLET) { $env:VCPKGRS_TRIPLET } else { 'x64-windows' }
$env:VCPKGRS_DYNAMIC = if ($env:VCPKGRS_DYNAMIC) { $env:VCPKGRS_DYNAMIC } else { '1' }

function Invoke-Cargo {
    param([string[]]$Args_)
    Push-Location $crate
    try {
        $out = & cargo @Args_ 2>&1 | Out-String
        return [pscustomobject]@{ Code = $LASTEXITCODE; Text = $out }
    } finally {
        Pop-Location
    }
}

# Replaces `find` with `replace` in a source file and puts the original back
# afterwards, whether or not the test failed. A mutation left behind is worse
# than no mutation: the next run would be testing the wrong thing and would say
# it passed.
function Invoke-Mutation {
    param(
        [string]$Name,
        [string]$File,
        [string]$Find,
        [string]$Replace,
        [string]$Test = '--test harness',
        [string[]]$CargoArgs = @('--features', 'libmagic')
    )

    $path = Join-Path $crate $File
    $original = [System.IO.File]::ReadAllText($path)
    if (-not $original.Contains($Find)) {
        Write-Host ("  SKIP  {0}: the text to change is not in {1} any more -- the mutation is stale" -f $Name, $File) -ForegroundColor DarkYellow
        return [pscustomobject]@{ Name = $Name; Caught = $false; Stale = $true }
    }

    [System.IO.File]::WriteAllText($path, $original.Replace($Find, $Replace))
    try {
        $run = Invoke-Cargo (@('test') + $CargoArgs + @($Test))
        $caught = $run.Code -ne 0
        $colour = if ($caught) { 'Green' } else { 'Red' }
        Write-Host ("  {0,-6} {1}" -f $(if ($caught) { 'CAUGHT' } else { 'MISSED' }), $Name) -ForegroundColor $colour
        if (-not $caught) {
            Write-Host '        the tests passed with the mutation in place' -ForegroundColor Red
        }
        return [pscustomobject]@{ Name = $Name; Caught = $caught; Stale = $false }
    } finally {
        [System.IO.File]::WriteAllText($path, $original)
    }
}

$mutations = @(
    @{
        Name = 'the report sorts its rows by time, best first'
        File = 'src\report.rs'
        Find = 'for m in rows {'
        Replace = 'for m in rows.iter().sorted_by_key(|m| m.median_ns as u64) {'
    },
    @{
        Name = 'the timing table hides a non-finite median by printing the slowest'
        File = 'src\report.rs'
        Find = '.filter(|n| n.is_finite())'
        Replace = '.map(|n| if n.is_finite() { n } else { 0.0 })'
    },
    @{
        Name = 'the ratio is computed against the fastest library, not the slowest'
        File = 'src\report.rs'
        Find = '.fold(0.0_f64, f64::max);'
        Replace = '.fold(f64::INFINITY, f64::min);'
    },
    @{
        Name = 'the startup table prints nothing at all'
        File = 'src\report.rs'
        Find = '            .filter(|m| m.startup.is_some())'
        Replace = '            .filter(|m| m.startup.is_some())
            .take(0)'
    },
    @{
        Name = 'the answer table is filtered to the cases that agree'
        File = 'src\report.rs'
        Find = 'for (row, answers) in self.answers.iter().zip(&cells) {'
        Replace = 'for (row, answers) in self.answers.iter().zip(&cells).filter(|(_, a)| all_agree(a)) {'
    },
    @{
        Name = 'the from-path table silently reuses the in-memory numbers'
        File = 'src\report.rs'
        Find = 'let from_path = measure(adapters, &[], &on_disk.paths);'
        Replace = 'let from_path = measure(adapters, &cases, &[]);'
    },
    @{
        Name = 'the warm read is removed, so the slow library evicts the corpus'
        File = 'src\report.rs'
        Find = '            warm(cases);
            samples[index].push'
        Replace = '            samples[index].push'
    },
    @{
        Name = 'the in-memory corpus is shorter than the on-disk one, so the two tables are not comparable'
        File = 'src\report.rs'
        Find = 'let cases: Vec<Case> = positives.into_iter().chain(negatives).collect();'
        Replace = 'let cases: Vec<Case> = positives.into_iter().chain(negatives.into_iter().take(1)).collect();'
    },
    @{
        Name = 'the corpus is not written to disk, so the from-path table reads the same file 286 times'
        File = 'src\report.rs'
        Find = 'let path = directory.join(&case.name);'
        Replace = 'let path = directory.join("the-same-file-every-time");'
    },
    @{
        Name = 'the real files are not found, so the answer table compares fixtures'
        File = 'src\corpus.rs'
        Find = '.find(|dir| dir.join("Makefile").is_file())'
        Replace = '.find(|dir| dir.join("Cargo.toml").is_file() && dir.join("Makefile").is_file() == false)'
    },
    @{
        Name = 'a negative case is generated from a seed that matches something'
        File = 'src\corpus.rs'
        Find = 'let bytes = clean_filler(seed, READ_SIZE);'
        Replace = 'let bytes = filler(FILLER_SEED ^ 0x5DEECE66D ^ seed, READ_SIZE);'
    },
    @{
        Name = 'the corpus is generated from a different seed each call'
        File = 'src\corpus.rs'
        Find = 'const FILLER_SEED: u64 = 0x6D61_6769_6361_6C72; // "magicalr"'
        Replace = 'const FILLER_SEED: u64 = 0x00C0_FFEE_1234_5678; // "the magic number"'
    },
    @{
        Name = 'the header size is the default rather than the full one'
        File = 'src\corpus.rs'
        Find = 'pub const READ_SIZE: usize = 36_870;'
        Replace = 'pub const READ_SIZE: usize = 2048;'
    },
    @{
        Name = 'a case name loses the signature index, so rows collide'
        File = 'src\corpus.rs'
        Find = 'format!("{}@{offset}-{which}", entry.kind.variant_name())'
        Replace = 'format!("{}@{offset}", entry.kind.variant_name())'
    },
    @{
        Name = 'the corpus declares one fewer positive than it builds'
        File = 'src\corpus.rs'
        Find = 'let expected: usize = SIGNATURE_KIND
            .iter()
            .filter(|entry| !matches!(entry.rules, MatchRules::WithFn(_)))
            .map(|entry| entry.signatures.len() * entry.offsets.len())
            .sum();'
        Replace = 'let expected: usize = SIGNATURE_KIND
            .iter()
            .filter(|entry| !matches!(entry.rules, MatchRules::WithFn(_)))
            .count();'
    },
    @{
        Name = 'the answer pass reports an expectation this crate does not hold'
        File = 'src\report.rs'
        Find = '.map_or_else(|| "nothing".to_owned(), |k| k.variant_name().to_owned()),'
        Replace = '.map_or_else(|| "nothing".to_owned(), |_| "a format".to_owned()),'
    },
    @{
        Name = 'the measured call is not black-boxed, so it can be deleted'
        File = 'src\report.rs'
        Find = '    sink_of(sink);
    elapsed * 1e9'
        Replace = '    let _ = sink;
    elapsed * 1e9'
    },
    @{
        Name = 'the ratio column is printed even when the median is not a number'
        File = 'src\report.rs'
        Find = 'let ratio = if m.median_ns > 0.0 && slowest > 0.0 {'
        Replace = 'let ratio = if slowest > 0.0 {'
    },
    @{
        Name = 'the in-memory table is measured on the paths instead of the buffers'
        File = 'src\report.rs'
        Find = 'let in_memory = measure(adapters, &cases, &[]);'
        Replace = 'let in_memory = measure(adapters, &[], &on_disk.paths);'
    },
    @{
        Name = 'the report claims the corpus is bigger than it is'
        File = 'src\report.rs'
        Find = 'self.positives + self.negatives,'
        Replace = 'self.positives + self.negatives + 1,'
    },
    @{
        Name = 'the corpus is not cleaned up out of the temporary directory'
        File = 'src\report.rs'
        Find = 'let _ = std::fs::remove_dir_all(&self.directory);'
        Replace = 'let _ = self.directory.metadata();'
    }
)

Write-Host ''
Write-Host 'Mutation check for the benchmark harness' -ForegroundColor Cyan
Write-Host ('  ' + $mutations.Count + ' mutations, each reverted after it is tried') -ForegroundColor DarkGray
Write-Host ''

$results = foreach ($m in $mutations) {
    Invoke-Mutation @m
}

$stale = @($results | Where-Object { $_.Stale })
$missed = @($results | Where-Object { -not $_.Caught -and -not $_.Stale })
$caught = @($results | Where-Object { $_.Caught })

Write-Host ''
Write-Host ("  {0} caught, {1} missed, {2} stale" -f $caught.Count, $missed.Count, $stale.Count) -ForegroundColor $(if ($missed.Count -eq 0 -and $stale.Count -eq 0) { 'Green' } else { 'Red' })

if ($missed.Count -gt 0) {
    Write-Host ''
    Write-Host '  Missed, which means a test does not exist for:' -ForegroundColor Red
    $missed | ForEach-Object { Write-Host ("    - " + $_.Name) -ForegroundColor Red }
}
if ($stale.Count -gt 0) {
    Write-Host ''
    Write-Host '  Stale, which means the mutation no longer applies to the code:' -ForegroundColor DarkYellow
    $stale | ForEach-Object { Write-Host ("    - " + $_.Name) -ForegroundColor DarkYellow }
}

Write-Host ''
if ($missed.Count -gt 0 -or $stale.Count -gt 0) {
    exit 1
}
Write-Host '  Every mutation was caught.' -ForegroundColor Green
exit 0
